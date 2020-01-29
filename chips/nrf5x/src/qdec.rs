//!  QDEC driver, nRF5x-family
//!  set_client(), enable, get_ticks,
//!  The nRF5x quadrature decoder 
//!


use kernel::common::cells::OptionalCell;
use kernel::common::registers::{self, register_bitfields, ReadOnly, ReadWrite, ReadOnly};
use kernel::common::StaticRef;
use kernel::hil;


//! In this section I declare a struct called QDECRegisters, which contains all the 
//! relevant registers as outlined in the Nordic 5x specification of the QDEC.
//! TODO: add in missing registers; TODO: add in register reserves
register_structs! {
    struct QDEC {
        /// Start QDEC sampling
        (0x000 => tasks_start: WriteOnly<u32, Task::Register>),
        /// Stop QDEC sampling
        (0x004 => tasks_stop: WriteOnly<u32, Task::Register>),
        /// Read and clear ACC and ACCDBL
        (0x008 => tasks_readclracc: ReadWrite<u32, Task::Register>),
        /// Read and clear ACC
        (0x00C => tasks_rdclracc: ReadWrite<u32, Task::Register>),
        /// Read nad clear ACCDBL
        (0x010 => tasks_rdclrdbl: ReadWrite<u32, Task::Register>),
        ///Reserve space so tasks_rdclrdbl has access to its entire address space (?)
        (0x0012 => _reserved),
        (0x0014 => word: ReadWrite<u32>),
        /// All the events which have interrupts!
        (0x100 => events_arr: ReadWrite<u32, Event::Register>),
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
        ///Reserve space so events_stopped has access to its entire address space (?)
        (0x0102 => _reserved2),
        (0x0104 => word: Read<u32>),
        /// Shortcut register
        (0x200 => shorts: Read<u32, Shorts::Register>),
        ///Reserve space so shorts has access to its entire address space (?)
        (0x204 => _reserved3),
        (0x208 => word: ReadWrite<u32>),
        /// Enable interrupt
        (0x304 => intenset: ReadWrite<u32, Inte::Register>),
        /// Disable Interrupt
        (0x308 => intenclr: ReadWrite<u32, Inte::Register>),
        /// Enable the quad decoder
        (0x500 => enable: ReadWrite<u32, Ena::Register>),
        /// <----- MISSING A REGISTER ----> ////
        /// Sampling-rate register
        (0x508 => sample_per: WriteOnly<u32, SampPer::Register>),
        /// Sample register (receives all samples)
        (0x50C => sample: WriteOnly<u32, Sample::Register>),
        /// Reportper
        (0x510 => report_per: ReadOnly<u32, Report::Register>),
        /// Accumulating motion-sample values register
        (0x514 => acc: ReadOnly<u32, Acc::Register>),
        /// Reserve space for the rest of the registers ? 
        (0x0102 => _reserved4),
        (0x0104 => word: Read<u32>),
        /// <------MISSING MORE REGISTERS ----> ////
        (0x550 => @END),
    }
}


//! In this section, I initialize all the bitfields associated with the type
//! of register assigned to each member of the struct above. (is that right?)
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
        SAMPLERDY_READCLRACC 6,
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
        STOPPED 4,
        /// Write '1' to Enable interrupt on EVENTS_COMPARE\[5\] event
        ///COMPARE5 21
    ],
    Ena [
        /// idk...
        ENABLE 1
    ],
    Sample [
        SAMPLE 1
    ],
    SampPer [
        SAMPLEPER 1
    ],
    Report [
        REPORTPER 1
    ],
    Acc [
        ACC 1
    ],
];

//! This defines the beginning of memory which is memory-mapped to the QDEC
//! This base is declared under the Registers Table 3
const QDEC_BASE: StaticRef<QdecRegisters> = unsafe { StaticRef::new(0x40012000 as *const QdecRegisters) };

/// The client referenced here is the capsule code which will be built on top of this
pub trait CompareClient {
    fn compare(&self);
}

/// QDEC type declaration: gives the QDEC instance registers and a client 
pub struct QDEC<'a> {
    registers: StaticRef<QDECRegisters>,
    client: OptionalCell<&'static dyn CompareClient>,
}

pub static mut QDEC: QDEC = QDEC {
        registers: QDEC_BASE,
        client: OptionalCell<&'static dyn CompareClient>,
};

/// QDEC impl: provides the QDEC type with vital functionality including:
/// FIRST DESIRED FUNCTIONALITY: new(arg1, arg2, ..., argN) -> define QDEC struct 
/// TODO: Set up the mess that is functionality of QDEC
impl QDEC<'a> {
    pub const fn new(registers: StaticRef<QDECRegisters>, sample: usize) -> QDEC {
        QDEC {
            registers: QDEC_BASE,
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client(&self, client: &'static dyn CompareClient) {
        self.client.set(client);
    }

    /// When an interrupt occurs, check to see if any
    /// of the interrupt register bits are set. If it
    /// is, then put it in the client's bitmask
     
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

    //add more functions here!
    fn enable_interrupts(&self) { ///IS THIS THE RIGHT MACRO TO USE?
        let regs = &*self.registers;
        regs.intenset.write(Inte::____::SET /*TODO: Correct macro */);
    }

    fn disable_interrupts(&self) {
        let regs = &*self.registers;
        regs.intenclr.write(/*MACRO*/);
    }

    fn interrupts_enable(&self) -> bool {
        let regs = &*self.registers;
        self.registers.intenset.is_set(/*MACRO*/);
    }

    pub fn enable(&self) -> ReturnCode {
        let regs = &*self.registers;
        self.registers.enable.write(/*MACRO*/);
    }

    fn is_enabled(&self) {
        let regs = &*self.registers;
        self.registers.enable.is_set(/*MACRO*/);
    }

    pub fn get_ticks(&self) -> Result<u32, ReturnCode> {
        let regs = &*self.registers;
        self.registers.accumulate.read(/*MACRO*/);
    }   
}
