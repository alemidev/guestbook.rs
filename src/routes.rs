use std::sync::Arc;

use axum::{Json, Form, Router, routing::{put, post, get}, extract::{State, Query}, response::Redirect};
use chrono::Utc;
use md5::{Md5, Digest};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{notifications::NotificationProcessor, model::{GuestBookPage, PageOptions, Insertion}, storage::StorageStrategy};

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

async fn send_suggestion(payload: Insertion, state: SafeContext) -> Result<Redirect, String> {
	let mut hasher = Md5::new();
	let id = payload.contact.clone().unwrap_or(Uuid::new_v4().to_string());
	hasher.update(id.as_bytes());
	let avatar = hasher.finalize();
	let page = GuestBookPage {
		avatar: format!("{:x}", avatar),
		author: payload.author.map(|x| x.chars().take(25).collect()), // TODO don't hardcode char limits!
		contact: payload.contact.clone().map(|x| x.chars().take(50).collect()),
		body: payload.body,
		date: Utc::now(),
		url: match payload.contact {
			None => None,
			Some(c) => if c.starts_with("http") {
				Some(c)
			} else if c.contains('@') {
				Some(format!("mailto:{}", c))
			} else {
				None
			}
		},
	};
	let mut lock = state.write().await;
	lock.process(&page).await;
	match lock.storage.archive(page).await {
		Ok(()) => Ok(Redirect::to("/")),
		Err(e) => Err(e.to_string()),
	}
}

async fn send_suggestion_json(State(state): State<SafeContext>, Json(payload): Json<Insertion>) -> Result<Redirect, String> { send_suggestion(payload, state).await }
async fn send_suggestion_form(State(state): State<SafeContext>, Form(payload): Form<Insertion>) -> Result<Redirect, String> { send_suggestion(payload, state).await }


async fn get_suggestion(State(state): State<SafeContext>, Query(page): Query<PageOptions>) -> Result<Json<Vec<GuestBookPage>>, String> {
	let offset = page.offset.unwrap_or(0);
	let limit = std::cmp::min(page.limit.unwrap_or(20), 20);

	match state.read().await.storage.extract(offset, limit).await {
		Ok(x) => Ok(Json(x)),
		Err(e) => Err(e.to_string()),
	}
}
