use clap::{Parser, Subcommand};
use config::ConfigOverrides;

use crate::{storage::StorageProvider, routes::Context, notifications::console::ConsoleTracingNotifier, config::{Config, NotifierProvider}};

mod notifications;

mod routes;
mod model;
mod storage;
mod config;

#[cfg(feature = "web")]
mod web;


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
	Config,

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
		CliAction::Config => println!("{}", toml::to_string(&Config::default()).unwrap()),
		CliAction::Review { batch } => {
			use std::io::Write;
			sqlx::any::install_default_drivers(); // must install all available drivers before connecting
			if_using_sqlite_driver_and_file_is_missing_create_it_beforehand(&args.db);
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
			let config = match config {
				None => Config::default(),
				Some(path) => {
					let cfg_file = std::fs::read_to_string(path).unwrap();
					toml::from_str(&cfg_file).unwrap()
				}
			};

			sqlx::any::install_default_drivers(); // must install all available drivers before connecting
			if_using_sqlite_driver_and_file_is_missing_create_it_beforehand(&args.db);
			let storage = StorageProvider::connect(&args.db, config.overrides).await.unwrap();

			let mut state = Context::new(storage, #[cfg(feature = "web")] config.template);

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

			tracing::info!("serving on http://{}/", addr);

			let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

			axum::serve(listener, router)
				.await
				.unwrap();
		}
	}
}

// it drives me nuts that it doesn't do it by default!!! is there an option?
fn if_using_sqlite_driver_and_file_is_missing_create_it_beforehand(uri: &str) {
	use std::str::FromStr;
	if !uri.starts_with("sqlite://") { return };
	let path = uri.replace("sqlite://", "");
	if let Ok(p) = std::path::PathBuf::from_str(&path) {
		if !p.is_file() {
			if let Err(e) = std::fs::File::create(p) {
				tracing::warn!("could not create sqlite database file at {} : {}", path, e);
			}
		}
	}

}
