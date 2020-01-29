//! Move all development to chips/src
//! Check spec for all registers to be seit

use core::cell::Cell;
use kernel::hil;
use kernel::{AppId, Callback, Driver, ReturnCode};

pub struct QDEC<'a, Q: hil::qdec::QDEC + 'a> {
    //QDEC Driver
    qdec: &'a A,
    channels: &'a [&'a <Q as hil::qdec::QDEC>::Channel],
    pins_init: &'a [(&'a dyn gpio::Pin, ActivationMode)],
    //App State
    callback: Cell<Option<Callback>>, //what is this useful for EXACTLY?????
}

impl<'a, Q: hil::qdec::QDEC> QDEC<'a, Q> {
    pub fn new(
        qdec: &'a A,
        pins_init: &'a [(&'a dyn gpio::Pin, ActivationMode)]
        channels: &'a[&'a <Q as hil::qdec::QDEC>::Channel],
    ) -> QDEC<'a, Q> {
        //FINISH PINS CODE!!
        QDEC {
            //QDEC driver
            qdec: qdec,
            channels: channels,

            //App state
            callback: Cell::new(None),
        }
    }
    fn initialize_pins (&self) -> ReturnCode {
        //initializes selected GPIO pins
    }

    fn rotation(&self) -> ReturnCode {
        //rotation of the rotary encoder
    }

    fn pressed_button(&self) -> ReturnCode {
        //pressing buttons
    }

    fn releasd_button(&self) -> ReturnCode {
        //releasing buttons
    }
    
    fn toggle(&self) -> ReturnCode {
        //toggle buttons
    }
}
