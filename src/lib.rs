//! Universal convergence controller trait.
//!
//! Every integration in the pleme-io platform implements this trait.
//! The convergence loop is: declare -> simulate -> prove -> render -> deploy -> verify -> reconverge.
//!
//! The trait IS the convergence computing model expressed as Rust types.
//! If an integration doesn't implement ConvergenceController, it's not
//! part of the convergence platform.

pub mod controller;
pub mod proof;
pub mod types;

pub use controller::ConvergenceController;
pub use proof::*;
pub use types::*;
