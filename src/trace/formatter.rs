/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use ansi_term::Color;
use tracing::{Level, Subscriber};
use tracing_subscriber::fmt::time::{ChronoLocal, FormatTime};
use tracing_subscriber::fmt::{self, FormatFields};
use tracing_subscriber::fmt::{format, FormatEvent};
use tracing_subscriber::registry::LookupSpan;

use super::visitor::MessageVisitor;

pub struct EventFormatter {
    color: bool,
}

impl EventFormatter {
    pub fn new(color: bool) -> Self {
        Self { color }
    }
}

impl<S, N> FormatEvent<S, N> for EventFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &fmt::FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        write_time(&mut writer)?;
        write_level(&mut writer, event)?;

        if self.color {
            let level = event.metadata().level();

            let mut message_visitor = MessageVisitor::default();
            event.record(&mut message_visitor);
            let message = message_visitor.message;

            write_colored_message(&mut writer, *level, message)?;
        } else {
            ctx.field_format().format_fields(writer.by_ref(), event)?;
        }

        writeln!(writer)
    }
}

pub fn write_time(writer: &mut format::Writer<'_>) -> std::fmt::Result {
    if ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f ".to_string())
        .format_time(writer)
        .is_err()
    {
        write!(writer, "Time error ")?;
    }

    Ok(())
}

pub fn write_level(
    writer: &mut format::Writer<'_>,
    event: &tracing::Event<'_>,
) -> std::fmt::Result {
    let level = event.metadata().level();
    write!(writer, "{:>5} ", level)
}

pub fn write_colored_message(
    writer: &mut format::Writer<'_>,
    level: Level,
    mut message: String,
) -> std::fmt::Result {
    match level {
        tracing::Level::ERROR => message = Color::Red.paint(message).to_string(),
        tracing::Level::WARN => message = Color::Yellow.paint(message).to_string(),
        tracing::Level::INFO => message = Color::Green.paint(message).to_string(),
        tracing::Level::DEBUG => message = Color::Blue.paint(message).to_string(),
        tracing::Level::TRACE => {
            if message.contains("->") {
                message = Color::Purple.paint(message).to_string();
            } else if message.contains("<-") {
                message = Color::Cyan.paint(message).to_string();
            }
        }
    }

    write!(writer, "{}", message)
}
