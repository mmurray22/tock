#![no_std]

pub use nrf52::{
    acomp, adc, aes, ble_radio, clock, constants, crt1, ficr, i2c, ieee802154_radio, init, nvmc,
    pinmux, ppi, pwm, rtc, spi, temperature, timer, trng, uart, uicr, usbd,
};
pub mod chip;
pub mod gpio;
pub mod interrupt_service;

mod peripheral_interrupts;
