// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_script::compilation::{CompiledModule, CompiledScriptPackage};
use cod4x_script::execution::VerifiedKernelModule;
use cod4x_script::source::{SourceLocation, SourceText};

#[derive(Clone, Copy)]
pub(crate) enum ScriptImage<'image, 'source> {
    Module {
        source: &'image SourceText<'source>,
        compiled: &'image CompiledModule,
    },
    Package {
        sources: &'image [SourceText<'source>],
        compiled: &'image CompiledScriptPackage,
    },
}

pub(crate) struct ScriptExecutionLocation {
    pub(crate) module: Option<u64>,
    pub(crate) position: SourceLocation,
}

impl ScriptImage<'_, '_> {
    pub(crate) fn program(&self) -> &VerifiedKernelModule {
        match self {
            Self::Module { compiled, .. } => compiled.program(),
            Self::Package { compiled, .. } => compiled.program(),
        }
    }

    pub(crate) fn profile(&self) -> &'static str {
        match self {
            Self::Module { .. } => "bound_primitive",
            Self::Package { .. } => "bound_primitive_package",
        }
    }

    pub(crate) fn location(
        &self,
        function: u64,
        instruction: u64,
    ) -> Option<ScriptExecutionLocation> {
        match self {
            Self::Module { source, compiled } => {
                let span = compiled.instruction_span(function, instruction)?;
                Some(ScriptExecutionLocation {
                    module: None,
                    position: source.location(span.start).ok()?,
                })
            }
            Self::Package { sources, compiled } => {
                let location = compiled.instruction_location(function, instruction)?;
                let source = sources.get(usize::try_from(location.module).ok()?)?;
                Some(ScriptExecutionLocation {
                    module: Some(location.module),
                    position: source.location(location.span.start).ok()?,
                })
            }
        }
    }
}
