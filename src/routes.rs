use std::sync::Arc;

use axum::{http::StatusCode, Json, Form, Router, routing::{put, post, get}, extract::{State, Query}, response::IntoResponse};
use tokio::sync::RwLock;

use crate::{notifications::NotificationProcessor, model::{GuestBookPage, Acknowledgement, PageOptions, Insertion}, storage::StorageStrategy};

pub fn create_router_with_app_routes(state: Context) -> Router {
	Router::new()
		.route("/api", get(get_suggestion))
		.route("/api", post(send_suggestion_form))
		.route("/api", put(send_suggestion_json))
		.with_state(Arc::new(RwLock::new(state)))
}

type SafeContext = Arc<RwLock<Context>>;

pub struct Context {
	providers: Vec<Box<dyn NotificationProcessor<GuestBookPage>>>,
	storage: Box<dyn StorageStrategy<GuestBookPage>>,
}

impl Context {
	pub fn new(storage: Box<dyn StorageStrategy<GuestBookPage>>) -> Self {
		Context { providers: Vec::new(), storage }
	}
	
	pub fn register(mut self, notifier: Box<dyn NotificationProcessor<GuestBookPage>>) -> Self {
		self.providers.push(notifier);
		self
	}

	async fn process(&self, x: &GuestBookPage) {
		for p in self.providers.iter() {
			p.process(x).await;
		}
	}
}

async fn send_suggestion(payload: Suggestion, state: SafeContext) -> impl IntoResponse {
	let mut lock = state.write().await;
	lock.process(&page).await;
	match lock.storage.archive(page).await {
		Ok(()) => (StatusCode::OK, Json(Acknowledgement::Sent("".into()))),
		Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(Acknowledgement::Refused(e.to_string()))),
	}
}

async fn send_suggestion_json(State(state): State<SafeContext>, Json(payload): Json<Insertion>) -> impl IntoResponse { send_suggestion(payload, state).await }
async fn send_suggestion_form(State(state): State<SafeContext>, Form(payload): Form<Insertion>) -> impl IntoResponse { send_suggestion(payload, state).await }


async fn get_suggestion(State(state): State<SafeContext>, Query(page): Query<PageOptions>) -> Result<Json<Vec<GuestBookPage>>, String> {
	let offset = page.offset.unwrap_or(0);
	let limit = std::cmp::min(page.limit.unwrap_or(20), 20);

	match state.read().await.storage.extract(offset, limit).await {
		Ok(x) => Ok(Json(x)),
		Err(e) => Err(e.to_string()),
	}
}
