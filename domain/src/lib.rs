pub mod models;
pub mod services;

mod error;

pub use error::Error;
pub type Result<T> = core::result::Result<T, Error>;
