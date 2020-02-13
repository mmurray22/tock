//!  Qdec driver, nRF5x-family
//!  set_client(), enable, get_ticks,
//!  The nRF5x quadrature decoder
//!
#[allow(unused_imports)] 
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{
    self, register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::{debug};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::hil::gpio::Pin;
use kernel::ReturnCode;
use nrf5x::gpio::GPIOPin;

// In this section I declare a struct called QdecRegisters, which contains all the
// relevant registers as outlined in the Nordic 5x specification of the Qdec.
register_structs! {
    pub QdecRegisters {
        /// Start Qdec sampling
        (0x000 => tasks_start: WriteOnly<u32, Task::Register>),
        /// Stop Qdec sampling
        (0x004 => tasks_stop: WriteOnly<u32, Task::Register>),
        /// Read and clear ACC and ACCDBL
        (0x008 => tasks_readclracc: WriteOnly<u32, Task::Register>),
        /// Read and clear ACC
        (0x00C => tasks_rdclracc: WriteOnly<u32, Task::Register>),
        /// Read nad clear ACCDBL
        (0x010 => tasks_rdclrdbl: WriteOnly<u32, Task::Register>),
        ///Reserve space so tasks_rdclrdbl has access to its entire address space (?)
        (0x0014 => _reserved),
        /// All the events which have interrupts!
        (0x100 => events_arr: [ReadWrite<u32, Event::Register>; 5]),
        /// Event being generated for every new sample
        ///(0x100 => events_samplerdy: Write<u32, Event::Register>),
        /// Non-null report ready
        ///(0x104 => events_reportrdy: Write<u32, Event::Register>),
        /// ACC or ACCDBL register overflow
        ///(0x108 => events_accof: Write<u32, Event::Register>),
        /// Double displacement detected
        ///(0x10C => events_dblrdy: Read<u32, Event::Register>),
        /// events stopped
        ///(0x110 => events_stopped: Read<u32, Event::Register>),
        (0x0114 => _reserved2),
        /// Shortcut register
        (0x200 => shorts: ReadWrite<u32, Shorts::Register>),
        (0x204 => _reserved3),
        /// Enable interrupt
        (0x304 => intenset: ReadWrite<u32, Inte::Register>),
        /// Disable Interrupt
        (0x308 => intenclr: ReadWrite<u32, Inte::Register>),
        (0x30C => _reserved4),
        /// Enable the quad decoder
        (0x500 => enable: ReadWrite<u32, Task::Register>),
        /// Set the LED output pin polarity
        (0x504 => ledpol: WriteOnly<u32, LedPol::Register>),
        /// Sampling-rate register
        (0x508 => sample_per: WriteOnly<u32, SampPer::Register>),
        /// Sample register (receives all samples)
        (0x50C => sample: WriteOnly<u32, Sample::Register>),
        /// Reportper
        (0x510 => report_per: ReadOnly<u32, ReportPer::Register>),
        /// Accumulating motion-sample values register
        (0x514 => acc: ReadOnly<u32, Acc::Register>),
        (0x518 => acc_read: ReadOnly<u32, Acc::Register>),
        (0x51C => reserved6),
        (0x520 => psel_a: ReadWrite<u32, PinSelect::Register>),
        (0x524 => psel_b: ReadWrite<u32, PinSelect::Register>),
        (0x528 => reserved5),
        (0x550 => @END),
    }
}

// In this section, I initialize all the bitfields associated with the type
// of register assigned to each member of the struct above. (is that right?)
register_bitfields![u32,
    Task [
        ENABLE 0
    ],
    Shorts [
        /// Write '1' to Enable shortcut on EVENTS_COMPARE\[0\] event
        REPORTRDY_READCLRACC 0,
        /// Write '1' to Enable shortcut on EVENTS_COMPARE\[1\] event
        SAMPLERDY_STOP 1,
        /// Write '1' to Enable shortcut on EVENTS_COMPARE\[2\] event
        REPORTRDY_RDCLRACC 2,
        /// Write '1' to Enable shortcut on EVENTS_COMPARE\[3\] event
        REPORTRDY_STOP 3,
        /// Write '1' to Enable shortcut on EVENTS_COMPARE\[4\] event
        DBLRDY_RDCLRDBL 4,
        /// Write '1' to Enable shortcut on EVENTS_COMPARE\[5\] event
        DBLRDY_STOP 5,
        /// Write '1' to Enable shortcut on EVENTS_COMPARE\[6\] event
        SAMPLERDY_READCLRACC 6
    ],
    Event [
        READY 0
    ],
    PinSelect [
        Pin OFFSET(0) NUMBITS(5),
        Port OFFSET(5) NUMBITS(1),
        Connect OFFSET(31) NUMBITS(1)
    ],
    Inte [
        /// Write '1' to Enable interrupt on EVENTS_COMPARE\[0\] event
        SAMPLERDY 0,
        /// Write '1' to Enable interrupt on EVENTS_COMPARE\[1\] event
        REPORTRDY 1,
        /// Write '1' to Enable interrupt on EVENTS_COMPARE\[2\] event
        ACCOF 2,
        /// Write '1' to Enable interrupt on EVENTS_COMPARE\[3\] event
        DBLRDY 3,
        /// Write '1' to Enable interrupt on EVENTS_COMPARE\[4\] event
        STOPPED 4
    ],
    LedPol [
        LedPol OFFSET(0) NUMBITS(1) [
            ActiveLow = 0,
            ActiveHigh = 1
        ]
    ],
    Sample [
        SAMPLE 1
    ],
    SampPer [
        SAMPLEPER OFFSET(0) NUMBITS(4) [
            us128 = 0,
            us256 = 1,
            us512 = 2,
            us1024 = 3,
            us2048 = 4,
            us4096 = 5,
            us8192 = 6,
            us16384 = 7,
            ms32 = 8,
            ms65 = 9,
            ms131 = 10
        ]
    ],
    ReportPer [
        REPORTPER OFFSET(0) NUMBITS(4) [
            hz10 = 0,
            hz40 = 1,
            hz80 = 2,
            hz120 = 3,
            hz160 = 4,
            hz200 = 5,
            hz240 = 6,
            hz280 = 7,
            hz1 = 8
        ]
    ],
    Acc [
        ACC OFFSET(0) NUMBITS(32)
    ]
];

/// This defines the beginning of memory which is memory-mapped to the Qdec
/// This base is declared under the Registers Table 3
const QDEC_BASE: StaticRef<QdecRegisters> =
    unsafe { StaticRef::new(0x40012000 as *const QdecRegisters) };

/// Qdec type declaration: gives the Qdec instance registers and a client
pub struct Qdec {
    registers: StaticRef<QdecRegisters>,
}

pub static mut QDEC: Qdec = Qdec {
    registers: QDEC_BASE,
};

/// Qdec impl: provides the Qdec type with vital functionality including:
/// FIRST DESIRED FUNCTIONALITY: new(arg1, arg2, ..., argN) -> define Qdec struct
impl Qdec {
    /*
    pub fn set_client(&self, client: &'static dyn CompareClient) {
        self.client.set(client);
    }
    */

    /// When an interrupt occurs, check to see if any
    /// of the interrupt register bits are set. If it
    /// is, then put it in the client's bitmask
    /*
    pub fn handle_interrupt(&self) {
        self.client.map(|client| {
            let mut val = 0;
            // For each of 4 possible compare events, if it's happened,
            // clear it and sort its bit in val to pass in callback
            for i in 0..4 { // TODO: either add events_compare or add each individual register
                if self.registers.events_arr[i].is_set(Event::READY) {
                    val = val | 1 << i;
                    self.registers.events_arr[i].write(Event::READY::CLEAR);
                    // Disable corresponding interrupt
                    let interrupt_bit = match i {
                        0 => Inte::SAMPLERDY::SET,
                        1 => Inte::REPORTRDY::SET,
                        2 => Inte::ACCOF::SET,
                        3 => Inte::DBLRDY::SET,
                        4 => Inte::STOPPED::SET,
                    };
                    self.registers.intenclr.write(interrupt_bit);
                }
            }
            client.compare(val as u32);
        });
    }
    */

    //add more functions here!
    /*
    fn enable_interrupts(&self) { ///IS THIS THE RIGHT MACRO TO USE?
        let regs = &*self.registers;
        regs.intenset.write(Inte::____::SET /*TODO: Correct macro */);
    }

    fn disable_interrupts(&self) {
        let regs = &*self.registers;
        regs.intenclr.write(/*MACRO*/);
    }

    fn interrupts_enabled(&self) -> bool {
        let regs = &*self.registers;
        self.registers.intenset.is_set(/*MACRO*/);
    }
    */
    pub fn enable(&self) {
        let regs = &*self.registers;
        //TODO: Use `Pinmux` struct here instead of usize to prevent collisions
        regs.psel_a.write(PinSelect::Pin.val(2) + PinSelect::Port.val(0) + 
                          PinSelect::Connect.val(0));
        regs.psel_b.write(PinSelect::Pin.val(29) + PinSelect::Port.val(0) + 
                          PinSelect::Connect.val(0));
        regs.enable.write(Task::ENABLE::SET);
        regs.sample_per.write(SampPer::SAMPLEPER.val(5));
        regs.tasks_start.write(Task::ENABLE::SET);
        debug!("Enabled!");
    }


    pub fn is_enabled(&self) -> bool {
        let regs = &*self.registers;
        regs.enable.is_set(Task::ENABLE)
    }

    pub fn get_acc(&self) -> u32 {
        let regs = &*self.registers;
        regs.tasks_readclracc.write(Task::ENABLE::SET);
        /*TODO: Test code, not sure if this will work*/
        //regs.tasks_readclraccdbl.write(Task::ENABLE::SET);
        /*TODO: End of Test code*/
        let dummy = regs.acc_read.read(Acc::ACC);
        dummy
    }
}
