/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use proc_macros::{
    add_context, add_trace, check_in_group, check_od_login, check_senders, check_tg_login,
};
use reqwest::header;

use super::utils::{cmd_parser, get_filename, TextExt};
use crate::error::{Error, Result};
use crate::message::TelegramMessage;
use crate::state::AppState;
use crate::tasker::CmdType;
use crate::utils::get_http_client;

pub const PATTERN: &str = "/url";

#[check_od_login]
#[check_tg_login]
#[check_senders]
#[check_in_group]
#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let cmd = cmd_parser(message.text());

    if cmd.len() == 2 {
        let telegram_user = &state.telegram_user;
        let onedrive = &state.onedrive;
        let task_session = state.task_session.clone();

        let url = cmd[1].url_encode();

        if url.starts_with("http://") || url.starts_with("https://") {
            let http_client = get_http_client()?;

            let response =
                http_client.head(&url).send().await.map_err(|e| {
                    Error::new_http_request(e, "failed to send head request for /url")
                })?;

            let filename = get_filename(&url, &response)?;

            let total_length = match response.headers().get(header::CONTENT_LENGTH) {
                Some(content_length) => content_length
                    .to_str()
                    .map_err(|e| Error::new_http_header_to_str(e, "header Content-Length has invisible ASCII chars"))?
                    .parse::<u64>()
                    .map_err(|e| Error::new_parse_int(e, "failed to parse header Content-Length to u64"))?,
                None => return Err(Error::new(format!(
                    "Content-Length not found in response headers.\nStatus code:\n{}\nResponse headers:\n{:#?}",
                    response.status(),
                    response.headers()
                ))),
            };

            let root_path = onedrive.get_root_path(true).await?;

            let (upload_session, upload_session_meta) = onedrive
                .multipart_upload_session_builder(&root_path, &filename)
                .await?;

            let current_length = upload_session_meta
                .next_expected_ranges
                .first()
                .map_or(0, |range| range.start);

            let chat_bot_hex = message.chat().pack().to_hex();
            let chat_user_hex = telegram_user
                .get_chat(message.clone())
                .await?
                .pack()
                .to_hex();

            task_session
                .insert_task(
                    CmdType::Url,
                    &filename,
                    &root_path,
                    Some(url),
                    upload_session.upload_url(),
                    current_length,
                    total_length,
                    &chat_bot_hex,
                    &chat_user_hex,
                    None,
                    message.id(),
                    None,
                    None,
                )
                .await
        } else {
            Err(Error::new("not an http url"))
        }
    } else {
        Err(Error::new("Unknown command for /url"))
    }
}
