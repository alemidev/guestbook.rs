use mail_send::{SmtpClientBuilder, SmtpClient, mail_builder::MessageBuilder};
use tokio::{sync::Mutex, net::TcpStream};
use tokio_rustls::client::TlsStream;

use crate::model::Page;

use super::NotificationProcessor;

pub struct EmailNotifier {
	client: Mutex<SmtpClient<TlsStream<TcpStream>>>,
	from: String,
	to: String,
	subject: String,
}

impl EmailNotifier {
	pub async fn new(
			server: &str,
			port: u16,
			username: &str,
			password: &str,
			from: &str,
			to: &str,
			subject: &str,
	) -> Self {
		let client = SmtpClientBuilder::new(server, port)
			.implicit_tls(false)
			.credentials((username, password))
			.connect()
			.await
			.unwrap();

		EmailNotifier {
			client: Mutex::new(client),
			from: from.to_string(),
			to: to.to_string(),
			subject: subject.to_string()
		}
	}
}

#[async_trait::async_trait]
impl NotificationProcessor<Page> for EmailNotifier {
	async fn process(&self, notification: &Page) {
		let message = MessageBuilder::new()
			.from((notification.author.as_str(), notification.contact.as_ref().unwrap_or(&self.from).as_str()))
			.to(vec![("Guestbook", self.to.as_str())])
			.subject(&self.subject)
			.text_body(&notification.body);

		if let Err(e) = self.client.lock().await.send(message).await {
			tracing::error!("could not send message via email: {}", e);
		}

	}
}
