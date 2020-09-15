//! Interface for a Qdec compatible chip
//!
//! This trait provides a stanfard interface for chips with a
//! quadrature decoder. Any interface functions that
//! a chip cannot implement can be ignored by the chip capsule

use crate::returncode::ReturnCode;

pub trait QdecDriver {
    /// Sets the client which will receive interrupts
    fn set_client(&self, client: &'static dyn QdecClient);

    /// Enables the SAMPLERDY interrupt
    fn enable_interrupts(&self) -> ReturnCode;

    /// Enables the Qdec, returning error if Qdec does not exist
    fn enable_qdec(&self) -> ReturnCode;

    /// Checks if the qdec has been enabled
    fn enabled(&self) -> ReturnCode;

    /// Reads the accumulator value and resets it
    fn get_acc(&self) -> u32;
}

pub trait QdecClient {
    /// Callback obtaining offset
    fn sample_ready(&self);
    /// Callback dealing with overflows
    fn overflow(&self);
}
