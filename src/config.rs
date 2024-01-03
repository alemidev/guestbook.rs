

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Config {
	pub overrides: ConfigOverrides,

	pub notifiers: ConfigNotifiers,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ConfigOverrides {
	pub author: Option<String>,

	pub public: Option<bool>,

	#[serde(default = "_true")]
	pub date: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigNotifiers {
	pub providers: Vec<NotifierProvider>,
}

// by default enable console notifier
impl Default for ConfigNotifiers {
	fn default() -> Self {
		ConfigNotifiers {
			providers: vec![NotifierProvider::Console],
		}
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NotifierProvider {
	Console,

	#[cfg(feature = "telegram")]
	Telegram {
		token: String,
		chat_id: i64,
	},

	#[cfg(feature = "email")]
	Email {
		server: String,
		port: u16,
		username: String,
		password: String,
		from: String,
		to: String,
		subject: String,
	},
}

fn _true() -> bool { true }
