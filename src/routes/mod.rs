use crate::store::Store;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

pub mod answers;
pub mod questions;

pub fn store_filter() -> BoxedFilter<(Store,)> {
    let store = Store::new();
    warp::any().map(move || store.clone()).boxed()
}
