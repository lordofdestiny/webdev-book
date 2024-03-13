//! Module for API wrappers for the various API endpoints used by the server
//!
//! The API wrappers are used to interact with the various APIs in a more convenient way, and to
//! provide a consistent interface for the server to use

use serde::{Deserialize, Serialize};

pub mod bad_words;

/// Wrapper for the response from any of the API endpoints, which are wrapped by this module
///
/// # Fields
/// - `message` - String containing the response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIResponse {
    pub message: String,
}
