// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ffi::OsString;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;

use cod4x_observability::{ConsoleColor, ConsoleFormat, LogConfig};

use crate::app_error::AppError;

pub(crate) const HELP: &str = "CoD4X Rust development server

Usage: cod4x-server --query-only [options]
       cod4x-server --handshake-probe [options]
       cod4x-server --verify-iwd PATH [logging options]

  --query-only             Enable discovery only; gameplay is not implemented
  --handshake-probe        Exchange challenges; refuse connects without gameplay
  --verify-iwd PATH        Verify all IWD entries offline; no network listener
  --bind ADDRESS           UDP address (default: 127.0.0.1:28960)
  --log-dir PATH           Dedicated directory for daily JSON log files
  --log-level DIRECTIVES   Explicit tracing filter (default: info)
  --human-logs             Pretty stderr (default in development builds)
  --json-logs              JSON stderr (default in release builds)
  --color MODE             auto, always, or never (default: auto)
  --run-for-ms NUMBER      Stop after this duration (1..604800000 milliseconds)
  --max-datagrams NUMBER   Stop after this many received datagrams (nonzero)
  --help                   Show this help
  --version                Show the development build version
";

#[derive(Debug)]
pub(crate) struct Arguments {
    pub(crate) mode: Mode,
    pub(crate) bind: SocketAddr,
    pub(crate) logs: LogConfig,
    pub(crate) duration: Option<Duration>,
    pub(crate) max_datagrams: Option<u64>,
}

#[derive(Debug)]
pub(crate) enum Mode {
    QueryOnly,
    HandshakeProbe,
    VerifyIwd(PathBuf),
}

#[derive(Debug)]
pub(crate) enum Action {
    Help,
    Version,
    Run(Arguments),
}

impl Arguments {
    pub(crate) fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Action, AppError> {
        let mut options = Self {
            mode: Mode::QueryOnly,
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 28960)),
            logs: LogConfig::default(),
            duration: None,
            max_datagrams: None,
        };
        let mut query_only = false;
        let mut handshake_probe = false;
        let mut verify_iwd = None;
        let mut query_settings = false;
        let mut arguments = arguments.into_iter();
        while let Some(argument) = arguments.next() {
            let Some(argument) = argument.to_str() else {
                return Err(AppError::Arguments("option names must be ASCII"));
            };
            match argument {
                "--help" => return Ok(Action::Help),
                "--version" => return Ok(Action::Version),
                "--query-only" => query_only = true,
                "--handshake-probe" => handshake_probe = true,
                "--verify-iwd" => {
                    if verify_iwd.is_some() {
                        return Err(AppError::Arguments("--verify-iwd accepts one archive"));
                    }
                    verify_iwd = Some(PathBuf::from(
                        arguments
                            .next()
                            .ok_or(AppError::Arguments("--verify-iwd needs a path"))?,
                    ));
                }
                "--human-logs" => options.logs.console_format = ConsoleFormat::Human,
                "--json-logs" => options.logs.console_format = ConsoleFormat::Json,
                "--log-dir" => {
                    let value = arguments
                        .next()
                        .ok_or(AppError::Arguments("--log-dir needs a path"))?;
                    options.logs.directory = Some(PathBuf::from(value));
                }
                "--bind" | "--log-level" | "--run-for-ms" | "--max-datagrams" | "--color" => {
                    let value = arguments
                        .next()
                        .ok_or(AppError::Arguments("option requires a value"))?;
                    let value = value
                        .into_string()
                        .map_err(|_| AppError::Arguments("option value must be UTF-8"))?;
                    query_settings |=
                        matches!(argument, "--bind" | "--run-for-ms" | "--max-datagrams");
                    options.apply_text_option(argument, value)?;
                }
                _ => return Err(AppError::Arguments("unknown option; use --help")),
            }
        }
        if let Some(path) = verify_iwd {
            if query_only || handshake_probe || query_settings {
                return Err(AppError::Arguments(
                    "archive verification cannot use query options",
                ));
            }
            options.mode = Mode::VerifyIwd(path);
        } else if handshake_probe {
            if query_only {
                return Err(AppError::Arguments("select one network mode"));
            }
            options.mode = Mode::HandshakeProbe;
        } else if !query_only {
            return Err(AppError::Arguments(
                "gameplay is not implemented; select an explicit development mode; use --help",
            ));
        }
        Ok(Action::Run(options))
    }

    fn apply_text_option(&mut self, name: &str, value: String) -> Result<(), AppError> {
        match name {
            "--bind" => {
                self.bind = value
                    .parse()
                    .map_err(|_| AppError::Arguments("invalid UDP bind address"))?;
            }
            "--log-level" => self.logs.filter = value,
            "--color" => {
                self.logs.console_color = match value.as_str() {
                    "auto" => ConsoleColor::Auto,
                    "always" => ConsoleColor::Always,
                    "never" => ConsoleColor::Never,
                    _ => return Err(AppError::Arguments("color must be auto, always, or never")),
                };
            }
            "--run-for-ms" => {
                let milliseconds: u64 = value
                    .parse()
                    .map_err(|_| AppError::Arguments("invalid run duration"))?;
                if !(1..=604_800_000).contains(&milliseconds) {
                    return Err(AppError::Arguments("run duration must be 1 ms..7 days"));
                }
                self.duration = Some(Duration::from_millis(milliseconds));
            }
            "--max-datagrams" => {
                let maximum: u64 = value
                    .parse()
                    .map_err(|_| AppError::Arguments("invalid datagram count"))?;
                if maximum == 0 {
                    return Err(AppError::Arguments("datagram count must be nonzero"));
                }
                self.max_datagrams = Some(maximum);
            }
            _ => return Err(AppError::Arguments("unsupported text option")),
        }
        Ok(())
    }
}
