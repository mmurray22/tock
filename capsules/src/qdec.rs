//! Capsule for QDEC 

use crate::driver;
use kernel::hil;
use kernel::debug;
use core::cell::Cell;
use kernel::{AppId, Callback, ReturnCode, Driver, Grant};

pub const DRIVER_NUM: usize = driver::NUM::Qdec as usize;

pub struct QdecInterface<'a> {
    driver: &'a dyn hil::qdec::QdecDriver,
    apps: Grant<App>,
    curr_acc: u32,
}

#[derive(Default)]
pub struct App {
    callback: Option<Callback>,
    subscribed: bool,
}

impl QdecInterface<'a> {
    pub fn new (
        driver: &'a dyn  hil::qdec::QdecDriver, 
        grant: Grant<App>,
    ) -> QdecInterface<'a> {
        QdecInterface {
            driver: driver,
            apps: grant,
            curr_acc: 0,
        }
    }

    fn configure_callback(&self, callback: Option<Callback>, app_id: AppId) -> ReturnCode {
        self.driver.enable_interrupts();
        self.apps
            .enter(app_id, |app, _| {
                app.callback = callback;
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|err| err.into())
    }
}

impl hil::qdec::QdecClient for QdecInterface<'a> {
    fn sample_ready (&self, qdec_val: usize) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| {
                if app.subscribed {
                    self.curr_acc = self.driver.get_acc();
                    app.subscribed = false;
                    app.callback.map(|mut cb| cb.schedule(qdec_val, 0,0));                }
            });
        }
    }
}

impl Driver for QdecInterface<'a> {
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            // subscribe to qdec reading with callback
            0 => self.configure_callback(callback, app_id),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command (&self, command_num: usize, _: usize, _: usize, appid: AppId) -> ReturnCode {
        match command_num {
            //dummy value
            0 => ReturnCode::SUCCESS,
            //enable qdec
            1 => self.driver.enable(),
            //get qdec acc
            2 =>
              ReturnCode::SuccessWithValue {
                value: self.get_acc() as usize,
              },
            //
            /*TODO: any others? 3 =>
              ReturnCode::SuccessWithValue {
                value: 
              }*/
            //default
            _ => ReturnCode::ENOSUPPORT
        }
    }
}
