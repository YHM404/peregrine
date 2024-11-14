use anyhow::Result;
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};

use crate::config::LogConfig;

const LOG_PATTERN: &str = "{d} {l} {t} - {m}{n}";

pub(crate) fn init_logger(log_config: Option<LogConfig>) -> Result<()> {
    let config = match log_config {
        Some(log_config) => {
            let appender = FileAppender::builder()
                .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
                .build(log_config.log_path)?;

            Config::builder()
                .appender(Appender::builder().build("logfile", Box::new(appender)))
                .build(
                    Root::builder()
                        .appender("logfile")
                        .build(log::LevelFilter::Info),
                )?
        }
        None => {
            let stdout = ConsoleAppender::builder()
                .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
                .build();

            Config::builder()
                .appender(Appender::builder().build("stdout", Box::new(stdout)))
                .build(
                    Root::builder()
                        .appender("stdout")
                        .build(log::LevelFilter::Info),
                )?
        }
    };

    let _ = log4rs::init_config(config)?;

    Ok(())
}
