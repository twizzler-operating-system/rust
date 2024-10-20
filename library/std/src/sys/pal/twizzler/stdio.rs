use crate::io::{self, const_io_error, ErrorKind};

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    pub const fn new() -> Stdin {
        Stdin
    }
}

impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let runtime = twizzler_runtime_api::get_runtime();
        runtime.with_stdin(&mut |stdin| stdin.read(buf)).map_err(|e| match e {
            _ => const_io_error!(ErrorKind::Other, "unknown twizzler-runtime read error"),
        })
    }
}

impl Stdout {
    pub const fn new() -> Stdout {
        Stdout
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let runtime = twizzler_runtime_api::get_runtime();
        runtime.with_stdout(&|stdout| stdout.write(buf)).map_err(|e| match e {
            _ => const_io_error!(ErrorKind::Other, "unknown twizzler-runtime write error"),
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        let runtime = twizzler_runtime_api::get_runtime();
        runtime.with_stdout(&|stdout| stdout.flush().map(|_| 0)).map_err(|e| match e {
            _ => const_io_error!(ErrorKind::Other, "unknown twizzler-runtime write error"),
        })?;
        Ok(())
    }
}

impl Stderr {
    pub const fn new() -> Stderr {
        Stderr
    }
}

impl io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let runtime = twizzler_runtime_api::get_runtime();
        runtime
            .with_stderr(&|stderr| stderr.write(buf).and_then(|len| stderr.flush().map(|_| len)))
            .map_err(|e| match e {
                _ => const_io_error!(ErrorKind::Other, "unknown twizzler-runtime write error"),
            })
    }

    fn flush(&mut self) -> io::Result<()> {
        let runtime = twizzler_runtime_api::get_runtime();
        runtime.with_stderr(&|stderr| stderr.flush().map(|_| 0)).map_err(|e| match e {
            _ => const_io_error!(ErrorKind::Other, "unknown twizzler-runtime write error"),
        })?;
        Ok(())
    }
}

pub const STDIN_BUF_SIZE: usize = crate::sys_common::io::DEFAULT_BUF_SIZE;

pub fn is_ebadf(_err: &io::Error) -> bool {
    false
}

pub fn panic_output() -> Option<impl io::Write> {
    Some(Stderr::new())
}
