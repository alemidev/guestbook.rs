use std::{net::SocketAddr, io::Write};
use clap::{Parser, Subcommand};
use config::ConfigOverrides;

use crate::{storage::StorageProvider, routes::Context, notifications::console::ConsoleTracingNotifier, config::{Config, NotifierProvider}};

mod notifications;

mod routes;
mod model;
mod storage;
mod config;


#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
/// api for sending anonymous telegram messages to a specific user
struct CliArgs {
	/// action to execute
	#[clap(subcommand)]
	action: CliAction,

	/// connection string of storage database
	#[arg(long, default_value = "sqlite://./guestbook.db")]
	db: String,

	#[arg(long, default_value_t = false)]
	/// increase log verbosity to DEBUG level
	debug: bool,
}

#[derive(Debug, Clone, Subcommand)]
enum CliAction {
	/// serve guestbook api
	Serve {
		#[arg(default_value = "127.0.0.1:37812")]
		/// host to bind onto
		addr: String,

		#[arg(long, short)]
		/// path to config file for overrides and notifiers
		config: Option<String>,
	},

	/// print a sample configuration, redirect to file and customize
	Default,

	/// review sent pages and approve for public view
	Review {
		#[arg(long, default_value_t = 20)]
		/// how many pages to fetch per query
		batch: i32,
	},
}

#[tokio::main]
async fn main() {
	let args = CliArgs::parse();

	tracing_subscriber::fmt::fmt()
		.with_max_level(if args.debug { tracing::Level::DEBUG } else { tracing::Level::INFO })
		.init();

	match args.action {
		CliAction::Default => {
			let mut cfg = Config::default();
			cfg.notifiers.providers.push(NotifierProvider::Console);
			#[cfg(feature = "telegram")]
			cfg.notifiers.providers.push(NotifierProvider::Telegram { token: "asd".into(), chat_id: -1 });
			println!("{}", toml::to_string(&cfg).unwrap());
		},
		CliAction::Review { batch } => {
			sqlx::any::install_default_drivers(); // must install all available drivers before connecting
			let storage = StorageProvider::connect(&args.db, ConfigOverrides::default()).await.unwrap();
			let mut offset = 0;
			let mut buffer = String::new();
			let stdin = std::io::stdin();
			let mut stdout = std::io::stdout();
			loop {
				let mut stop = true;
				for page in storage.extract(offset, batch, false).await.unwrap() {
					stop = false; // at least one page was returned
					println!("{:?}", page);
					print!("* approve? ");
					stdout.flush().unwrap();
					stdin.read_line(&mut buffer).unwrap();
					if !buffer.trim().is_empty() {
						println!("* OK published");
						storage.publish(page.id).await.unwrap();
					}
				}
				if stop { break }
				offset += batch;
			}
			println!("* done");
		},
		CliAction::Serve { addr, config } => {
			let addr : SocketAddr = addr.parse().expect("invalid host provided");

			let config = match config {
				None => Config::default(),
				Some(path) => {
					let cfg_file = std::fs::read_to_string(path).unwrap();
					toml::from_str(&cfg_file).unwrap()
				}
			};

			sqlx::any::install_default_drivers(); // must install all available drivers before connecting
			let storage = StorageProvider::connect(&args.db, config.overrides).await.unwrap();

			let mut state = Context::new(storage);

			for notifier in config.notifiers.providers {
				match notifier {
					NotifierProvider::Console => {
						tracing::info!("registering console notifier");
						state.register(Box::new(ConsoleTracingNotifier {}));
					},

					#[cfg(feature = "telegram")]
					NotifierProvider::Telegram { token, chat_id } => {
						tracing::info!("registering telegram notifier for chat {}", chat_id);
						state.register(Box::new(
							notifications::telegram::TGNotifier::new(&token, chat_id)
						));
					},

					#[cfg(feature = "email")]
					NotifierProvider::Email {
						server, port, username, password, from, to, subject
					} => {
						tracing::info!("registering email notifier to {} on server {}:{} ('{}')", to, server, port, subject);
						state.register(Box::new(
							notifications::email::EmailNotifier::new(
								&server, port, &username, &password, &from, &to, &subject
							).await
						));

					}
				}
			}

			let router = routes::create_router_with_app_routes(state);

			tracing::info!("listening on {}", addr);

			axum::Server::bind(&addr)
				.serve(router.into_make_service())
				.await
				.unwrap();
		}
	}
}
