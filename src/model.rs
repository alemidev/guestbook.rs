use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GuestBookPage {
	pub author: String,
	pub body: String,
	pub date: DateTime<Utc>,
	pub avatar: String,
	pub url: Option<String>,
	pub contact: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Insertion {
	#[serde(deserialize_with = "non_empty_str")]
	pub author: Option<String>,

	#[serde(deserialize_with = "non_empty_str")]
	pub contact: Option<String>,

	pub body: String,
}

impl Insertion {
	pub fn sanitize(self) -> Self {
		Insertion {
			author: self.author.map(|x| html_escape::encode_safe(&x).to_string()),
			contact: self.contact.map(|x| html_escape::encode_safe(&x).to_string()),
			body: html_escape::encode_safe(&self.body).to_string(),
		}
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
