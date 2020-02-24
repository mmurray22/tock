//! Capsule for QDEC 

use crate::driver;
use kernel::hil; 
use kernel::{AppId, ReturnCode, Driver, Grant};

pub const DRIVER_NUM: usize = driver::NUM::QDEC as usize;

pub struct QdecInterface<'a> {
    driver: &'a dyn hil::qdec::QdecDriver,
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

impl QdecInterface<'a> {
    pub fn new (driver: &'a dyn hil::qdec::QdecDriver, grant: Grant<App>) -> QdecInterface<'a> {
        QdecInterface {
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

impl Driver for QdecInterface<'a> {
    fn command (&self, command_num: usize, _data: usize, _data2: usize, _appid: AppId) -> ReturnCode {
        match command_num {
            0 => ReturnCode::SUCCESS,
            1 => self.enable_qdec (),
            _ => ReturnCode::ENOSUPPORT
        }
    }
}
