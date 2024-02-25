use std::collections::HashMap;

use crate::error;

/// Pagination struct that is getting extracted
/// from the query params
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    /// The index of the first item that has to be returned
    pub offset: i64,
    /// The maximum number of items that have to be returned
    pub limit: i64,
}

impl Pagination {
    /// Extract query params from the /questions route.
    /// If the query params are not provided we just return the default values.
    /// Default values are `start = 0` and `limit = usize::MAX`.
    /// If the provided query params are not valid ( cannot be parsed as integers)
    /// we return an error.
    /// # Example query
    /// GET requests to this route can have a pagination attached, so we just
    /// return the questions we need `/questions?start=0&limit=10`
    pub fn extract(params: &HashMap<String, String>) -> Result<Self, error::ServiceError> {
        // Extract the start and limit from the query params
        // If they are not provided we just return the default values,
        // which are: start = 0 and limit = usize::MAX
        let offset = params
            .get("offset")
            .map_or(Ok(0), |s| s.parse())
            .map_err(error::ServiceError::PaginationError)?;
        let limit = params
            .get("limit")
            .map_or(Ok(i64::MAX), |s| s.parse())
            .map_err(error::ServiceError::PaginationError)?;

        Ok(Pagination { offset, limit })
    }

    /// Check if the pagination is the default one
    ///
    /// Default pagination is `offset = 0` and `limit = i64::MAX`
    pub fn is_default(&self) -> bool {
        self.offset == 0 && self.limit == i64::MAX
    }
}
