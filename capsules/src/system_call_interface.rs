//! Reroutes system calls to remote tockOS devices if the particular request cannot be met on this
//! device
use core::cell::Cell;
use core::convert::TryInto;
use crate::driver;
use kernel::common::cells::{TakeCell};
use kernel::hil::spi;
use kernel::{debug, ReturnCode};

// The capsule takes in a driver number, system call number, and up to four 
// arguments and determines whether the system call can be handled locally or
// needs to be sent to a remote device. If it can be handled locally, then the 
// normal system call proceeds and if not, then 

#[derive(Copy, Clone, PartialEq)]
enum Status {
  Idle,
  Sending,
}

pub struct RemoteSystemCall<'a> {
  spi: &'a dyn spi::SpiMasterDevice,
  pass_buffer: TakeCell<'static, [u8]>,
  read_buffer: TakeCell<'static, [u8]>,
  status: Cell<Status>,
  client: TakeCell<'static, bool>,
}

impl<'a> spi::SpiMasterClient for RemoteSystemCall<'a> {
  fn read_write_done(
      &self,
      mut _write: &'static mut [u8],
      mut _read: Option<&'static mut [u8]>,
      _len: usize,
    ) {
      debug!("Client false!");
      self.client.map_or_else(
          || panic!("There is no spi pass buffer!"),
          |client| {
              *client = false;
          },
      );
  }
}

impl<'a> RemoteSystemCall<'a> {
  pub fn new(
      pass_buf: &'static mut [u8],
      read_buf: &'static mut [u8],
      client: &'static mut bool,
      spi: &'a dyn spi::SpiMasterDevice,
  ) -> RemoteSystemCall<'a> {
      RemoteSystemCall {
          spi: spi,
          pass_buffer: TakeCell::new(pass_buf),
          read_buffer: TakeCell::new(read_buf),
          status: Cell::new(Status::Idle),
          client: TakeCell::new(client),
      }
  }

  pub fn configure(&self) {
      self.spi.configure(
          spi::ClockPolarity::IdleLow,
          spi::ClockPhase::SampleLeading,
          400_000
      );
  }
  
  pub fn determine_route(&self, driver: usize) -> usize {
    // TODO: Need to figure out true metric for determining route //
    let mut route : usize = 0;
    if driver == (driver::NUM::Led as usize) {
        route = 1;
    }
    route
  }

  pub fn fill_buffer(
      &self,
      system_call_num: usize,
      driver_num: usize,
      arg_one: usize, 
      arg_two: usize, 
      arg_three: usize) {
    //debug!("Here 1!");
    self.pass_buffer.map_or_else(
        || panic!("There is no spi pass buffer!"),
        |pass_buffer| {
            //debug!("Here 2!");
            pass_buffer[0] = system_call_num.try_into().unwrap();
            pass_buffer[1] = driver_num.try_into().unwrap();
            pass_buffer[2] = arg_one.try_into().unwrap();
            pass_buffer[3] = arg_two.try_into().unwrap();
            pass_buffer[4] = arg_three.try_into().unwrap();
        },
    );
  }

  pub fn send_data(&self) -> ReturnCode {
      if self.status.get() == Status::Idle {
          self.pass_buffer.take().map_or_else(
              || panic!("There is no spi pass buffer!"),
              |pass_buffer| {
                  self.spi.read_write_bytes(pass_buffer, self.read_buffer.take(), 5);        
                  self.client.map_or_else(
                      || panic!("There is no spi pass buffer!"),
                      |client| {
                          *client = true;
                      },
                      );
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
