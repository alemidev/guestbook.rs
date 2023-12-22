
#[cfg(feature = "telegram")]
pub mod telegram;

pub mod console;


#[async_trait::async_trait]
pub trait NotificationProcessor<T> : Send + Sync {
	async fn process(&self, notification: &T);
}
