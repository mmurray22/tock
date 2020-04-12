//! Interface for a Qdec compatible chip
//!
//! This trait provides a stanfard interface for chips a 
//! quadrature decoder. Any interface functions that
//! a chip cannot implement can be ignored by the chip capsule 

use crate::returncode::ReturnCode;

pub trait QdecDriver {
  /// Sets the client which will receive interrupts
  fn set_client(&self, client: &'static dyn QdecClient);  
 
  /// Enables the SAMPLERDY interrupt
  fn enable_interrupts (&self);
  
  /// Enables the Qdec, returning error if Qdec does not exist
  fn enable (&self) -> ReturnCode;

  /// Gets the accumulation value
  fn get_acc (&self) -> u32;
}

pub trait QdecClient {
    /// Callback function
    fn sample_ready (&self);
}
