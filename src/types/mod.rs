use std::{
    str::FromStr,
    sync::atomic::{AtomicUsize, Ordering},
};

mod answer;
mod pagination;
mod question;

pub use answer::{Answer, AnswerId};
pub use pagination::Pagination;
pub use question::{Question, QuestionId};

pub trait NextId
where
    Self: FromStr<Err = std::io::Error>,
{
    fn counter() -> &'static AtomicUsize;

    fn next() -> Self {
        let id = Self::counter().fetch_add(1, Ordering::SeqCst);
        Self::from_str(&id.to_string()).unwrap()
    }
}
