use std::collections::HashMap;

/// Pagination struct that is getting extracted
/// from the query params
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    /// The index of the first item that has to be returned
    pub offset: i64,
    /// The maximum number of items that have to be returned
    pub limit: Option<i64>,
}

impl Pagination {
    /// Extract query params from the /questions route.
    /// If the query params are not provided we just return the default values.
    /// Default values are `start = 0` and `limit = None`.
    /// If the provided query params are not valid ( cannot be parsed as integers)
    /// we return an error.
    /// # Example query
    /// GET requests to this route can have a pagination attached, so we just
    /// return the questions we need `/questions?start=0&limit=10`
    pub fn extract(params: &HashMap<String, String>) -> Result<Self, PaginationParsingError> {
        // Extract the start and limit from the query params
        // If they are not provided we just return the default values,
        // which are: start = 0 and limit = usize::MAX
        let offset = params.get("offset").map_or(Ok(0), |s| s.parse())?;
        let limit = params
            .get("limit")
            .map(|s| s.parse())
            .map_or(Ok(None), |s| s.map(Some))?;

        Ok(Pagination { offset, limit })
    }
}

/// Error while parsing pagination parameters
///
/// This error is used when the pagination parameters cannot be parsed as integers.
/// It is used in the `Pagination` struct.
#[derive(thiserror::Error, Debug)]
#[error("failed to parse pagination parameters")]
pub struct PaginationParsingError(#[from] std::num::ParseIntError);
