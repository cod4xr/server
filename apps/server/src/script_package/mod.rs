// SPDX-License-Identifier: AGPL-3.0-or-later
mod frontend;
#[cfg(test)]
mod frontend_tests;
pub(crate) mod options;
#[cfg(test)]
mod options_tests;
mod package_runner;
mod phases;

pub(crate) use package_runner::run;
