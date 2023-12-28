use crate::model::Page;

use super::NotificationProcessor;


pub struct ConsoleTracingNotifier {}

#[async_trait::async_trait]
impl NotificationProcessor<Page> for ConsoleTracingNotifier {
	async fn process(&self, suggestion: &Page) {
		tracing::info!(" >> {:?}", suggestion);
	}
}

pub struct ConsolePrettyNotifier {}

#[async_trait::async_trait]
impl NotificationProcessor<Page> for ConsolePrettyNotifier {
	async fn process(&self, suggestion: &Page) {
		println!("{} -- {} <{}>", suggestion.body, suggestion.author, suggestion.contact.as_deref().unwrap_or("")); 
	}
}
