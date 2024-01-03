use chrono::Utc;

use crate::{model::{PageView, PageInsertion, Page}, config::ConfigOverrides};

#[derive(Debug)]
pub struct StorageProvider {
	db: sqlx::Pool<sqlx::Any>,
	overrides: ConfigOverrides,
}

// TODO bool type is not supported in Any driver?????
//  so the `public` field is an integer which is ridicolous
//  but literally cannot get it to work ffs
//
//  https://github.com/launchbadge/sqlx/issues/2778

const SQLITE_SCHEMA : &str = "
CREATE TABLE IF NOT EXISTS pages (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	author VARCHAR NOT NULL,
	contact VARCHAR,
	body VARCHAR NOT NULL,
	timestamp INTEGER NOT NULL,
	public INTEGER NOT NULL
);
";

const POSTGRES_SCHEMA : &str = "
CREATE TABLE IF NOT EXISTS pages (
	id SERIAL PRIMARY KEY,
	author TEXT NOT NULL,
	contact TEXT,
	body TEXT NOT NULL,
	timestamp INTEGER NOT NULL,
	public INTEGER NOT NULL
);
";

impl StorageProvider {
	pub async fn connect(dest: &str, overrides: ConfigOverrides) -> sqlx::Result<Self> {
		let db = sqlx::AnyPool::connect(dest).await?;

		match db.acquire().await?.backend_name() {
			"PostgreSQL" => { sqlx::query(POSTGRES_SCHEMA).execute(&db).await?; },
			"SQLite" => { sqlx::query(SQLITE_SCHEMA).execute(&db).await?; },
			"MySQL" => { sqlx::query(SQLITE_SCHEMA).execute(&db).await?; }, // TODO will this work?
			_ => tracing::warn!("could not ensure schema: unsupported database type"),
		}

		Ok(StorageProvider { db, overrides })
	}

	pub async fn archive(&self, mut page: PageInsertion) -> sqlx::Result<Page> {
		page.sanitize();
		page.overrides(&self.overrides);
		let result = sqlx::query("INSERT INTO pages (author, contact, body, timestamp, public) VALUES ($1, $2, $3, $4, $5)")
			.bind(page.author.as_deref().unwrap_or("anonymous").to_string())
			.bind(page.contact.clone())
			.bind(page.body.clone())
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

	pub async fn extract(&self, offset: i32, window: i32) -> sqlx::Result<Vec<PageView>> {
		// TODO since AnyPool won't handle booleans we compare with an integer
		let out = sqlx::query_as("SELECT * FROM pages WHERE public = 1 LIMIT $1 OFFSET $2")
			.bind(window)
			.bind(offset)
			.fetch_all(&self.db)
			.await?
			.iter()
			.map(PageView::from)
			.collect();
		Ok(out)
	}
}
