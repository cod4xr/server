// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

use cod4x_script::binding::{CanonicalScriptPath, LegacyScriptPath, ScriptPathBudget};
use cod4x_script::compilation::ScriptPackageBudget;

use crate::app_error::AppError;

pub(crate) struct ScriptPackageInput {
    pub(crate) name: CanonicalScriptPath,
    pub(crate) path: PathBuf,
}

impl fmt::Debug for ScriptPackageInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ScriptPackageInput")
            .field("name_bytes", &self.name.as_bytes().len())
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Default)]
pub(crate) struct ScriptPackageOptions {
    pub(crate) inputs: Vec<ScriptPackageInput>,
    raw_name_bytes: u64,
}

impl ScriptPackageOptions {
    pub(crate) fn push(&mut self, name: OsString, path: OsString) -> Result<(), AppError> {
        let budget = ScriptPackageBudget::default();
        if self.inputs.len() as u64 >= budget.max_modules {
            return Err(AppError::Arguments(
                "GSC package module count exceeds admission",
            ));
        }
        let raw = name
            .into_string()
            .map_err(|_| AppError::Arguments("GSC module name must be ASCII"))?;
        let next = self
            .raw_name_bytes
            .checked_add(raw.len() as u64)
            .ok_or(AppError::Arguments("GSC package name size overflow"))?;
        if next > budget.max_path_input_bytes {
            return Err(AppError::Arguments(
                "GSC package name bytes exceed admission",
            ));
        }
        let name = canonical_bytes(raw.as_bytes())?;
        self.inputs
            .try_reserve(1)
            .map_err(|_| AppError::Arguments("GSC package input allocation failed"))?;
        self.inputs.push(ScriptPackageInput {
            name,
            path: PathBuf::from(path),
        });
        self.raw_name_bytes = next;
        Ok(())
    }
}

pub(crate) fn canonical(raw: OsString) -> Result<CanonicalScriptPath, AppError> {
    let raw = raw
        .into_string()
        .map_err(|_| AppError::Arguments("GSC module name must be ASCII"))?;
    canonical_bytes(raw.as_bytes())
}

fn canonical_bytes(raw: &[u8]) -> Result<CanonicalScriptPath, AppError> {
    let path = LegacyScriptPath::new(raw, ScriptPathBudget::default())
        .map_err(|_| AppError::Arguments("invalid or excessive GSC virtual module name"))?
        .value;
    if path.canonical_len() == 0 {
        return Err(AppError::Arguments(
            "GSC virtual module name cannot be empty",
        ));
    }
    Ok(path
        .copy_owned(ScriptPathBudget::default())
        .map_err(|_| AppError::Arguments("GSC virtual module name ownership failed"))?
        .value)
}
