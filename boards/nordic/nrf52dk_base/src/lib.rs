//! Shared setup for nrf52dk boards.

#![no_std]
#![allow(dead_code)]
#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, debug_verbose, static_init};

use capsules::virtual_alarm::VirtualMuxAlarm;
use capsules::virtual_spi::MuxSpiMaster;
use capsules::virtual_uart::MuxUart;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::gpio::{Configure, FloatingState};
use nrf52::gpio::Pin;
use nrf52::qdec::Qdec;
use nrf52::rtc::Rtc;
use nrf52::uicr::Regulator0Output;

use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};

pub mod nrf52_components;
use nrf52_components::ble::BLEComponent;
use nrf52_components::ieee802154::Ieee802154Component;

mod qdec_test;

// Constants related to the configuration of the 15.4 network stack
const SRC_MAC: u16 = 0xf00f;
const PAN_ID: u16 = 0xABCD;

/// Pins for SPI for the flash chip MX25R6435F
#[derive(Debug)]
pub struct SpiMX25R6435FPins {
    chip_select: Pin,
    write_protect_pin: Pin,
    hold_pin: Pin,
}

impl SpiMX25R6435FPins {
    pub fn new(chip_select: Pin, write_protect_pin: Pin, hold_pin: Pin) -> Self {
        Self {
            chip_select,
            write_protect_pin,
            hold_pin,
        }
    }
}

/// Pins for the SPI driver
#[derive(Debug)]
pub struct SpiPins {
    mosi: Pin,
    miso: Pin,
    clk: Pin,
}

impl SpiPins {
    pub fn new(mosi: Pin, miso: Pin, clk: Pin) -> Self {
        Self { mosi, miso, clk }
    }
}

/// Pins for the UART
#[derive(Debug)]
pub struct UartPins {
    rts: Pin,
    txd: Pin,
    cts: Pin,
    rxd: Pin,
}

impl UartPins {
    pub fn new(rts: Pin, txd: Pin, cts: Pin, rxd: Pin) -> Self {
        Self { rts, txd, cts, rxd }
    }
}

/// Pins for the UART
#[derive(Debug)]
pub struct QdecPins {
    pin_a: Pin,
    pin_b: Pin,
}

impl QdecPins {
    pub fn new(pin_a: Pin, pin_b: Pin) -> Self {
        Self { pin_a, pin_b }
    }
}

/// Supported drivers by the platform
pub struct Platform {
    ble_radio: &'static capsules::ble_advertising_driver::BLE<
        'static,
        nrf52::ble_radio::Radio,
        VirtualMuxAlarm<'static, Rtc<'static>>,
    >,
    ieee802154_radio: Option<&'static capsules::ieee802154::RadioDriver<'static>>,
    button: &'static capsules::button::Button<'static>,
    console: &'static capsules::console::Console<'static>,
    gpio: &'static capsules::gpio::GPIO<'static>,
    led: &'static capsules::led::LED<'static>,
    rng: &'static capsules::rng::RngDriver<'static>,
    temp: &'static capsules::temperature::TemperatureSensor<'static>,
    ipc: kernel::ipc::IPC,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
    >,
    // The nRF52dk does not have the flash chip on it, so we make this optional.
    nonvolatile_storage:
        Option<&'static capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>>,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            capsules::ieee802154::DRIVER_NUM => match self.ieee802154_radio {
                Some(radio) => f(Some(radio)),
                None => f(None),
            },
            capsules::temperature::DRIVER_NUM => f(Some(self.temp)),
            capsules::nonvolatile_storage_driver::DRIVER_NUM => {
                f(self.nonvolatile_storage.map_or(None, |nv| Some(nv)))
            }
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

