/// A basic interface for a QDEC compatible chip
///
/// This trait provides a stanfard interface for chips that 
/// contain a quadrature encoder. Any interface functions that
/// a chip cannot implement can be ignored by the chip capsule 
/// and an error will automatically be returned.

use crate::returncode::ReturnCode;

pub trait QdecDriver {
  
  fn enable(&self);

  fn is_enabled (&self) -> ReturnCode;
    
  fn get_acc (&self) -> u32;
}
