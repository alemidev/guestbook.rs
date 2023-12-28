use md5::{Md5, Digest};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

const AUTHOR_MAX_CHARS: usize = 25;
const CONTACT_MAX_CHARS: usize = 50;
const BODY_MAX_CHARS: usize = 4096;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Page {
	pub author: String,
	pub contact: Option<String>,
	pub body: String,
	pub date: DateTime<Utc>,
	pub public: bool,
}




#[derive(Debug, Clone, Default, Serialize)]
pub struct PageView {
	pub author: String,
	pub contact: Option<String>,
	pub url: Option<String>,
	pub avatar: String,
	pub body: String,
	pub date: DateTime<Utc>,
}

impl From<Page> for PageView {
	fn from(page: Page) -> Self {
		let mut hasher = Md5::new();
		hasher.update(page.contact.as_deref().unwrap_or(&Uuid::new_v4().to_string()).as_bytes());
		let avatar = format!("{:x}", hasher.finalize());

		let url = match page.contact {
			None => None,
			Some(c) => if c.starts_with("http") {
				Some(c)
			} else if c.contains('@') {
				Some(format!("mailto:{}", c))
			} else if c.contains('.') {
				Some(format!("https://{}", c))
			} else {
				None
			}
		};

		PageView {
			url, avatar,
			author: page.author,
			contact: page.contact,
			body: page.body,
			date: page.date,
		}
	}
}




#[derive(Debug, Clone, Default, Deserialize)]
pub struct PageInsertion {
	#[serde(deserialize_with = "non_empty_str")]
	pub author: Option<String>,

	#[serde(deserialize_with = "non_empty_str")]
	pub contact: Option<String>,

	pub body: String,
}

impl PageInsertion {
	pub fn sanitize(&mut self) {
		self.author = self.author.map(|x| html_escape::encode_safe(&x.chars().take(AUTHOR_MAX_CHARS).collect::<String>()).to_string());
		self.contact = self.contact.map(|x| html_escape::encode_safe(&x.chars().take(CONTACT_MAX_CHARS).collect::<String>()).to_string());
		self.body = html_escape::encode_safe(&self.body.chars().take(BODY_MAX_CHARS).collect::<String>()).to_string();
	}

	pub fn convert(mut self, overrides: crate::CliServeOverrides) -> Page {
		self.sanitize();

		let mut page = Page {
			author: self.author.unwrap_or("".into()),
			contact: self.contact,
			body: self.body,
			date: Utc::now(),
			public: true,
		};

		if let Some(author) = overrides.author {
			page.author = author;
		}
		if let Some(public) = overrides.public {
			page.public = public;
		}

		page
	}
}





#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Acknowledgement {
	Sent(String),
	Refused(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageOptions {
	pub offset: Option<usize>,
	pub limit: Option<usize>,
}

fn non_empty_str<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
	Ok(Option::deserialize(d)?.filter(|s: &String| !s.is_empty()))
}
