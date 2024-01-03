use serde_inline_default::serde_inline_default;
// note that serde would want us to create a function for each immediate value we would want to set
// as default. that gets VERY annoying VERY fast, and we have a lot of fields here.
// i really dislike what serde-inline-default does, just adding more complexity making all these
// immediate functions for me, but the code would be EXTREMELY bad to maintain otherwise
//
// track progress for official #[serde(default_value = ...)] here : https://github.com/serde-rs/serde/issues/368


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, serde_default::DefaultFromSerde)]
pub struct Config {
	#[serde(default)]
	pub overrides: ConfigOverrides,

	#[serde(default)]
	pub notifiers: ConfigNotifiers,

	#[serde(default)]
	pub template: ConfigTemplate,
}

#[serde_inline_default]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, serde_default::DefaultFromSerde)]
pub struct ConfigOverrides {
	#[serde(default)]
	pub author: Option<String>,

	#[serde(default)]
	pub public: Option<bool>,

	#[serde_inline_default(true)]
	pub date: bool,
}

#[serde_inline_default]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, serde_default::DefaultFromSerde)]
pub struct ConfigTemplate {
	#[serde_inline_default("guestbook".into())]
	pub title: String,

	#[serde_inline_default("/logo.jpg".into())]
	pub logo: String,

	#[serde_inline_default("you found my guestbook! please take a moment to sign it (:".into())]
	pub description: String,

	#[serde_inline_default("Kilroy was here  Ω".into())]
	pub placeholder_body: String,

	#[serde(default)]
	pub canonical: Option<String>,

	#[serde(default)]
	pub palette: ConfigPalette,
}

#[serde_inline_default]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, serde_default::DefaultFromSerde)]
pub struct ConfigPalette {
	#[serde_inline_default("#201F29".into())]
	pub bg_main: String,
	#[serde_inline_default("#292835".into())]
	pub bg_off: String,
	#[serde_inline_default("#E8E1D3".into())]
	pub fg_main: String,
	#[serde_inline_default("#ADA9A1".into())]
	pub fg_off: String,
	#[serde_inline_default("#BF616A".into())]
	pub accent: String,
	#[serde_inline_default("#824E53".into())]
	pub accent_dark: String,
	#[serde_inline_default("#D1888E".into())]
	pub accent_light: String,
}

#[serde_inline_default]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, serde_default::DefaultFromSerde)]
pub struct ConfigNotifiers {
	#[serde_inline_default(vec![NotifierProvider::Console])]
	pub providers: Vec<NotifierProvider>,
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
