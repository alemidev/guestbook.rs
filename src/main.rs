use std::net::SocketAddr;
use clap::Parser;

use crate::{storage::JsonFileStorageStrategy, routes::Context, notifications::console::ConsoleTracingNotifier};

mod notifications;

mod routes;
mod model;
mod storage;


#[derive(Debug, Clone, Parser)]
/// api for sending anonymous telegram messages to a specific user
struct CliArgs {
	// chat id of target user
	//target: i64,

	#[arg(long, short, default_value = "127.0.0.1:37812")]
	/// host to bind onto
	addr: String,
}

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();

	let args = CliArgs::parse();

	let addr : SocketAddr = args.addr.parse().expect("invalid host provided");

	let storage = Box::new(JsonFileStorageStrategy::new("./storage.json"));

	let state = Context::new(storage)
		.register(Box::new(ConsoleTracingNotifier {}));

	let router = routes::create_router_with_app_routes(state);

	tracing::info!("listening on {}", addr);

	axum::Server::bind(&addr)
		.serve(router.into_make_service())
		.await
		.unwrap();
}
