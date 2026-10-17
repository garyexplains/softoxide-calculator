//! Portable calculator engine. Numerical policies are our own; Apple parity is unverified.
pub mod expression;
pub mod model;
pub mod programmer;
pub mod storage;
pub mod units;

pub use expression::{evaluate, format_number, Angle, CalcError};
pub use model::{Calculator, HistoryEntry, Mode};
