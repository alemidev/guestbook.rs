use std::sync::Arc;

use axum::{http::StatusCode, Json, Form, Router, routing::{put, post, get}, extract::{State, Query}, response::IntoResponse};
use tokio::sync::RwLock;

use crate::{notifications::NotificationProcessor, model::{Suggestion, Acknowledgement, PageOptions}, storage::StorageStrategy};

pub fn create_router_with_app_routes(state: Context) -> Router {
	Router::new()
		.route("/send", get(get_suggestion))
		.route("/send", post(send_suggestion_form))
		.route("/send", put(send_suggestion_json))
		.with_state(Arc::new(RwLock::new(state)))
}

type SafeContext = Arc<RwLock<Context>>;

pub struct Context {
	providers: Vec<Box<dyn NotificationProcessor<Suggestion>>>,
	storage: Box<dyn StorageStrategy<Suggestion>>,
}

impl Context {
	pub fn new(storage: Box<dyn StorageStrategy<Suggestion>>) -> Self {
		Context { providers: Vec::new(), storage }
	}
	
	pub fn register(mut self, notifier: Box<dyn NotificationProcessor<Suggestion>>) -> Self {
		self.providers.push(notifier);
		self
	}

	async fn process(&self, x: &Suggestion) {
		for p in self.providers.iter() {
			p.process(x).await;
		}
	}
}

async fn send_suggestion(payload: Suggestion, state: SafeContext) -> impl IntoResponse {
	let mut lock = state.write().await;
	lock.process(&payload).await;
	match lock.storage.archive(payload).await {
		Ok(()) => (StatusCode::OK, Json(Acknowledgement::Sent("".into()))),
		Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(Acknowledgement::Refused(e.to_string()))),
	}
}

async fn send_suggestion_json(State(state): State<SafeContext>, Json(payload): Json<Suggestion>) -> impl IntoResponse { send_suggestion(payload, state).await }
async fn send_suggestion_form(State(state): State<SafeContext>, Form(payload): Form<Suggestion>) -> impl IntoResponse { send_suggestion(payload, state).await }


async fn get_suggestion(State(state): State<SafeContext>, Query(page): Query<PageOptions>) -> Result<Json<Vec<Suggestion>>, String> {
	let offset = page.offset.unwrap_or(0);
	let limit = std::cmp::min(page.limit.unwrap_or(20), 20);

	match state.read().await.storage.extract(offset, limit).await {
		Ok(x) => Ok(Json(x)),
		Err(e) => Err(e.to_string()),
	}
}
