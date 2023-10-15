use std::net::SocketAddr;
use axum::{routing::{get, put, post}, http::StatusCode, Json, Router, Form};
use serde::{Deserialize, Serialize};
use clap::Parser;
use teloxide::prelude::*;
use tokio::sync::RwLock;

lazy_static::lazy_static! {
	static ref BOT : Bot = Bot::from_env();
	static ref CHAT_ID : RwLock<ChatId> = RwLock::new(ChatId(0)); // TODO ewwwwwww
}

#[derive(Debug, Clone, Parser)]
/// api for sending anonymous telegram messages to a specific user
struct CliArgs {
	/// chat id of target user
	target: i64,

	#[arg(long, short, default_value = "127.0.0.1:37812")]
	/// host to bind onto
	addr: String,
}

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();

	let args = CliArgs::parse();

	*CHAT_ID.write().await = ChatId(args.target);

	let app = Router::new()
		.route("/send", get(suggestion_help))
		.route("/send", post(suggestion_form))
		.route("/send", put(suggestion_json));

	let addr : SocketAddr = args.addr.parse().expect("invalid host provided");

	tracing::info!("listening on {}", addr);

	axum::Server::bind(&addr)
		.serve(app.into_make_service())
		.await
		.unwrap();
}

async fn suggestion_help() -> (StatusCode, Json<Suggestion>) {
	let body = "use this endpoint to send me direct messages. this is the accepted structure. on PUT send as json, on POST send as form-urlencoded".to_string();
	(StatusCode::OK, Json(Suggestion { author: None, contact: None, body }))
}

async fn suggestion_json(Json(payload): Json<Suggestion>) -> (StatusCode, Json<Acknowledgement>) {
	suggestion_inner(payload).await
}

async fn suggestion_form(Form(payload): Form<Suggestion>) -> (StatusCode, Json<Acknowledgement>) {
	suggestion_inner(payload).await
}

async fn suggestion_inner(payload: Suggestion) -> (StatusCode, Json<Acknowledgement>) {
	let message = format!(
		"[<code>{}</code>] <i>{}</i> | {}",
		html_escape::encode_text(payload.author.as_deref().unwrap_or("anon")),
		html_escape::encode_text(payload.contact.as_deref().unwrap_or("N/A")),
		html_escape::encode_text(&payload.body)
	);

	match BOT
		.send_message(*CHAT_ID.read().await, message)
		.parse_mode(teloxide::types::ParseMode::Html)
		.await
	{
		Ok(x) => (StatusCode::OK, Json(Acknowledgement::Sent(x.text().unwrap_or("").into()))),
		Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(Acknowledgement::Refused(e.to_string()))),
	}
}

#[derive(Serialize, Deserialize)]
struct Suggestion {
	author: Option<String>,
	contact: Option<String>,
	body: String,
}

#[derive(Serialize)]
enum Acknowledgement {
	Sent(String),
	Refused(String),
}
