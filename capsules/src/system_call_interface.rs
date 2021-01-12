//! Reroutes system calls to remote tockOS devices if the particular request cannot be met on this
//! device

use core::cell::Cell;
use crate::driver;
use core::cmp;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::spi;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

// The capsule takes in a driver number, system call number, and up to four 
// arguments and determines whether the system call can be handled locally or
// needs to be sent to a remote device. If it can be handled locally, then the 
// normal system call proceeds and if not, then 

pub struct DetermineRoute<'a> {
  driver_num: usize,
  system_call_num: usize,
}

impl<'a> DetermineRoute<'a> {
  pub fn new(
      driver_num: usize,
      system_call_num: usize
  ) -> DetermineRoute<'a> {
    DetermineRoute {
      driver_num: driver_num,
      system_call_num: system_call_num,
    }
  }

  pub fn determine_route(&self) -> usize {
    let usize route = 1;
    route
  }
}

#[derive(Copy, Clone, PartialEq)]
enum Status {
  Idle,
  Init,
  Sending,
  Receiving,
}

pub struct RemoteSystemCall<'a> {
  spi: &'a dyn spi::SpiMasterDevice,
  pass_buffer: TakeCell<'static, [u8]>,
  write_buffer: TakCell<'static, [u8]>,
  read_buffer: TakeCell<'static, [u8]>,
  status: Cell<Status>,
  driver_num: usize,
  system_call_num: usize,
}

impl<'a> RemoteSystemCall<'a> {
  pub fn new(
      spi: &'a dyn spi::SpiMasterDevice,
      pass_buffer: &'static mut [u8],
      read_buffer: &'static mut [u8],
      driver_num: usize,
      system_call_num: usize
  ) -> RemoteSystemCall<'a> {
    spi.configure(
        spi::ClockPolarity::IdleLow,
        spi::ClockPhase::SampleTrailing,
        4_000_000
    );
    RemoteSystemCall {
      spi: spi,
      pass_buffer: TakeCell::new(pass_buffer),
      write_buffer: TakeCell::empty(),
      read_buffer: TakeCell::new(read_buffer),
      status: Cell::new(Status::Idle),
      driver_num: driver_num,
      system_call_num: system_call_num,
    }
  }
  
  
  pub fn send_data(&self) -> usize {
      if self.status.get() == Status::Idle {
          let error = self.pass_buffer.map_or_else(
              || panic!("There is no spi pass buffer!"),
              |pass_buffer| {
                  self.spi.read_write_bytes(pass_buffer, None, 1);
              },
          );
      } else {
          ReturnCode::EBUSY
      }
  }

  pub fn receive_data() -> ReturnCode {
  }

  fn create_arg_buf (&self, arg_one: usize, arg_two: usize, arg_three: usize, arg_four: usize) {
  
  }
}