/// Generic function for starting an nrf52dk board.
#[inline]
pub unsafe fn setup_board(
    board_kernel: &'static kernel::Kernel,
    button_rst_pin: Pin,
    gpio_port: &'static nrf52::gpio::Port,
    gpio_pins: &'static mut [&'static dyn kernel::hil::gpio::InterruptValuePin],
    debug_pin1_index: Pin,
    debug_pin2_index: Pin,
    debug_pin3_index: Pin,
    led_pins: &'static mut [(
        &'static dyn kernel::hil::gpio::Pin,
        capsules::led::ActivationMode,
    )],
    uart_pins: &UartPins,
    spi_pins: &SpiPins,
    mx25r6435f: &Option<SpiMX25R6435FPins>,
    button_pins: &'static mut [(
        &'static dyn kernel::hil::gpio::InterruptValuePin,
        capsules::button::GpioMode,
    )],
    ieee802154: bool,
    app_memory: &mut [u8],
    process_pointers: &'static mut [Option<&'static dyn kernel::procs::ProcessType>],
    app_fault_response: kernel::procs::FaultResponse,
    reg_vout: Regulator0Output,
    nfc_as_gpios: bool,
    qdec_pins: &QdecPins,
) {
    // Make non-volatile memory writable and activate the reset button
    let uicr = nrf52::uicr::Uicr::new();

    // Check if we need to erase UICR memory to re-program it
    // This only needs to be done when a bit needs to be flipped from 0 to 1.
    let psel0_reset: u32 = uicr.get_psel0_reset_pin().map_or(0, |pin| pin as u32);
    let psel1_reset: u32 = uicr.get_psel1_reset_pin().map_or(0, |pin| pin as u32);
    let mut erase_uicr = ((!psel0_reset & (button_rst_pin as u32))
        | (!psel1_reset & (button_rst_pin as u32))
        | (!(uicr.get_vout() as u32) & (reg_vout as u32)))
        != 0;

    // Only enabling the NFC pin protection requires an erase.
    if nfc_as_gpios {
        erase_uicr |= !uicr.is_nfc_pins_protection_enabled();
    }

    if erase_uicr {
        nrf52::nvmc::NVMC.erase_uicr();
    }

    nrf52::nvmc::NVMC.configure_writeable();
    while !nrf52::nvmc::NVMC.is_ready() {}

    let mut needs_soft_reset: bool = false;

    // Configure reset pins
    if uicr
        .get_psel0_reset_pin()
        .map_or(true, |pin| pin != button_rst_pin)
    {
        uicr.set_psel0_reset_pin(button_rst_pin);
        while !nrf52::nvmc::NVMC.is_ready() {}
        needs_soft_reset = true;
    }
    if uicr
        .get_psel1_reset_pin()
        .map_or(true, |pin| pin != button_rst_pin)
    {
        uicr.set_psel1_reset_pin(button_rst_pin);
        while !nrf52::nvmc::NVMC.is_ready() {}
        needs_soft_reset = true;
    }

    // Configure voltage regulator output
    if uicr.get_vout() != reg_vout {
        uicr.set_vout(reg_vout);
        while !nrf52::nvmc::NVMC.is_ready() {}
        needs_soft_reset = true;
    }

    // Check if we need to free the NFC pins for GPIO
    if nfc_as_gpios {
        uicr.set_nfc_pins_protection(true);
        while !nrf52::nvmc::NVMC.is_ready() {}
        needs_soft_reset = true;
    }

    // Any modification of UICR needs a soft reset for the changes to be taken into account.
    if needs_soft_reset {
        cortexm4::scb::reset();
    }

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&gpio_port[debug_pin1_index]),
        Some(&gpio_port[debug_pin2_index]),
        Some(&gpio_port[debug_pin3_index]),
    );

    let gpio = static_init!(
        capsules::gpio::GPIO<'static>,
        capsules::gpio::GPIO::new(
            gpio_pins,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );

    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    // LEDs
    let led = static_init!(
        capsules::led::LED<'static>,
        capsules::led::LED::new(led_pins)
    );

    // Buttons
    let button = static_init!(
        capsules::button::Button<'static>,
        capsules::button::Button::new(
            button_pins,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );

    for (pin, _) in button_pins.iter() {
        pin.set_client(button);
    }

    let rtc = &nrf52::rtc::RTC;
    rtc.start();
    let mux_alarm = static_init!(
        capsules::virtual_alarm::MuxAlarm<'static, nrf52::rtc::Rtc>,
        capsules::virtual_alarm::MuxAlarm::new(&nrf52::rtc::RTC)
    );
    hil::time::Alarm::set_client(rtc, mux_alarm);

    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
        .finalize(components::alarm_component_helper!(nrf52::rtc::Rtc));

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = static_init!(
        MuxUart<'static>,
        MuxUart::new(
            &nrf52::uart::UARTE0,
            &mut capsules::virtual_uart::RX_BUF,
            115200
        )
    );
    uart_mux.initialize();
    hil::uart::Transmit::set_transmit_client(&nrf52::uart::UARTE0, uart_mux);
    hil::uart::Receive::set_receive_client(&nrf52::uart::UARTE0, uart_mux);

    nrf52::uart::UARTE0.initialize(
        nrf52::pinmux::Pinmux::new(uart_pins.txd as u32),
        nrf52::pinmux::Pinmux::new(uart_pins.rxd as u32),
        Some(nrf52::pinmux::Pinmux::new(uart_pins.cts as u32)),
        Some(nrf52::pinmux::Pinmux::new(uart_pins.rts as u32)),
    );

    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    let ble_radio =
        BLEComponent::new(board_kernel, &nrf52::ble_radio::RADIO, mux_alarm).finalize(());

    let ieee802154_radio = if ieee802154 {
        let (radio, _) = Ieee802154Component::new(
            board_kernel,
            &nrf52::ieee802154_radio::RADIO,
            PAN_ID,
            SRC_MAC,
        )
        .finalize(());
        Some(radio)
    } else {
        None
    };

    let temp = static_init!(
        capsules::temperature::TemperatureSensor<'static>,
        capsules::temperature::TemperatureSensor::new(
            &nrf52::temperature::TEMP,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    kernel::hil::sensors::TemperatureDriver::set_client(&nrf52::temperature::TEMP, temp);

    let rng = components::rng::RngComponent::new(board_kernel, &nrf52::trng::TRNG).finalize(());

    // SPI
    let mux_spi = static_init!(
        MuxSpiMaster<'static, nrf52::spi::SPIM>,
        MuxSpiMaster::new(&nrf52::spi::SPIM0)
    );
    hil::spi::SpiMaster::set_client(&nrf52::spi::SPIM0, mux_spi);
    hil::spi::SpiMaster::init(&nrf52::spi::SPIM0);
    nrf52::spi::SPIM0.configure(
        nrf52::pinmux::Pinmux::new(spi_pins.mosi as u32),
        nrf52::pinmux::Pinmux::new(spi_pins.miso as u32),
        nrf52::pinmux::Pinmux::new(spi_pins.clk as u32),
    );

    let nonvolatile_storage: Option<
        &'static capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>,
    > = if let Some(driver) = mx25r6435f {
        // Create a SPI device for the mx25r6435f flash chip.
        let mx25r6435f_spi = static_init!(
            capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>,
            capsules::virtual_spi::VirtualSpiMasterDevice::new(
                mux_spi,
                &gpio_port[driver.chip_select]
            )
        );
        // Create an alarm for this chip.
        let mx25r6435f_virtual_alarm = static_init!(
            VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
            VirtualMuxAlarm::new(mux_alarm)
        );
        // Setup the actual MX25R6435F driver.
        let mx25r6435f = static_init!(
            capsules::mx25r6435f::MX25R6435F<
                'static,
                capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>,
                nrf52::gpio::GPIOPin,
                VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
            >,
            capsules::mx25r6435f::MX25R6435F::new(
                mx25r6435f_spi,
                mx25r6435f_virtual_alarm,
                &mut capsules::mx25r6435f::TXBUFFER,
                &mut capsules::mx25r6435f::RXBUFFER,
                Some(&gpio_port[driver.write_protect_pin]),
                Some(&gpio_port[driver.hold_pin])
            )
        );
        mx25r6435f_spi.set_client(mx25r6435f);
        hil::time::Alarm::set_client(mx25r6435f_virtual_alarm, mx25r6435f);

        pub static mut FLASH_PAGEBUFFER: capsules::mx25r6435f::Mx25r6435fSector =
            capsules::mx25r6435f::Mx25r6435fSector::new();
        let nv_to_page = static_init!(
            capsules::nonvolatile_to_pages::NonvolatileToPages<
                'static,
                capsules::mx25r6435f::MX25R6435F<
                    'static,
                    capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>,
                    nrf52::gpio::GPIOPin,
                    VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
                >,
            >,
            capsules::nonvolatile_to_pages::NonvolatileToPages::new(
                mx25r6435f,
                &mut FLASH_PAGEBUFFER
            )
        );
        hil::flash::HasClient::set_client(mx25r6435f, nv_to_page);

        let nonvolatile_storage = static_init!(
            capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>,
            capsules::nonvolatile_storage_driver::NonvolatileStorage::new(
                nv_to_page,
                board_kernel.create_grant(&memory_allocation_capability),
                0x60000, // Start address for userspace accessible region
                0x20000, // Length of userspace accessible region
                0,       // Start address of kernel accessible region
                0x60000, // Length of kernel accessible region
                &mut capsules::nonvolatile_storage_driver::BUFFER
            )
        );
        hil::nonvolatile_storage::NonvolatileStorage::set_client(nv_to_page, nonvolatile_storage);
        Some(nonvolatile_storage)
    } else {
        None
    };

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    nrf52::clock::CLOCK.low_stop();
    nrf52::clock::CLOCK.high_stop();

    nrf52::clock::CLOCK.low_set_source(nrf52::clock::LowClockSource::XTAL);
    nrf52::clock::CLOCK.low_start();
    nrf52::clock::CLOCK.high_set_source(nrf52::clock::HighClockSource::XTAL);
    nrf52::clock::CLOCK.high_start();
    while !nrf52::clock::CLOCK.low_started() {}
    while !nrf52::clock::CLOCK.high_started() {}

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 1], Default::default());
    let dynamic_deferred_call = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_call);

    //We think this is needed bc the MBEDOS driver does that
    gpio_port[qdec_pins.pin_a].make_input();
    gpio_port[qdec_pins.pin_b].make_input();
    gpio_port[qdec_pins.pin_a].set_floating_state(FloatingState::PullUp);
    gpio_port[qdec_pins.pin_b].set_floating_state(FloatingState::PullUp);

    // TODO: Use pinmux
    debug!("Initializing QDEC test!");
    let qdec = static_init!(
        Qdec,
        Qdec::new(
            nrf52::pinmux::Pinmux::new(qdec_pins.pin_a as u32),
            nrf52::pinmux::Pinmux::new(qdec_pins.pin_b as u32),
        )
    );
    let qdec_test = qdec_test::initialize_all(mux_alarm, qdec);
    debug!("Testing: Qdec Initialized!");
    let platform = Platform {
        button: button,
        ble_radio: ble_radio,
        ieee802154_radio: ieee802154_radio,
        console: console,
        led: led,
        gpio: gpio,
        rng: rng,
        temp: temp,
        alarm: alarm,
        nonvolatile_storage: nonvolatile_storage,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
    };

    let chip = static_init!(nrf52::chip::NRF52, nrf52::chip::NRF52::new(gpio_port));

    debug!("Initialization complete. Entering main loop\r");
    debug!("{}", &nrf52::ficr::FICR_INSTANCE);
    qdec_test.start();
    debug!("Started QDEC");

    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }
    kernel::procs::load_processes(
        board_kernel,
        chip,
        &_sapps as *const u8,
        app_memory,
        process_pointers,
        app_fault_response,
        &process_management_capability,
    );

    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
