//! This module contains types used for resources and other helper types.

use std::{
    str::FromStr,
    sync::atomic::{AtomicUsize, Ordering},
};

mod answer;
mod pagination;
mod question;

pub use answer::{Answer, Id};
pub use pagination::Pagination;
pub use question::{Question, QuestionId};

/// `NextId` is a trait that allows us to generate a new id for each resource.
///
/// `NextId` is thread-safe, as it uses an `AtomicUsize` to generate the next id.
///
/// To use `NextId`, you need to implement the `FromStr` trait, which allows us to
/// convert a string to the type of the id.
/// You also need to implement the counter() function, which returns a static
/// `AtomicUsize`, which is used to generate the next id.
pub trait NextId
where
    Self: FromStr<Err = std::io::Error>,
{
    /// counter() returns a static `AtomicUsize`, which is used to generate the next id.
    /// The counter() function is implemented for each resource.
    fn counter() -> &'static AtomicUsize;

    /// next() generates a new id for the resource.
    fn next() -> Self {
        let id = Self::counter().fetch_add(1, Ordering::SeqCst);
        Self::from_str(&id.to_string()).unwrap()
    }
}
