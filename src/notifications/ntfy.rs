use crate::model::Page;

use super::NotificationProcessor;

pub struct NtfyNotifier {
	server: String,
	topic: String,
}

impl NtfyNotifier {
	pub fn new(server: String, topic: String) -> Self {
		NtfyNotifier { server, topic }
	}
}

#[async_trait::async_trait]
impl NotificationProcessor<Page> for NtfyNotifier {
	async fn process(&self, notification: &Page) {
		let author = if let Some(ref contact) = notification.contact {
			format!("{} <{}>", notification.author, contact)
		} else {
			notification.author.clone()
		};
		match reqwest::Client::new()
			.post(format!("{}/{}", self.server, self.topic))
			.body(notification.body.clone())
			.header("Title", author)
			.header("Tags", "computer")
			.header("Priority", "low")
			.send()
			.await
		{
			Err(e) => tracing::error!("failed sending ntfy event: {e}"),
			Ok(res) => if let Err(e) = res.error_for_status() {
				tracing::error!("ntfy server rejected event: {e}");
			},
		}
	}
}
