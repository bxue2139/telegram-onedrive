/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod docs;
mod reset;
mod set;
mod show;
mod utils;

use grammers_client::InputMessage;
use proc_macros::{add_context, add_trace};
use reset::{cancel_temp_dir, reset_dir};
use set::{set_dir, set_temp_dir};
use show::show_dir;

use super::utils::cmd_parser;
use crate::error::Result;
use crate::message::TelegramMessage;
use crate::state::AppState;
use crate::{check_in_group, check_od_login, check_senders};

pub const PATTERN: &str = "/dir";

#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    check_in_group!(message);
    check_senders!(message, state);
    check_od_login!(message, state);

    let onedrive = &state.onedrive;

    let cmd = cmd_parser(message.text());

    if cmd.len() == 1 {
        // /dir
        show_dir(onedrive, message).await?;
    } else if cmd.len() == 2 {
        if cmd[1] == "reset" {
            // /dir reset
            reset_dir(onedrive, message).await?;
        } else {
            // dir $root_path
            let root_path = &cmd[1];
            set_dir(onedrive, message, root_path).await?;
        }
    } else if cmd.len() == 3 {
        if cmd[1] == "temp" {
            if cmd[2] != "cancel" {
                // /dir temp $temp_root_path
                let temp_root_path = &cmd[2];
                set_temp_dir(onedrive, message, temp_root_path).await?;
            } else {
                // /dir temp cancel
                cancel_temp_dir(onedrive, message).await?;
            }
        } else {
            message
                .reply(InputMessage::html(format!(
                    "Unknown sub command for /dir\n{}",
                    docs::USAGE
                )))
                .await
                .context("sub command error")?;
        }
    } else {
        message
            .reply(InputMessage::html(format!(
                "Unknown command for /dir\n{}",
                docs::USAGE
            )))
            .await
            .context("command error")?;
    }

    Ok(())
}
