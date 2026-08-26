#![allow(unused_imports)]
pub use std::result::Result as StdResult;
pub type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<T, DynError>;

pub use macron::*;
// pub use std::{collections::HashMap, path::PathBuf};
