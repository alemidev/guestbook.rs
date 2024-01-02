use std::net::SocketAddr;
use clap::{Parser, Subcommand};

use crate::{storage::JsonFileStorageStrategy, routes::Context, notifications::console::ConsoleTracingNotifier};

mod notifications;

mod routes;
mod model;
mod storage;


#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
/// api for sending anonymous telegram messages to a specific user
struct CliArgs {
	/// action to execute
	#[clap(subcommand)]
	action: CliAction,

	#[arg(long, default_value_t = false)]
	/// increase log verbosity to DEBUG level
	debug: bool,
}

#[derive(Debug, Clone, Subcommand)]
enum CliAction {
	Serve {
		#[arg(long, short, default_value = "127.0.0.1:37812")]
		/// host to bind onto
		addr: String,

		#[arg(long)]
		/// force public field content
		public: Option<bool>,

		#[arg(long)]
		/// force author field content
		author: Option<String>,
	}
}

struct CliServeOverrides {
	author: Option<String>,
	public: Option<bool>,
}

#[tokio::main]
async fn main() {
	let args = CliArgs::parse();

	tracing_subscriber::fmt::fmt()
		.with_max_level(if args.debug { tracing::Level::DEBUG } else { tracing::Level::INFO })
		.pretty()
		.finish();

	match args.action {
		CliAction::Serve { addr, public, author } => {
			let addr : SocketAddr = addr.parse().expect("invalid host provided");

			let storage = Box::new(JsonFileStorageStrategy::new("./storage.json"));

			let overrides = CliServeOverrides { author, public };

			let mut state = Context::new(storage, overrides);

			state.register(Box::new(ConsoleTracingNotifier {}));

			let router = routes::create_router_with_app_routes(state);

			tracing::info!("listening on {}", addr);

			axum::Server::bind(&addr)
				.serve(router.into_make_service())
				.await
				.unwrap();
		}
	}
}
