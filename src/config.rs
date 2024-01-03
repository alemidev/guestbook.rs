

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

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ConfigNotifiers {
	pub providers: Vec<ConfigNotifierProvider>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ConfigNotifierProvider {
	ConsoleNotifier,

	#[cfg(feature = "telegram")]
	TelegramNotifier {
		token: String,
		chat_id: i64,
	},
}

fn _true() -> bool { true }
