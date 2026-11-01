// SPDX-License-Identifier: AGPL-3.0-or-later
use std::io::{self, Write};

use cod4x_script::execution::KernelValue;
use cod4x_script::legacy::LegacyNumber;

pub(crate) fn kind(value: KernelValue) -> &'static str {
    match value {
        KernelValue::Undefined => "undefined",
        KernelValue::Number(LegacyNumber::Integer(_)) => "integer",
        KernelValue::Number(LegacyNumber::Float(_)) => "float",
        KernelValue::Vector(_) => "vector",
        KernelValue::Function(_) => "function",
    }
}

pub(crate) fn write_stdout(value: KernelValue) -> io::Result<()> {
    let mut output = io::stdout().lock();
    write_result(&mut output, value)
}

fn write_result(output: &mut impl Write, value: KernelValue) -> io::Result<()> {
    match value {
        KernelValue::Undefined => writeln!(output, "undefined")?,
        KernelValue::Number(LegacyNumber::Integer(value)) => writeln!(output, "integer {value}")?,
        KernelValue::Number(LegacyNumber::Float(value)) => writeln!(output, "float {value}")?,
        KernelValue::Vector(value) => {
            let [x, y, z] = value.components();
            writeln!(output, "vector {x} {y} {z}")?;
        }
        KernelValue::Function(_) => writeln!(output, "function")?,
    }
    output.flush()
}

#[cfg(test)]
mod tests {
    use cod4x_script::execution::KernelValue;
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
            match super::write_result(&mut FailingOutput(point), KernelValue::Undefined) {
                Err(error) => assert_eq!(error.kind(), io::ErrorKind::BrokenPipe),
                Ok(()) => return Err("Output failure was swallowed".into()),
            }
        }
        Ok(())
    }
}
