// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use slog::Drain;

pub struct LogState {
    _guard: slog_scope::GlobalLoggerGuard,
}

pub fn init_logger() -> Result<LogState, Box<dyn Error>> {
    let decorator = slog_term::TermDecorator::new().stderr().build();

    let logger = slog::Logger::root(
        std::sync::Mutex::new(
            slog_term::CompactFormat::new(decorator)
                .build()
                .ignore_res(),
        )
        .fuse(),
        slog::o!(),
    );

    let _guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().unwrap();

    slog_scope::info!("Version: {}", std::env!("CARGO_PKG_VERSION"));
    if cfg!(feature = "debug") {
        slog_scope::debug!(
            "Debug build ({})",
            std::option_env!("GIT_HASH").unwrap_or("Unknown")
        );
    }

    Ok(LogState {
        _guard,
    })
}