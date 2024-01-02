use teloxide::prelude::*;

use crate::model::Page;

use super::NotificationProcessor;

pub struct TGNotifier {
	bot: Bot,
	chat_id: i64,
}

impl TGNotifier {
	pub fn new(token: &str, chat_id: i64) -> Self {
		TGNotifier {
			chat_id,
			bot: Bot::new(token),
		}
	}
}

#[async_trait::async_trait]
impl NotificationProcessor<Page> for TGNotifier {
	async fn process(&self, notification: &Page) {
		let message = format!(
			"[<code>{}</code>] <i>{}</i> | {}",
			html_escape::encode_text(&notification.author),
			html_escape::encode_text(notification.contact.as_deref().unwrap_or("N/A")),
			html_escape::encode_text(&notification.body)
		);

		if let Err(e) = self.bot
			.send_message(ChatId(self.chat_id), message)
			.parse_mode(teloxide::types::ParseMode::Html)
			.await
		{
			tracing::error!("could not send message on telegram: {}", e);
		}
	}
}
