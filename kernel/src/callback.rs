//! Data structure for storing a callback to userspace or kernelspace.

use core::ptr::NonNull;

use crate::process;
use crate::process::AppId;

/// Type to uniquely identify a callback subscription across all drivers.
///
/// This contains the driver number and the subscribe number within the driver.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CallbackId {
    pub driver_num: usize,
    pub subscribe_num: usize,
}

/// Type for calling a callback in a process.
///
/// This is essentially a wrapper around a function pointer.
#[derive(Clone, Copy)]
pub struct Callback {
    app_id: AppId,
    callback_id: CallbackId,
    appdata: usize,
    fn_ptr: NonNull<*mut ()>,
}

impl Callback {
    crate fn new(
        app_id: AppId,
        callback_id: CallbackId,
        appdata: usize,
        fn_ptr: NonNull<*mut ()>,
    ) -> Callback {
        Callback {
            app_id,
            callback_id,
            appdata,
            fn_ptr,
        }
    }

    /// Actually trigger the callback.
    ///
    /// This will queue the `Callback` for the associated process. It returns
    /// `false` if the queue for the process is full and the callback could not
    /// be scheduled.
    ///
    /// The arguments (`r0-r2`) are the values passed back to the process and
    /// are specific to the individual `Driver` interfaces.
    pub fn schedule(&mut self, r0: usize, r1: usize, r2: usize) -> bool {
        self.app_id
            .kernel
            .process_map_or(false, self.app_id, |process| {
                process.enqueue_task(process::Task::FunctionCall(process::FunctionCall {
                    source: process::FunctionCallSource::Driver(self.callback_id),
                    argument0: r0,
                    argument1: r1,
                    argument2: r2,
                    argument3: self.appdata,
                    pc: self.fn_ptr.as_ptr() as usize,
                }))
            })
    }
}
