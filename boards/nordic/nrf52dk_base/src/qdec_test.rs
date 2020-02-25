#[allow(unused_imports)]
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::hil::time;
use kernel::hil::time::{Alarm, Frequency};
use kernel::{debug, static_init};
use nrf52::qdec::Qdec;
use kernel::hil::qdec::QdecDriver;
//use nrf5x::pinmux;

pub const TEST_DELAY_MS: u32 = 1000;

pub struct QdecTest<'a, A: time::Alarm<'a>> {
    alarm: &'a A,
    pub qdec: &'a mut Qdec,
}

pub unsafe fn initialize_all(
    mux_alarm: &'static MuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
    qdec: &'static mut Qdec,
) -> &'static QdecTest<
    'static,
    capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
> {
    let qdec_alarm = static_init!(
        VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let qdec_test = static_init!(
        QdecTest<capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>>,
        QdecTest {
            alarm: qdec_alarm,
            qdec: qdec,
        }
    );
    qdec_alarm.set_client(qdec_test);
    qdec_test.qdec.set_client(qdec_test);
    qdec_test
}

impl<'a, A: time::Alarm<'a>> QdecTest<'a, A> {
    pub fn start(&self) {
        self.qdec.enable_qdec();
        debug!("Is enabled?");
        self.qdec.enable_interrupts_qdec();
        debug!("Is interruptable?");
        self.schedule_next();
    }

    fn schedule_next(&self) {
        let delta = (A::Frequency::frequency() * TEST_DELAY_MS) / 1000;
        let next = self.alarm.now().wrapping_add(delta);
        self.alarm.set_alarm(next);
    }
}

impl<'a, A: time::Alarm<'a>> time::AlarmClient for QdecTest<'a, A> {
    fn fired(&self) {
        self.qdec.enable_qdec();
        //self.qdec.enable_interrupts_qdec();
        let acc = self.qdec.get_acc();
        debug!("Is enabled?");
        debug!("Acc: {:?}", acc);
        self.schedule_next();
    }
}

impl<'a, A: time::Alarm<'a>> kernel::hil::qdec::QdecClient for QdecTest<'a, A> {
 fn sample_ready (&self, acc:u32) { //TODO go back and change name
     debug!("Val:{:?}", acc);
 }
}
