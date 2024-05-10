use std::fmt;
use std::thread;
pub mod console_health_monitor;

#[derive(Debug)]
pub enum OozeError {
    InitializationError,
}

impl fmt::Display for OozeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            OozeError::InitializationError => write!(f, "Initilization failed"),
        }
    }
}

pub trait Ooze {
    fn run(&self) -> Result<thread::JoinHandle<()>, OozeError>;
}
