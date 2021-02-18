//! Reroutes system calls to remote tockOS devices if the particular request 
//! cannot be met on this device
//!
//! The capsule struct Remote SystemCall is comprised of a virtual spi 
//! controller device, a write buffer, a read buffer, a data buffer (which takes
//! in the raw argument data from the system call), a client boolean to indicate
//! whether the read/write has finished, and a GPIO pin. It depends on the
//! platform specific remote_syscall function to reroute the data via SPI to a
//! peripheral device.
//!
//! Usage
//! ------
//! Create of a remote_system_call object:
//!
//!    let remote_mux_spi = components::spi::SpiMuxComponent::new(&sam4l::spi::SPI)
//!        .finalize(components::spi_mux_component_helper!(sam4l::spi::SpiHw));
//!    let remote_spi = SpiComponent::new(remote_mux_spi, 2)
//!        .finalize(components::spi_component_helper!(sam4l::spi::SpiHw));
//!    let remote_pin = &sam4l::gpio::PC[31];
//!    let remote_system_call = static_init!(capsules::system_call_interface::RemoteSystemCall<'static>,
//!                                          RemoteSystemCall::new(&mut BUF,
//!                                                                &mut BUF_CLI,
//!                                                                &mut DATA,
//!                                                                &mut CLIENT,
//!                                                                remote_spi,
//!                                                                remote_pin));
//!    remote_spi.set_client(remote_system_call);
//!    remote_pin.set_client(remote_system_call);
//!    remote_system_call.configure();
//!
//!
//! Create a logical path for a system call:
//!
//! Within the platform struct's remote_syscall function, there is a  match 
//! statement. The match statement contains all currently supported system calls
//! If we wanted to add a system call to support, we do the following (taking 
//! command as an example):
//!
//! syscall::Syscall::COMMAND {
//!             driver_number,
//!             subdriver_number,
//!             arg0,
//!             arg1,
//!         } => {
//!             if self.remote_system_call.determine_route(*driver_number) == 0 {
//!               return Ok(());
//!             }
//!             self.remote_system_call.fill_buffer(2,
//!                                                 *driver_number,
//!                                                 *subdriver_number,
//!                                                 *arg0,
//!                                                 *arg1);
//!             self.remote_system_call.send_data();
//!             core::prelude::v1::Err(ReturnCode::FAIL)
//! },
//!
//! Note that we return an error once we have finished sending off the data.
//! This is due to the way the remote_syscall function is called in the
//! scheduler. If the remote_syscall function returns an error, that means that
//! the system call was rerouted to be executed remotely and thus does not need
//! a corresponding local execution as well. 
//!
//! NOTE: Once you add this system call support to the controller, you must also
//! ensure the peripheral app in libtock-c has the appropriate support for the 
//! system call as well.

use crate::driver;
use kernel::common::cells::{TakeCell};
use kernel::hil::{spi, gpio};
use kernel::{debug, ReturnCode};

pub struct RemoteSystemCall<'a> {
  spi: &'a dyn spi::SpiMasterDevice,
  pass_buffer: TakeCell<'static, [u8]>, //write_buffer
  read_buffer: TakeCell<'static, [u8]>,
  data_buffer: TakeCell<'static, [u32]>,
  client: TakeCell<'static, bool>,
  pin: &'a dyn gpio::InterruptPin<'a>,
}

impl<'a> spi::SpiMasterClient for RemoteSystemCall<'a> {
  //Executed once the SPI data transfer is complete
  fn read_write_done(
      &self,
      mut write: &'static mut [u8],
      mut read: Option<&'static mut [u8]>,
      _len: usize,
    ) {
      debug!("Client!");
      self.client.map_or_else(
          || panic!("There is no spi pass buffer!"),
          |client| {
              *client = false;
          },
      );
      let rbuf = read.take().unwrap();
      debug!("Length read: {}", rbuf.len());
      for i in 0..rbuf.len() {
          debug!("{}", rbuf[i]);
      }
      self.pass_buffer.replace(write);
      self.read_buffer.replace(rbuf);
  }
}

impl<'a> gpio::Client for RemoteSystemCall<'a> {
    //Fires when toggled
    fn fired(&self) {
        debug!("Hey! The GPIO Pin fired!");
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
      syscall_pin: &'a dyn gpio::InterruptPin<'a>,
  ) -> RemoteSystemCall<'a> {
      RemoteSystemCall {
          spi: spi,
          pass_buffer: TakeCell::new(pass_buf),
          read_buffer: TakeCell::new(read_buf),
          data_buffer: TakeCell::new(data_buf),
          client: TakeCell::new(client),
          pin: syscall_pin, 
      }
  }

  // Configures different hardware associated with the capsule
  pub fn configure(&self) {
      /*Configure SPI*/
      self.spi.configure(
          spi::ClockPolarity::IdleLow,
          spi::ClockPhase::SampleLeading,
          400_000
      );

      /*Configure GPIO*/
      self.pin.make_input();
      self.pin.clear();
      self.pin.set_floating_state(gpio::FloatingState::PullNone);
      self.pin.enable_interrupts(gpio::InterruptEdge::RisingEdge);

  }
 
  // Determines whether or not to reroute system call to be remote
  pub fn determine_route(&self, driver: usize) -> usize {
    // TODO: Need to implement true metric for determining route //
    let mut route : usize = 0;
    if driver == (driver::NUM::Led as usize) ||
       driver == (driver::NUM::Rng as usize) {
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
        || panic!("There is no data buffer!"),
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
  
  // Sends the data over SPI
  pub fn send_data(&self) -> ReturnCode {
      self.data_buffer.map_or_else(
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
              let rbuf = self.read_buffer.take().unwrap();
              self.spi.read_write_bytes(pass_buffer, Some(rbuf), pass_buffer.len());
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
}
