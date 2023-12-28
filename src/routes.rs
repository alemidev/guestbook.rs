use std::sync::Arc;

use axum::{Json, Form, Router, routing::{put, post, get}, extract::{State, Query}, response::Redirect};
use chrono::Utc;
use md5::{Md5, Digest};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{notifications::NotificationProcessor, model::{Page, PageOptions, PageInsertion}, storage::StorageStrategy};

pub fn create_router_with_app_routes(state: Context) -> Router {
	Router::new()
		.route("/api", get(get_suggestion))
		.route("/api", post(send_suggestion_form))
		.route("/api", put(send_suggestion_json))
		.with_state(Arc::new(RwLock::new(state)))
}

type SafeContext = Arc<RwLock<Context>>;

pub struct Context {
	providers: Vec<Box<dyn NotificationProcessor<Page>>>,
	storage: Box<dyn StorageStrategy<Page>>,
}

impl Context {
	pub fn new(storage: Box<dyn StorageStrategy<Page>>) -> Self {
		Context { providers: Vec::new(), storage }
	}
	
	pub fn register(mut self, notifier: Box<dyn NotificationProcessor<Page>>) -> Self {
		self.providers.push(notifier);
		self
	}

	async fn process(&self, x: &Page) {
		for p in self.providers.iter() {
			p.process(x).await;
		}
	}
}

async fn send_suggestion(payload: PageInsertion, state: SafeContext) -> Result<Redirect, String> {
	let page = Page::from(payload);
	// lock state, process and archive new page
	let mut lock = state.write().await;
	lock.process(&page).await;
	match lock.storage.archive(page).await {
		Ok(()) => Ok(Redirect::to("/")),
		Err(e) => Err(e.to_string()),
	}
}

async fn send_suggestion_json(State(state): State<SafeContext>, Json(payload): Json<PageInsertion>) -> Result<Redirect, String> { send_suggestion(payload, state).await }
async fn send_suggestion_form(State(state): State<SafeContext>, Form(payload): Form<PageInsertion>) -> Result<Redirect, String> { send_suggestion(payload, state).await }


async fn get_suggestion(State(state): State<SafeContext>, Query(page): Query<PageOptions>) -> Result<Json<Vec<Page>>, String> {
	let offset = page.offset.unwrap_or(0);
	let limit = std::cmp::min(page.limit.unwrap_or(20), 20);

	match state.read().await.storage.extract(offset, limit).await {
		Ok(x) => Ok(Json(x)),
		Err(e) => Err(e.to_string()),
	}
}

fn limit_string(s: &str, l: usize) -> String {
	// TODO is there a better way? slicing doesn't work when l > s.len
	s.chars().take(l).collect()
}
