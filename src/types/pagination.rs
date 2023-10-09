use crate::error;
use std::collections::HashMap;

/// Pagination struct that is getting extracted
/// from the query params
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    /// The index of the first item that has to be returned
    pub start: usize,
    /// The maximum number of items that have to be returned
    pub limit: usize,
}

impl Pagination {
    /// Extract query params from the /questions route.
    /// If the query params are not provided we just return the default values.
    /// Default values are `start = 0` and `limit = usize::MAX`.
    /// If the provided query params are not valid ( cannot be parsed as integers)
    /// we return an error.
    /// # Example query
    /// GET requests to this route can have a pagination attached so we just
    /// return the questions we need `/questions?start=0&limit=10`
    pub fn extract(params: HashMap<String, String>) -> Result<Self, error::PaginationError> {
        // Extract the start and limit from the query params
        // If they are not provided we just return the default values
        // Default values are start = 0 and limit = usize::MAX
        let start = params
            .get("start")
            .map(|s| s.parse())
            .unwrap_or(Ok(0))
            .map_err(error::PaginationError)?;
        let limit = params
            .get("limit")
            .map(|s| s.parse())
            .unwrap_or(Ok(usize::MAX))
            .map_err(error::PaginationError)?;
        Ok(Pagination { start, limit })
    }
}
