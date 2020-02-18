//! Capsule for QDEC 

use crate::driver;
use core::cell::Cell;
use kernel::hil; 
use kernel::{AppId, ReturnCode, Driver, Grant};

pub const DRIVER_NUM: usize = driver::NUM::QDEC as usize;

pub struct Qdec<'a> {
    driver: &'a dyn hil::qdec::Qdec<'a>,
    apps: Grant<App>,
}

pub struct App {
    threshold: usize,
}

impl Default for App {
    fn default() -> App {
        App {
            threshold: 0,
        }
    }
}

impl Qdec<'a> {
    pub fn new (driver: &'a dyn hil::qdec::Qdec<'a>, grant: Grant<App>) -> Qdec<'a> {
        Qdec {
            driver: driver,
            apps: grant,
        }
    }
    
    fn enable_qdec (&self) -> ReturnCode {
        self.driver.enable();
        self.driver.is_enabled()
    }
    
    fn get_rotation_changes (&self) -> u32 {
        self.driver.get_acc()
    }
}

impl Driver for Qdec<'a> {
    fn command (&self, command_num: usize, data: usize, data2: usize, appid: AppId) -> ReturnCode {
        match command_num {
            0 => ReturnCode::SUCCESS,
            1 => self.enable_qdec (),
            //2 => self.get_rotation_changes (&self)
            _ => ReturnCode::ENOSUPPORT
        }
    }
}
