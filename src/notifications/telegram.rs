use teloxide::prelude::*;

lazy_static::lazy_static! {
	static ref BOT : Bot = Bot::from_env();
	static ref CHAT_ID : RwLock<ChatId> = RwLock::new(ChatId(0)); // TODO ewwwwwww
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
