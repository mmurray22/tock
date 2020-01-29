//! This is a new hardware interface for the QDEC!
//! Generic QDEC for any board
use crate::hil::gpio;

pub trait QDEC {
  //!
  fn set_client();
  fn enable();
  fn get_ticks(&self) -> Result<u32, ReturnCode>; 
  fn enable(&self) -> ReturnCode; //! success or error
 //! ACC register, enable debounce?, prob 0 for sample rate
  fn rotation(&mut self);
  //!USE BUTTON HIL FOR THIS
  //!fn pressed_button(&mut self);
  //!fn released_button(&mut self);
  fn initialize_pins (&mut self);
}

//! In kernel test in the boards folder
//! create a new qdec, config, and then call fxns avail
//! Need to set a timer 
//! Samples imix -> src -> udp_lowpan_test
//! Where to proceed from here?
