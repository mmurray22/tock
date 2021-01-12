//! Reroutes system calls to remote tockOS devices if the particular request cannot be met on this
//! device

use core::cell::Cell;
use crate::driver;
use core::cmp;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

// The capsule takes in a driver number, system call number, and up to four 
// arguments 

pub struct UserSystemCall<'a> {
  driver_num: usize,
  system_call_num: usize,
  arg_one: usize,
  arg_two: usize,
  arg_three: usize,
  arg_four: usize,
}

impl<'a> UserSystemCall<'a> {
  pub fn new(
      driver_num: usize,
      system_call_num: usize
      arg_one: usize,
      arg_two: usize,
      arg_three: usize,
      arg_four: usize,
  ) -> UserSystemCall<'a> {
    UserSystemCall {
      driver_num: driver_num,
      system_call_num: system_call_num,
       arg_one: arg_one,
       arg_two: arg_two,
       arg_three: arg_three,
       arg_four: arg_four,
    }
  }

  pub fn route_system_call (&self) -> ReturnCode {
    //TODO Checks some metrics to determine whether the system call should be remote //
    bool route_away = 1;
    //TODO End of metric checks //
    
    if route_away == 1 {
      send_data(self.driver_num, self.system_call_num, arg_one, arg_two, arg_three);
    }
    match self.system_call_num {
      //Subscribe
      1 = > {
          subscribe(self.driver_num, arg_one, arg_two, arg_three)
      }
      //Command
      2 => {
          command(self.driver_num, arg_one, arg_two, arg_three);
      }
      //Allow
      3 => {
          allow(self.driver_num, arg_one, arg_two, arg_three);
      }
    }
  }
}

