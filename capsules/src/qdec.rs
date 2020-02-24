//! Capsule for QDEC 

use crate::driver;
use kernel::hil;
use core::cell::Cell;
use kernel::{AppId, Callback, ReturnCode, Driver, Grant};

pub const DRIVER_NUM: usize = driver::NUM::QDEC as usize;

pub struct QdecInterface<'a> {
    driver: &'a dyn hil::qdec::QdecDriver,
    apps: Grant<App>,
    busy: Cell<bool>,
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
            busy: Cell::new(false),
        }
    }
    
    fn enqueue_command(&self, appid: AppId) -> ReturnCode {
        self.apps
            .enter(appid, |app, _| {
                if !self.busy.get() {
                    app.subscribed = true;
                    self.busy.set(true);
                    self.driver.enable_qdec()
                } else {
                    ReturnCode::EBUSY
                }
            })
            .unwrap_or_else(|err| err.into())
    }

    fn configure_callback(&self, callback: Option<Callback>, app_id: AppId) -> ReturnCode {
        self.apps
            .enter(app_id, |app, _| {
                app.callback = callback;
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|err| err.into())
    }
}

impl hil::qdec::QdecClient for QdecInterface<'a> {
    fn callback(&self, qdec_val: usize) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| {
                if app.subscribed {
                    self.busy.set(false);
                    app.subscribed = false;
                    app.callback.map(|mut cb| cb.schedule(qdec_val, 0,0));                }
            });
        }
    }

    fn compare (&self, val: u32) -> bool {
        val > 0
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
            0 => ReturnCode::SUCCESS,
            1 => self.enqueue_command (appid),
            //2 => self.get_acc(), 
            _ => ReturnCode::ENOSUPPORT
        }
    }
}
