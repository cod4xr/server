// SPDX-License-Identifier: AGPL-3.0-or-later
use std::io::{self, Write};

use cod4x_script::execution::{KernelHeapContext, KernelValue};
use cod4x_script::heap::HeapErrorKind;
use cod4x_script::legacy::LegacyNumber;

pub(crate) fn kind(value: KernelValue) -> &'static str {
    match value {
        KernelValue::Undefined => "undefined",
        KernelValue::Number(LegacyNumber::Integer(_)) => "integer",
        KernelValue::Number(LegacyNumber::Float(_)) => "float",
        KernelValue::Vector(_) => "vector",
        KernelValue::Function(_) => "function",
        KernelValue::String(_) => "string",
    }
}

pub(crate) enum ScriptResult<'a> {
    Undefined,
    Integer(i32),
    Float(f32),
    Vector([f32; 3]),
    Function,
    String(&'a [u8]),
}

impl<'a> ScriptResult<'a> {
    pub(crate) fn resolve(
        value: KernelValue,
        heap: &'a KernelHeapContext<'_>,
    ) -> Result<Self, HeapErrorKind> {
        Ok(match value {
            KernelValue::Undefined => Self::Undefined,
            KernelValue::Number(LegacyNumber::Integer(value)) => Self::Integer(value),
            KernelValue::Number(LegacyNumber::Float(value)) => Self::Float(value),
            KernelValue::Vector(value) => Self::Vector(value.components()),
            KernelValue::Function(_) => Self::Function,
            KernelValue::String(id) => Self::String(heap.get(id)?),
        })
    }

    pub(crate) fn write_stdout(&self) -> io::Result<()> {
        let mut output = io::stdout().lock();
        write_result(&mut output, self)
    }
}

fn write_result(output: &mut impl Write, value: &ScriptResult<'_>) -> io::Result<()> {
    match value {
        ScriptResult::Undefined => writeln!(output, "undefined")?,
        ScriptResult::Integer(value) => writeln!(output, "integer {value}")?,
        ScriptResult::Float(value) => writeln!(output, "float {value}")?,
        ScriptResult::Vector([x, y, z]) => {
            writeln!(output, "vector {x} {y} {z}")?;
        }
        ScriptResult::Function => writeln!(output, "function")?,
        ScriptResult::String(bytes) => writeln!(output, "string b\"{}\"", bytes.escape_ascii())?,
    }
    output.flush()
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};

    #[derive(Clone, Copy)]
    enum FailurePoint {
        Write,
        Flush,
    }

    struct FailingOutput(FailurePoint);

    impl Write for FailingOutput {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            match self.0 {
                FailurePoint::Write => Err(io::ErrorKind::BrokenPipe.into()),
                FailurePoint::Flush => Ok(bytes.len()),
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            match self.0 {
                FailurePoint::Write => Ok(()),
                FailurePoint::Flush => Err(io::ErrorKind::BrokenPipe.into()),
            }
        }
    }

    #[test]
    fn failed_result_writes_and_flushes_return_io_errors() -> Result<(), String> {
        for point in [FailurePoint::Write, FailurePoint::Flush] {
            match super::write_result(&mut FailingOutput(point), &super::ScriptResult::Undefined) {
                Err(error) => assert_eq!(error.kind(), io::ErrorKind::BrokenPipe),
                Ok(()) => return Err("Output failure was swallowed".into()),
            }
        }
        Ok(())
    }

    #[test]
    fn string_result_bytes_escape_quotes_controls_terminators_and_non_utf8() -> Result<(), String> {
        let mut bytes = Vec::new();
        super::write_result(
            &mut bytes,
            &super::ScriptResult::String(b"a\"\\\n\r\t\0\xff"),
        )
        .map_err(|error| error.to_string())?;
        assert_eq!(bytes, b"string b\"a\\\"\\\\\\n\\r\\t\\x00\\xff\"\n");
        Ok(())
    }
}
