/// A basic interface for a QDEC compatible chip
///
/// This trait provides a stanfard interface for chips that 
/// contain a quadrature encoder. Any interface functions that
/// a chip cannot implement can be ignored by the chip capsule 
/// and an error will automatically be returned.

use crate::returncode::ReturnCode;

pub trait QdecDriver {
  fn set_client(&self, client: &'static dyn QdecClient);  
 
  fn enable_interrupts (&self);
  
  fn enable (&self) -> ReturnCode;

  fn get_acc (&self) -> u32;
}

pub trait QdecClient {
    fn sample_ready (&self);
}
