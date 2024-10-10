/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::sync::atomic::Ordering;

use super::utils::upload::upload_thumb;
use crate::{
    env::BYPASS_PREFIX,
    error::{Error, Result},
    handlers::utils::{get_tg_file_size, preprocess_tg_file_name},
    message::{ChatEntity, TelegramMessage},
    state::AppState,
    tasker::{CmdType, InsertTask},
};
use grammers_client::{types::Media, InputMessage};
use proc_macros::{
    add_context, add_trace, check_in_group, check_od_login, check_senders, check_tg_login,
};

#[check_od_login]
#[check_tg_login]
#[check_senders]
#[check_in_group]
#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let telegram_user = &state.telegram_user;
    let onedrive = &state.onedrive;
    let task_session = &state.task_session;

    let chat_user = telegram_user
        .get_chat(&ChatEntity::from(message.chat()))
        .await?;

    let message_user = telegram_user.get_message(&chat_user, message.id()).await?;

    let media = message_user
        .media()
        .ok_or_else(|| Error::new("message does not contain any media"))?;

    let filename = preprocess_tg_file_name(&media);

    let total_length = get_tg_file_size(&media);

    let mut message_id = message.id();
    let mut message_id_forward = None;

    let cmd_type = match media {
        Media::Photo(_) | Media::Document(_) | Media::Sticker(_) => CmdType::File,
        _ => Err(Error::new(
            "media type is not one of photo, document and sticker",
        ))?,
    };

    // if message is forwarded, or is grouped in a album, send its file name and thumb if exists
    // so that information of uploading successful can be showed
    if message_user.forward_header().is_some() || message_user.raw.grouped_id().is_some() {
        message_id_forward = Some(message_id);

        let uploaded = match media {
            Media::Photo(file) => upload_thumb(telegram_user, file.thumbs()).await?,
            Media::Document(file) => upload_thumb(telegram_user, file.thumbs()).await?,
            Media::Sticker(file) => upload_thumb(telegram_user, file.document.thumbs()).await?,
            _ => Err(Error::new(
                "media type is not one of photo, document and sticker",
            ))?,
        };

        let response = format!("{}{}", BYPASS_PREFIX, filename);
        match uploaded {
            Some(uploaded) => {
                message_id = telegram_user
                    .send_message(
                        &chat_user,
                        InputMessage::text(response.as_str()).photo(uploaded),
                    )
                    .await
                    .context("forwarded message with thumb")
                    .details(response)?
                    .id();
            }
            None => {
                message_id = telegram_user
                    .send_message(&chat_user, response.as_str())
                    .await
                    .context("forwarded message without thumn")
                    .details(response)?
                    .id();
            }
        }
    }

    let root_path = onedrive.get_root_path(true).await?;

    let (upload_session, upload_session_meta) = onedrive
        .multipart_upload_session_builder(&root_path, &filename)
        .await?;

    // all task should be new, so this should always be 0
    let current_length = upload_session_meta
        .next_expected_ranges
        .first()
        .map_or(0, |range| range.start);

    let chat_bot_hex = message.chat().pack().to_hex();
    let chat_user_hex = chat_user.pack().to_hex();

    let auto_delete = state.should_auto_delete.load(Ordering::Acquire);

    let _aborters = state.task_session.aborters.lock().await;

    task_session
        .insert_task(InsertTask {
            cmd_type,
            filename: filename.clone(),
            root_path,
            url: None,
            upload_url: upload_session.upload_url().to_string(),
            current_length,
            total_length,
            chat_id: chat_user.id(),
            chat_bot_hex,
            chat_user_hex,
            chat_origin_hex: None,
            message_id,
            message_id_forward,
            message_id_origin: None,
            auto_delete,
        })
        .await?;

    tracing::info!("inserted file task: {} size: {}", filename, total_length);

    Ok(())
}
