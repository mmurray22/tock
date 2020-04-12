//! Provides userspace access to the Qdec on a board.
//!
//! Usage
//! -----
//!
//! ````
//! let qdec = static_init!(
//!     capsules::qdec::Qdec<'static>,
//!     capsules::qdec::QdecInterface::new(&nrf52::qdec::QDEC,
//!                                         kernel::Grant::create())
//! );
//! kernel::hil::QdecDriver.set_client(qdec);
//! ````
//!
//! #Number of Qdecs
//! The number of qdecs available depends on the
//! board/microcontroller
//!
//! #Interrupt Spurred Readings versus Regular Readings
//! An application can either enable interrupts to get the
//! accumulation value or manually read it whenever it wants

use crate::driver;
use kernel::hil;
use kernel::debug;
use core::cell::Cell;
use kernel::{AppId, Callback, ReturnCode, Driver, Grant};

pub const DRIVER_NUM: usize = driver::NUM::Qdec as usize;

/// This struct contains the resources necessary for the QdecInterface
pub struct QdecInterface<'a> {
    driver: &'a dyn hil::qdec::QdecDriver,
    apps: Grant<App>,
}

#[derive(Default)]
/// This struct contains the necessary fields for an app
pub struct App {
    callback: Option<Callback>,
    subscribed: bool,
}

impl QdecInterface<'a> {
    /// Create a new instance of the QdecInterface
    pub fn new (
        driver: &'a dyn  hil::qdec::QdecDriver,
        grant: Grant<App>,
    ) -> QdecInterface<'a> {
        QdecInterface {
            driver: driver,
            apps: grant,
        }
    }

    /// Configure callback
    fn configure_callback(&self, callback: Option<Callback>, app_id: AppId)
        -> ReturnCode {
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
    /// Goes through all the apps and if the app is
    /// subscribed then it sends back the acc value
    fn sample_ready (&self) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| {
                if app.subscribed {
                    app.subscribed = false;
                    app.callback.map(|mut cb| cb.schedule(self.driver.get_acc(), 0,0));                }
            });
        }
    }
}

impl Driver for QdecInterface<'a> {
    /// Subscribes a client to (newly enabled) interrupts
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => self.configure_callback(callback, app_id),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Command switch statement for various essential processes
    fn command (&self, command_num: usize, _: usize, _: usize, appid: AppId) -> ReturnCode {
        match command_num {
            /// dummy value
            0 => ReturnCode::SUCCESS,
            /// enable qdec
            1 => self.driver.enable(),
            /// get qdec acc
            2 =>
              ReturnCode::SuccessWithValue {
                value: self.get_acc() as usize,
              },
            /// default
            _ => ReturnCode::ENOSUPPORT
        }
    }
}
