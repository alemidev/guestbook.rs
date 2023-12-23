use crate::model::GuestBookPage;


#[derive(Debug, thiserror::Error)]
pub enum StorageStrategyError {
	#[error("could not interact with filesystem: {0}")]
	IOError(#[from] std::io::Error),
	#[error("could not serialize/deserialize data: {0}")]
	JsonSerializeError(#[from] serde_json::Error),
}

#[async_trait::async_trait]
pub trait StorageStrategy<T> : Send + Sync {
	async fn archive(&mut self, payload: T) -> Result<(), StorageStrategyError>;
	async fn extract(&self, offset: usize, window: usize) -> Result<Vec<T>, StorageStrategyError>;
}


/// this strategy is rather inefficient since it has to iterate the whole file every time, but it
/// requires literally zero effort
pub struct JsonFileStorageStrategy {
	path: String,
}

impl JsonFileStorageStrategy {
	pub fn new(path: &str) -> Self {
		JsonFileStorageStrategy { path: path.to_string() }
	}
}


#[async_trait::async_trait]
impl StorageStrategy<GuestBookPage> for JsonFileStorageStrategy {
	async fn archive(&mut self, payload: GuestBookPage) -> Result<(), StorageStrategyError> {
		let file_content = std::fs::read_to_string(&self.path)?;
		let mut current_content : Vec<GuestBookPage> = serde_json::from_str(&file_content)?;
		current_content.push(payload);
		let updated_content = serde_json::to_string(&current_content)?;
		std::fs::write(&self.path, updated_content)?;
		Ok(())
	}

	async fn extract(&self, offset: usize, window: usize) -> Result<Vec<GuestBookPage>, StorageStrategyError> {
		let file_content = std::fs::read_to_string(&self.path)?;
		let current_content : Vec<GuestBookPage> = serde_json::from_str(&file_content)?;
		let mut out = Vec::new();
		for sugg in current_content.iter().rev().skip(offset) {
			out.push(sugg.clone());
			if out.len() >= window { break };
		}
		Ok(out)
	}
}
