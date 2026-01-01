//! RLP Client Library
//! 
//! This library provides generated types and instruction builders for the RLP program.

pub mod generated;

// Re-export commonly used items for convenience
pub use generated::*;
pub use generated::instructions::*;
pub use generated::accounts::*;
pub use generated::types::*;
pub use generated::errors::*;
pub use generated::programs::RLP_ID;

