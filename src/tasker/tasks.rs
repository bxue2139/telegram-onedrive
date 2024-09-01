/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use sea_orm::entity::prelude::DeriveEntityModel;
use sea_orm::sea_query::{ArrayType, ValueType, ValueTypeErr};
use sea_orm::{
    ActiveModelBehavior, ColIdx, ColumnType, DbErr, DerivePrimaryKey, DeriveRelation, EntityTrait,
    EnumIter, PrimaryKeyTrait, QueryResult, TryGetError, TryGetable, Value,
};
use std::fmt::Display;

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub cmd_type: CmdType,
    pub filename: String,
    pub root_path: String,
    pub url: Option<String>,
    pub upload_url: String,
    pub current_length: i64,
    pub total_length: i64,
    pub chat_bot_hex: String,
    pub chat_user_hex: String,
    pub chat_origin_hex: Option<String>,
    pub message_id: i32,
    pub message_id_forward: Option<i32>,
    pub message_id_origin: Option<i32>,
    pub status: TaskStatus,
}

#[derive(Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, PartialEq)]
pub enum CmdType {
    File,
    Link,
    Url,
}

impl ValueType for CmdType {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::String(Some(value)) => match value.as_str() {
                "file" => Ok(Self::File),
                "link" => Ok(Self::Link),
                "url" => Ok(Self::Url),
                _ => Err(ValueTypeErr),
            },
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        "CmdType".to_string()
    }

    fn array_type() -> ArrayType {
        ArrayType::String
    }

    fn column_type() -> ColumnType {
        ColumnType::String(None)
    }
}

impl From<CmdType> for Value {
    fn from(value: CmdType) -> Self {
        match value {
            CmdType::File | CmdType::Link | CmdType::Url => {
                Self::String(Some(Box::new(value.to_string())))
            }
        }
    }
}

impl TryGetable for CmdType {
    fn try_get_by<I: ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        let value: String = res.try_get_by(index)?;

        match value.as_str() {
            "file" => Ok(Self::File),
            "link" => Ok(Self::Link),
            "url" => Ok(Self::Url),
            _ => Err(TryGetError::DbErr(DbErr::Type(format!(
                "cmd type value should be one of file, photo, link and url: {}",
                value
            )))),
        }
    }
}

impl Display for CmdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File => write!(f, "file"),
            Self::Link => write!(f, "link"),
            Self::Url => write!(f, "url"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Waiting,
    Fetched,
    Started,
    Completed,
    Failed,
}

impl ValueType for TaskStatus {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::String(Some(value)) => match value.as_str() {
                "waiting" => Ok(Self::Waiting),
                "fetched" => Ok(Self::Fetched),
                "started" => Ok(Self::Started),
                "completed" => Ok(Self::Completed),
                "failed" => Ok(Self::Failed),
                _ => Err(ValueTypeErr),
            },
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        "TaskStatus".to_string()
    }

    fn array_type() -> ArrayType {
        ArrayType::String
    }

    fn column_type() -> ColumnType {
        ColumnType::String(None)
    }
}

impl From<TaskStatus> for Value {
    fn from(value: TaskStatus) -> Self {
        match value {
            TaskStatus::Waiting
            | TaskStatus::Fetched
            | TaskStatus::Started
            | TaskStatus::Completed
            | TaskStatus::Failed => Self::String(Some(Box::new(value.to_string()))),
        }
    }
}

impl TryGetable for TaskStatus {
    fn try_get_by<I: ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        let value: String = res.try_get_by(index)?;

        match value.as_str() {
            "waiting" => Ok(Self::Waiting),
            "fetched" => Ok(Self::Fetched),
            "started" => Ok(Self::Started),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err(TryGetError::DbErr(DbErr::Type(format!(
                "task status value should be one of waiting, started, completed and failed: {}",
                value
            )))),
        }
    }
}

impl Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Waiting => write!(f, "waiting"),
            Self::Fetched => write!(f, "fetched"),
            Self::Started => write!(f, "started"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}
