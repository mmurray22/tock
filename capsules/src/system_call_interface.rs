//! Reroutes system calls to remote tockOS devices if the particular request cannot be met on this
//! device

use core::cell::Cell;
use core::convert::TryInto;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::spi;
use kernel::{AppId, AppSlice, Callback, debug, Driver, Grant, ReturnCode, Shared};

// The capsule takes in a driver number, system call number, and up to four 
// arguments and determines whether the system call can be handled locally or
// needs to be sent to a remote device. If it can be handled locally, then the 
// normal system call proceeds and if not, then 

pub struct DetermineRoute {
  driver_num: usize,
  system_call_num: usize,
}

impl DetermineRoute {
  pub fn new(
      driver_num: usize,
      system_call_num: usize
  ) -> DetermineRoute {
    DetermineRoute {
      driver_num: driver_num,
      system_call_num: system_call_num,
    }
  }

  pub fn determine_route(&self) -> usize {
    // TODO: Need to figure out metric for determining route //
    let route : usize = 1;
    route
  }

  pub fn create_read_buffer(
      &self, 
      arg_one: usize, 
      arg_two: usize, 
      arg_three: usize, 
      buf: &mut [u8; 5]) {
    debug!("Here 1!");
    buf[0] = self.system_call_num.try_into().unwrap();
    buf[1] = self.driver_num.try_into().unwrap();
    buf[2] = arg_one.try_into().unwrap();
    buf[3] = arg_two.try_into().unwrap();
    buf[4] = arg_three.try_into().unwrap();
  }
}

#[derive(Copy, Clone, PartialEq)]
enum Status {
  Idle,
  Sending,
}

pub struct RemoteSystemCall<'a> {
  spi: &'a dyn spi::SpiMasterDevice,
  pass_buffer: TakeCell<'static, [u8]>,
  write_buffer: TakeCell<'static, [u8]>,
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
  
  pub fn send_data(&self) -> ReturnCode {
      if self.status.get() == Status::Idle {
          self.pass_buffer.take().map_or_else(
              || panic!("There is no spi pass buffer!"),
              |pass_buffer| {
                  self.spi.read_write_bytes(pass_buffer, None, 1);
                  self.status.set(Status::Sending);
              },
          );
          ReturnCode::SUCCESS
      } else {
          ReturnCode::EBUSY
      }
  }

  pub fn receive_data(&self) -> Option<&'static mut [u8]> {
      if self.status.get() == Status::Sending {
        self.read_buffer.take()
      } else {
          None
      }
  }
}
