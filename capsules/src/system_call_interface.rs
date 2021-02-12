//! Reroutes system calls to remote tockOS devices if the particular request cannot be met on this
//! device
use core::cell::Cell;
use crate::driver;
use kernel::common::cells::{TakeCell};
use kernel::hil::spi;
use kernel::{debug, ReturnCode};

// The capsule takes in a driver number, system call number, and up to four 
// arguments and determines whether the system call can be handled locally or
// needs to be sent to a remote device. If it can be handled locally, then the 
// normal system call proceeds and if not, then 

/*#[derive(Default)]
pub struct App {
    callback: Option<Callback>,
    subscribed: bool,
}*/


pub struct RemoteSystemCall<'a> {
  spi: &'a dyn spi::SpiMasterDevice,
  pass_buffer: TakeCell<'static, [u8]>,
  read_buffer: TakeCell<'static, [u8]>,
  data_buffer: TakeCell<'static, [u32]>,
  client: TakeCell<'static, bool>,
  //apps: Grant<App>,
}

impl<'a> spi::SpiMasterClient for RemoteSystemCall<'a> {
  fn read_write_done(
      &self,
      mut _write: &'static mut [u8],
      mut _read: Option<&'static mut [u8]>,
      _len: usize,
    ) {
      debug!("Client!");
      self.client.map_or_else(
          || panic!("There is no spi pass buffer!"),
          |client| {
              *client = false;
          },
      );
  }
}

impl<'a> RemoteSystemCall<'a> {
  // Initializes RemoteSystemCall struct
  pub fn new(
      pass_buf: &'static mut [u8],
      read_buf: &'static mut [u8],
      data_buf: &'static mut [u32],
      client: &'static mut bool,
      spi: &'a dyn spi::SpiMasterDevice,
      //apps: Grant<App>
  ) -> RemoteSystemCall<'a> {
      RemoteSystemCall {
          spi: spi,
          pass_buffer: TakeCell::new(pass_buf),
          read_buffer: TakeCell::new(read_buf),
          data_buffer: TakeCell::new(data_buf),
          client: TakeCell::new(client),
          //apps: grant,
      }
  }

  // Configures SPI
  pub fn configure(&self) {
      self.spi.configure(
          spi::ClockPolarity::IdleLow,
          spi::ClockPhase::SampleLeading,
          400_000
      );
  }
 
  // Determines whether or not to reroute system call to be remote
  pub fn determine_route(&self, driver: usize) -> usize {
    // TODO: Need to figure out true metric for determining route //
    let mut route : usize = 0;
    if driver == (driver::NUM::Led as usize) {
        route = 1;
    }
    route
  }

  // Takes the 4 arguments provided to the system call and fills the 
  // data buffer with the information
  pub fn fill_buffer(
      &self,
      system_call_num: usize,
      driver_num: usize,
      arg_one: usize, 
      arg_two: usize, 
      arg_three: usize) {
    self.data_buffer.map_or_else(
        || panic!("There is no spi pass buffer!"),
        |data_buffer| {
            data_buffer[0] = system_call_num as u32;
            data_buffer[1] = driver_num as u32;
            data_buffer[2] = arg_one as u32;
            data_buffer[3] = arg_two as u32;
            data_buffer[4] = arg_three as u32;
        },
    );
  }

  // Helper function to transform the u32 to a u8 array
  fn transform_u32_to_u8_array(&self, y: u32) -> [u8; 4]{
      let b1 = ((y >> 24) & 0xff) as u8;
      let b2 = ((y >> 16) & 0xff) as u8;
      let b3 = ((y >> 8) & 0xff) as u8;
      let b4 = (y & 0xff) as u8;
      [b1, b2, b3, b4]
  }
  
  // Subscribes to callbacks from SPI <-- WIP
  /*pub fn subscribe(&self, callback: Option<Callback>) {
      self.app.enter(|app| {
          app.callback = callback;
      }).unwrap_or_else(|err| err.into());
  }*/
  
  // Sends the data over SPI
  pub fn send_data(&self) -> ReturnCode {
          self.data_buffer.take().map_or_else(
              || panic!("There is no data buffer!"),
              |data_buffer| {
                  self.pass_buffer.map(|pass_buffer| {
                    for i in 0..data_buffer.len() {
                        let temp_arr = self.transform_u32_to_u8_array(data_buffer[i]);
                        for j in 0..4 {
                            pass_buffer[j + 4*i] = temp_arr[j]; 
                        }
                    }
                  });
              }
          );
          self.pass_buffer.take().map_or_else(
              || panic!("There is no spi pass buffer!"),
              |pass_buffer| {
                  
                  self.spi.read_write_bytes(pass_buffer, self.read_buffer.take(), pass_buffer.len());        
                  self.client.map_or_else(
                      || panic!("There is no spi pass buffer!"),
                      |client| {
                          *client = true;
                      },
                      );
              },
          );
          ReturnCode::SUCCESS
  }

  // Receives data (not yet in use)
  pub fn receive_data(&self) {
      self.read_buffer.take().map_or_else(
          || panic!("There is no read buffer!"),
          |read_buffer| {
              for i in 0..read_buffer.len() {
                  debug!("{}", read_buffer[i]);
              }
          }
      );
  }
}
