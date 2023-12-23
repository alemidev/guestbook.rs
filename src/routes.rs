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

async fn send_suggestion(unsafe_payload: Insertion, state: SafeContext) -> Result<Redirect, String> {
	// sanitize all user input! we don't want XSS or html injections!
	let payload = unsafe_payload.sanitize();
	// limit author and contact fields to 25 and 50 characters, TODO don't hardcode limits
	let contact_limited = payload.contact.clone().map(|x| limit_string(&x, 50));
	let author_limited = payload.author.map(|x| limit_string(&x, 25));
	// calculate contact hash for libravatar
	let mut hasher = Md5::new();
	hasher.update(contact_limited.as_deref().unwrap_or(&Uuid::new_v4().to_string()).as_bytes());
	let avatar = hasher.finalize();
	// populate guestbook page struct
	let page = GuestBookPage {
		avatar: format!("{:x}", avatar),
		author: author_limited.unwrap_or("anonymous".to_string()),
		url: url_from_contact(contact_limited.clone()),
		contact: contact_limited,
		body: payload.body,
		date: Utc::now(),
	};
	// lock state, process and archive new page
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

fn url_from_contact(contact: Option<String>) -> Option<String> {
	match contact {
		None => None,
		Some(c) => if c.starts_with("http") {
			Some(c)
		} else if c.contains('@') {
			Some(format!("mailto:{}", c))
		} else if c.contains('.') {
			Some(format!("https://{}", c))
		} else {
			None
		}
	}
}

fn limit_string(s: &str, l: usize) -> String {
	// TODO is there a better way? slicing doesn't work when l > s.len
	s.chars().take(l).collect()
}
