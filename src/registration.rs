use std::fmt;

// error
#[derive(Debug)]
pub enum RegistrationError {
    InvalidCredentials,
}

impl fmt::Display for RegistrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            RegistrationError::InvalidCredentials => write!(f, "Invalid credentials provided"),
        }
    }
}

impl std::error::Error for RegistrationError {}

// trait
pub trait Registration {
    fn register(&self) -> Result<(), RegistrationError>;
    fn deregister(&self) -> Result<(), RegistrationError>;
}

// dummy implementation
pub struct DummyRegistration;

impl Registration for DummyRegistration {
    fn register(&self) -> Result<(), RegistrationError> {
        return Ok(());
    }

    fn deregister(&self) -> Result<(), RegistrationError> {
        return Ok(());
    }
}
