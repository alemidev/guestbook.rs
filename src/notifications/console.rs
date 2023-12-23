use crate::model::GuestBookPage;

use super::NotificationProcessor;


pub struct ConsoleTracingNotifier {}

#[async_trait::async_trait]
impl NotificationProcessor<GuestBookPage> for ConsoleTracingNotifier {
	async fn process(&self, suggestion: &GuestBookPage) {
		tracing::info!(" >> {:?}", suggestion);
	}
}

pub struct ConsolePrettyNotifier {}

#[async_trait::async_trait]
impl NotificationProcessor<GuestBookPage> for ConsolePrettyNotifier {
	async fn process(&self, suggestion: &GuestBookPage) {
		println!("{} -- {} <{}>", suggestion.body, suggestion.author.as_deref().unwrap_or("anon"), suggestion.contact.as_deref().unwrap_or("")); 
	}
}
