use std::sync::Arc;

use axum::{Json, Form, Router, routing::{put, post, get}, extract::{State, Query}, response::Redirect};

use crate::{notifications::NotificationProcessor, model::{Page, PageOptions, PageInsertion}, storage::StorageStrategy, CliServeOverrides};

pub fn create_router_with_app_routes(state: Context) -> Router {
	Router::new()
		.route("/api", get(get_suggestion))
		.route("/api", post(send_suggestion_form))
		.route("/api", put(send_suggestion_json))
		.with_state(Arc::new(state))
}

pub struct Context {
	providers: Vec<Box<dyn NotificationProcessor<Page>>>,
	storage: Box<dyn StorageStrategy<Page>>,
	overrides: CliServeOverrides,
}

impl Context {
	pub fn new(storage: Box<dyn StorageStrategy<Page>>, overrides: CliServeOverrides) -> Self {
		Context { providers: Vec::new(), storage, overrides }
	}
	
	pub fn register(&mut self, notifier: Box<dyn NotificationProcessor<Page>>) {
		self.providers.push(notifier);
	}
}

async fn send_suggestion(payload: PageInsertion, state: Arc<Context>) -> Result<Redirect, String> {
	let page = payload.convert(&state.overrides);
	for p in state.providers.iter() {
		p.process(&page).await;
	}
	match state.storage.archive(page).await {
		Ok(()) => Ok(Redirect::to("/")),
		Err(e) => Err(e.to_string()),
	}
}

async fn send_suggestion_json(State(state): State<Arc<Context>>, Json(payload): Json<PageInsertion>) -> Result<Redirect, String> { send_suggestion(payload, state).await }
async fn send_suggestion_form(State(state): State<Arc<Context>>, Form(payload): Form<PageInsertion>) -> Result<Redirect, String> { send_suggestion(payload, state).await }


async fn get_suggestion(State(state): State<Arc<Context>>, Query(page): Query<PageOptions>) -> Result<Json<Vec<Page>>, String> {
	let offset = page.offset.unwrap_or(0);
	let limit = std::cmp::min(page.limit.unwrap_or(20), 20);

	match state.storage.extract(offset, limit).await {
		Ok(x) => Ok(Json(x)),
		Err(e) => Err(e.to_string()),
	}
}
