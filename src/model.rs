use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GuestBookPage {
	pub author: Option<String>,
	pub contact: Option<String>,
	pub body: String,
	pub date: DateTime<Utc>,
	pub avatar: String,
	pub url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Insertion {
	pub author: Option<String>,
	pub contact: Option<String>,
	pub body: String,
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

