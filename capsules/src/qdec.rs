//! There will be lots of comments here eventually...
//!
//!
//!
//! KK will add all those later
//!

use core::cell::Cell;
use kernel::hil::gpio;
use crate::driver;


#[derive(Clone, Copy)]
pub enum Position {
    PositionUp,
    PositionDown,
}

///I want to create some public struct for the QDEC
pub struct QDEC<'a> {
    ///what do I want to put in here?
}

impl<'a> QDEC<'a> {

}

impl<'a> Driver for QDEC<'a> { //! this is for the hil
}

