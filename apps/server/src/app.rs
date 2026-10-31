// SPDX-License-Identifier: AGPL-3.0-or-later
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use cod4x_observability::LogRuntime;

use crate::app_error::AppError;
use crate::arguments::{self, Action, Arguments, Mode};
use crate::content;
use crate::runtime::Runtime;
use crate::service::RuntimeService;

pub(crate) fn run() -> Result<(), AppError> {
    let options = match Arguments::parse(std::env::args_os().skip(1))? {
        Action::Help => {
            print!("{}", arguments::HELP);
            return Ok(());
        }
        Action::Version => {
            println!(
                "cod4x-server {} (query-only development)",
                env!("CARGO_PKG_VERSION")
            );
            return Ok(());
        }
        Action::Run(options) => options,
    };
    let logs = LogRuntime::build(&options.logs).map_err(AppError::Logging)?;
    tracing::dispatcher::set_global_default(logs.dispatch().clone())
        .map_err(AppError::Subscriber)?;
    match &options.mode {
        Mode::VerifyIwd(path) => return content::verify_iwd(path),
        Mode::InspectFastFile(path) => return content::inspect_fastfile(path),
        Mode::InspectZonePrefix(path) => return content::inspect_zone_prefix(path),
        Mode::QueryOnly | Mode::HandshakeProbe => {}
    }
    let service = match RuntimeService::for_mode(&options.mode) {
        Ok(service) => service,
        Err(error) => {
            tracing::error!(
                event = "server.failed",
                kind = "initialization",
                "development service initialization failed"
            );
            return Err(error);
        }
    };
    let mode = service.label();
    // The signal handler and main thread share only this shutdown flag.
    let stop = Arc::new(AtomicBool::new(false));
    let signal_stop = Arc::clone(&stop);
    ctrlc::set_handler(move || signal_stop.store(true, Ordering::Relaxed))
        .map_err(AppError::Signal)?;
    let result = (|| {
        let mut runtime = Runtime::bind(options.bind, service)?;
        let address = runtime.local_address()?;
        tracing::info!(event = "server.started", version = env!("CARGO_PKG_VERSION"),
            bind = %address, mode, "development UDP runtime started");
        runtime.run(&options, &stop, &logs)
    })();
    match result {
        Ok(reason) => {
            tracing::info!(
                event = "server.stopped",
                reason,
                dropped_logs = logs.dropped_records(),
                "development UDP runtime stopped"
            );
            Ok(())
        }
        Err(error) => {
            tracing::error!(event = "server.failed", kind = ?error.kind(),
                "development UDP runtime failed");
            Err(AppError::Io(error))
        }
    }
}
