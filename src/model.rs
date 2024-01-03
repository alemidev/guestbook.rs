use md5::{Md5, Digest};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::config::ConfigOverrides;

const AUTHOR_MAX_CHARS: usize = 25;
const CONTACT_MAX_CHARS: usize = 50;
const BODY_MAX_CHARS: usize = 4096;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Page {
	pub id: i64,
	pub author: String,
	pub contact: Option<String>,
	pub body: String,
	pub timestamp: i64, // sqlx::Any db doesn't support DateTime<Utc>
	pub public: bool,
}

// TODO this is only necessary until sqlx fixes parsing BOOL and NULL, check model.rs for more
impl<'r> sqlx::FromRow<'r, sqlx::any::AnyRow> for Page {
	fn from_row(row: &'r sqlx::any::AnyRow) -> Result<Self, sqlx::Error> {
		Ok(
			Page {
				id: row.get::<i64, usize>(0),
				author: row.get::<String, usize>(1),
				contact: _non_empty_string(row.get::<String, usize>(2)),
				body: row.get::<String, usize>(3),
				timestamp: row.get::<i64, usize>(4),
				public: row.get::<i32, usize>(5) > 0,
			}
		)
	}
}



#[derive(Debug, Clone, Default, Serialize)]
pub struct PageView {
	pub id: i64,
	pub author: String,
	pub contact: Option<String>,
	pub url: Option<String>,
	pub avatar: String,
	pub body: String,
	pub date: DateTime<Utc>,
}

impl From<&Page> for PageView {
	fn from(page: &Page) -> Self {
		let mut hasher = Md5::new();
		hasher.update(page.contact.as_deref().unwrap_or(&Uuid::new_v4().to_string()).as_bytes());
		let avatar = format!("{:x}", hasher.finalize());

		let url = match page.contact.as_deref() {
			None => None,
			Some(c) => if c.starts_with("http") {
				Some(c.to_string())
			} else if c.contains('@') {
				Some(format!("mailto:{}", c))
			} else if c.contains('.') {
				Some(format!("https://{}", c))
			} else {
				None
			}
		};

		PageView {
			id: page.id,
			url, avatar,
			author: page.author.clone(),
			contact: page.contact.clone(),
			body: page.body.clone(),
			date: DateTime::from_timestamp(page.timestamp, 0).unwrap_or(DateTime::UNIX_EPOCH),
		}
	}
}



#[derive(Debug, Clone, Default, Deserialize)]
pub struct PageInsertion {
	pub body: String,
	pub author: Option<String>,
	pub contact: Option<String>,
	pub public: Option<bool>,
	pub date: Option<DateTime<Utc>>,
}

impl PageInsertion {
	fn trim_and_escape(input: &str, len: usize) -> String {
		html_escape::encode_safe(&input.chars().take(len).collect::<String>()).to_string()
	}

	pub fn sanitize(&mut self) {
		self.author = self.author.as_mut()
			.map(|a| Self::trim_and_escape(a, AUTHOR_MAX_CHARS).to_string())
			.and_then(|a| if a.is_empty() { None } else { Some(a) });

		self.contact = self.contact.as_mut()
			.map(|c| Self::trim_and_escape(c, CONTACT_MAX_CHARS).to_string())
			.and_then(|c| if c.is_empty() { None } else { Some(c) });

		self.body = Self::trim_and_escape(&self.body, BODY_MAX_CHARS);
	}

	pub fn overrides(&mut self, overrides: &ConfigOverrides) {
		if let Some(public) = overrides.public { self.public = Some(public) };
		if let Some(author) = &overrides.author { self.author = Some(author.clone()) };
		if overrides.date { self.date = Some(Utc::now()) };
	}
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageOptions {
	pub offset: Option<i32>,
	pub limit: Option<i32>,
}

fn _non_empty_string(input: String) -> Option<String> {
	match input.is_empty() {
		true => None,
		false => Some(input),
	}
}
