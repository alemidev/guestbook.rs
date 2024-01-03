
#[cfg(feature = "telegram")]
pub mod telegram;

#[cfg(feature = "email")]
pub mod email;

pub mod console;


#[async_trait::async_trait]
pub trait NotificationProcessor<T> : Send + Sync {
	async fn process(&self, notification: &T);
}
