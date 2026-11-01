// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::kernel_object_error;
use crate::execution::heap::kernel_object_validation;
use crate::execution::{
    KernelErrorKind, KernelHeapContext, KernelScriptRoot, KernelScriptRoots, KernelValue,
};

impl KernelHeapContext<'_> {
    /// Adds fixed, caller-owned script roots without allocation. Supplied bindings
    /// are checked before entry, including roots unused by the executed path.
    /// Root-only graphs survive pressure collection. Missing bindings fail only
    /// when a root instruction executes; the context creates no implicit objects.
    #[must_use]
    pub fn with_script_roots(mut self, roots: KernelScriptRoots) -> Self {
        self.script_roots = Some(roots);
        self
    }

    pub(super) fn validate_script_roots(&mut self) -> Result<(), KernelErrorKind> {
        if let Some(roots) = self.script_roots {
            for id in roots.ordinary_objects() {
                self.charge(1)?;
                kernel_object_validation::reference(self, id)?;
            }
            self.charge(1)?;
            let budget = self.remaining(0)?;
            let result = self
                .objects
                .as_deref()
                .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
                .array_data(roots.game, budget);
            match result {
                Ok(result) => self.record(result.usage, 0)?,
                Err(error) => return Err(kernel_object_error::record(self, error, 0)?),
            }
        }
        Ok(())
    }

    pub(in crate::execution) fn script_root(
        &mut self,
        root: KernelScriptRoot,
        frame: usize,
    ) -> Result<KernelValue, KernelErrorKind> {
        self.charge(1)?;
        let roots = self
            .script_roots
            .ok_or(KernelErrorKind::ScriptRootContextRequired)?;
        let id = match root {
            KernelScriptRoot::CurrentObject => roots.receiver,
            KernelScriptRoot::Level => roots.level,
            KernelScriptRoot::Animation => roots.animation,
            KernelScriptRoot::Game => {
                return self
                    .copy_host_array(frame, roots.game)
                    .map(KernelValue::Array);
            }
        };
        kernel_object_validation::reference(self, id)?;
        Ok(KernelValue::Object(id))
    }
}
