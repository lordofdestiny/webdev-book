//! This module contains types used for resources and other helper types.

pub use answer::{Answer, NewAnswer};
pub use pagination::Pagination;
pub use question::{NewQuestion, Question, QuestionId};

mod answer;
mod pagination;
mod question;
