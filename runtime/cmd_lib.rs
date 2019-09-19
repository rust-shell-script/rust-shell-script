use std::io::{Error, ErrorKind};
use std::process;
use std::process::ExitStatus;
use std::str;
use std::fmt::Display;

pub type FunResult = Result<String, std::io::Error>;
pub type CmdResult = Result<(), std::io::Error>;

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        info(format!($($arg)*))
    }
}

#[macro_export]
macro_rules! output {
    ($($arg:tt)*) => {
        output(format!($($arg)*))
    }
}

#[macro_export]
macro_rules! _run_cmd {
    ($cmd:expr) => (run_cmd($cmd, &[]));
    ($cmd:expr, $($arg:expr),+) =>
        (run_cmd($cmd, &[$($arg),+]));
}

#[macro_export]
macro_rules! run_cmd {
    ($($arg:tt)*) => {
        _run_cmd!(format!($($arg)*).as_ref())
    }
}

#[macro_export]
macro_rules! run_fun {
    ($cmd:expr) => (run_fun($cmd, &[]));
    ($cmd:expr, $($arg:expr),+) =>
        (run_fun($cmd, &[$($arg),+]));
}

pub fn info<S>(msg: S) -> CmdResult where S: Into<String> + Display {
    eprintln!("{}", msg);
    Ok(())
}

pub fn output<S>(msg: S) -> FunResult where S: Into<String> {
    Ok(msg.into())
}

pub fn run_cmd(command: &str, args: &[&str]) -> CmdResult {
    info!("Running {} ...", command)?;
    let status = process::Command::new(command)
                                  .args(args)
                                  .status()?;
    if !status.success() {
        Err(to_io_error(command, status))
    } else {
        Ok(())
    }
}

pub fn run_fun(command: &str) -> FunResult {
    let output = process::Command::new(command).output()?;
    if ! output.status.success() {
        Err(to_io_error(command, output.status))
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

fn to_io_error(command: &str, status: ExitStatus) -> Error {
    if let Some(code) = status.code() {
        Error::new(ErrorKind::Other,
                        format!("{} exit with {}", command, code))
    } else {
        Error::new(ErrorKind::Other, "Unknown error")
    }
}