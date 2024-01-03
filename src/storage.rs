use chrono::Utc;
use sqlx::Database;

use crate::{model::{PageView, PageInsertion, Page}, config::ConfigOverrides};

#[derive(Debug)]
pub struct StorageProvider {
	db: sqlx::Pool<sqlx::Any>,
	overrides: ConfigOverrides,
}

// TODO what the fuck is wrong with AnyPool driver?????
// * deserializing Option<T> values on AnyDriver is broken, pr to fix is in progress
//    https://github.com/launchbadge/sqlx/issues/2416
//    https://github.com/launchbadge/sqlx/pull/2716
//   until this is merged, must implement by hand sqlx::FromRow<sqlx::any::AnyRow>
//   once this is merged, just do #[derive(sqlx::FromRow)]
//
// * serializing Option<T> values on Postgres is unreliable, not even sure why?
//    `incorrect binary data format in bind parameter`
//    it seems related to expression caching and types that change (sometimes T, sometimes NULL)
//     https://github.com/launchbadge/sqlx/issues/2885
//    the suggested solution (`.persistent(false)`) doesn't work!
//    we are forced to serialize NULLs as empty strings and lose the distinction
//
// * deserializing BOOL just doesn't work for some reason
//    https://github.com/launchbadge/sqlx/issues/2778
//   so the `public` field is an integer which is ridicolous

const SQLITE_SCHEMA : &str = "
CREATE TABLE IF NOT EXISTS pages (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	author VARCHAR NOT NULL,
	contact VARCHAR NOT NULL,
	body VARCHAR NOT NULL,
	timestamp INTEGER NOT NULL,
	public INTEGER NOT NULL
);
";

const POSTGRES_SCHEMA : &str = "
CREATE TABLE IF NOT EXISTS pages (
	id SERIAL PRIMARY KEY,
	author TEXT NOT NULL,
	contact TEXT NOT NULL,
	body TEXT NOT NULL,
	timestamp INTEGER NOT NULL,
	public INTEGER NOT NULL
);
";

impl StorageProvider {
	pub async fn connect(dest: &str, overrides: ConfigOverrides) -> sqlx::Result<Self> {
		let db = sqlx::AnyPool::connect(dest).await?;

		match db.acquire().await?.backend_name() {
			sqlx::Postgres::NAME => { sqlx::query(POSTGRES_SCHEMA).execute(&db).await?; },
			sqlx::Sqlite::NAME => { sqlx::query(SQLITE_SCHEMA).execute(&db).await?; },
			sqlx::MySql::NAME => { sqlx::query(SQLITE_SCHEMA).execute(&db).await?; }, // TODO will this work?
			_ => tracing::warn!("could not ensure schema: unsupported database type"),
		}

		Ok(StorageProvider { db, overrides })
	}

	pub async fn archive(&self, mut page: PageInsertion) -> sqlx::Result<Page> {
		page.sanitize();
		page.overrides(&self.overrides);
		let result = sqlx::query("INSERT INTO pages (author, contact, body, timestamp, public) VALUES ($1, $2, $3, $4, $5)")
			.bind(page.author.as_deref().unwrap_or("anonymous"))
			.bind(page.contact.as_deref().unwrap_or(""))
			.bind(page.body.as_str())
			.bind(page.date.unwrap_or(Utc::now()).timestamp())
			.bind(if page.public.unwrap_or(true) { 1 } else { 0 })
			.execute(&self.db)
			.await?;
		Ok(
			Page {
				id: result.last_insert_id().unwrap_or(-1),
				author: page.author.unwrap_or("anonymous".into()),
				contact: page.contact,
				body: page.body,
				timestamp: page.date.unwrap_or(Utc::now()).timestamp(),
				public: page.public.unwrap_or(true),
			}
		)
	}

	pub async fn extract(&self, offset: i32, window: i32, public: bool) -> sqlx::Result<Vec<PageView>> {
		let out = sqlx::query_as::<_, Page>("SELECT * FROM pages WHERE public = $1 ORDER BY timestamp DESC LIMIT $2 OFFSET $3")
			.bind(if public { 1 } else { 0 }) // TODO since AnyPool won't handle booleans we compare with an integer
			.bind(window)
			.bind(offset)
			.fetch_all(&self.db)
			.await?
			.iter()
			.map(PageView::from)
			.collect();
		Ok(out)
	}

	pub async fn publish(&self, id: i64) -> sqlx::Result<()> {
		sqlx::query("UPDATE pages SET public = 1 WHERE id = $1")
			.bind(id)
			.execute(&self.db)
			.await?;
		Ok(())
	}
}
