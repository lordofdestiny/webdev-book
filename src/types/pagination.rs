use crate::error;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub start: usize,
    pub limit: usize,
}
impl Pagination {
    pub fn extract(params: HashMap<String, String>) -> Result<Self, error::PaginationError> {
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
