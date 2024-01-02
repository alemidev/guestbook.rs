

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Config {
	pub overrides: ConfigOverrides,

	pub notifiers: Vec<ConfigNotifier>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ConfigOverrides {
	pub author: Option<String>,

	pub public: Option<bool>,

	pub date: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ConfigNotifier {
	ConsoleNotifier,

	#[cfg(feature = "telegram")]
	TelegramNotifier {
		token: String,
		chat_id: i64,
	},
}
