

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Config {
	pub overrides: ConfigOverrides,

	pub notifiers: ConfigNotifiers,

	pub template: ConfigTemplate,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ConfigOverrides {
	pub author: Option<String>,

	pub public: Option<bool>,

	#[serde(default = "_true")]
	pub date: bool,
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigTemplate {
	pub title: String,
	pub logo: String,
	pub description: String,
	pub placeholder_body: String,
	pub canonical: Option<String>,
	pub palette: ConfigPalette,
}

impl Default for ConfigTemplate {
	fn default() -> Self {
		ConfigTemplate {
			title: "guestbook.rs".into(),
			logo: "https://cdn.alemi.dev/social/someriver.jpg".into(),
			description: "you found my guestbook! please take a moment to sign it (:".into(),
			placeholder_body: "Kilroy was here  Ω".into(),
			canonical: None,
			palette: ConfigPalette::default(),
		}
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigPalette {
	pub bg_main: String,
	pub bg_off: String,
	pub fg_main: String,
	pub fg_off: String,
	pub accent: String,
	pub accent_dark: String,
	pub accent_light: String,
}

impl Default for ConfigPalette {
	fn default() -> Self {
		ConfigPalette {
			bg_main: "#201F29".into(),
			bg_off: "#292835".into(),
			fg_main: "#E8E1D3".into(),
			fg_off: "#ADA9A1".into(),
			accent: "#BF616A".into(),
			accent_dark: "#824E53".into(),
			accent_light: "#D1888E".into(),
		}
	}
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
