use std::sync::Arc;

use axum::{Json, Form, Router, routing::{put, post, get}, extract::{State, Query}, response::{Redirect, Html}};
use axum_extra::response::{Css, JavaScript};

use crate::{notifications::NotificationProcessor, model::{Page, PageOptions, PageInsertion, PageView}, storage::StorageProvider, web::IndexTemplate};

pub fn create_router_with_app_routes(state: Context) -> Router {
	let mut router = Router::new()
		.route("/api", get(get_suggestion))
		.route("/api", post(send_suggestion_form))
		.route("/api", put(send_suggestion_json));

	#[cfg(feature = "web")]
	{
		use sailfish::TemplateOnce;
		let template = state.template.clone();
		router = router
			.route("/favicon.ico", get(|| async { crate::web::STATIC_FAVICON }))
			.route("/logo.jpg", get(|| async { crate::web::STATIC_LOGO }))
			.route("/style.css", get(|| async { Css(crate::web::STATIC_CSS) }))
			.route("/infiniscroll.js", get(|| async { JavaScript(crate::web::STATIC_JS) }))
			.route("/", get(|| async move {
				match IndexTemplate::from(&template).render_once() {
					Ok(txt) => Ok(Html(txt)),
					Err(e) => Err((
						axum::http::StatusCode::INTERNAL_SERVER_ERROR,
						format!("could not render template: {}", e)
					)),
				}
			}));
	}

	router.with_state(Arc::new(state))
}

pub struct Context {
	providers: Vec<Box<dyn NotificationProcessor<Page>>>,
	storage: StorageProvider,

	#[cfg(feature = "web")]
	template: crate::config::ConfigTemplate,
}

impl Context {
	pub fn new(
		storage: StorageProvider,
		#[cfg(feature = "web")] template: crate::config::ConfigTemplate,
	) -> Self {
		Context {
			providers: Vec::new(),
			storage,
			#[cfg(feature = "web")] template,
		}
	}
	
	pub fn register(&mut self, notifier: Box<dyn NotificationProcessor<Page>>) {
		self.providers.push(notifier);
	}
}

async fn send_suggestion(payload: PageInsertion, state: Arc<Context>) -> Result<Redirect, String> {
	tracing::debug!("processing insertion {:?}", payload);
	match state.storage.archive(payload).await {
		Err(e) => Err(e.to_string()),
		Ok(page) => {
			for p in state.providers.iter() {
				p.process(&page).await;
			}
			Ok(Redirect::to("/"))
		},
	}
}

async fn send_suggestion_json(State(state): State<Arc<Context>>, Json(payload): Json<PageInsertion>) -> Result<Redirect, String> { send_suggestion(payload, state).await }
async fn send_suggestion_form(State(state): State<Arc<Context>>, Form(payload): Form<PageInsertion>) -> Result<Redirect, String> { send_suggestion(payload, state).await }


async fn get_suggestion(State(state): State<Arc<Context>>, Query(page): Query<PageOptions>) -> Result<Json<Vec<PageView>>, String> {
	let offset = page.offset.unwrap_or(0);
	let limit = std::cmp::min(page.limit.unwrap_or(20), 20);
	tracing::debug!("serving suggestions (offset {} limit {}", offset, limit);

	match state.storage.extract(offset, limit, true).await {
		Ok(x) => Ok(Json(x)),
		Err(e) => Err(e.to_string()),
	}
}
