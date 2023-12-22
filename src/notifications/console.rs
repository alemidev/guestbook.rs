use crate::model::Suggestion;

use super::NotificationProcessor;


pub struct ConsoleTracingNotifier {}

#[async_trait::async_trait]
impl NotificationProcessor<Suggestion> for ConsoleTracingNotifier {
	async fn process(&self, suggestion: &Suggestion) {
		tracing::info!(" >> {:?}", suggestion);
	}
}

pub struct ConsolePrettyNotifier {}

#[async_trait::async_trait]
impl NotificationProcessor<Suggestion> for ConsolePrettyNotifier {
	async fn process(&self, suggestion: &Suggestion) {
		println!("{} -- {} <{}>", suggestion.body, suggestion.author.as_deref().unwrap_or("anon"), suggestion.contact.as_deref().unwrap_or("")); 
	}
}
