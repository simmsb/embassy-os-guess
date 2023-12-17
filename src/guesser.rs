use crate::sniffer::Sniffer;

pub struct OSGuesser<F> {
    detected_cb: Option<F>,
    count: u8,
    num_02: u8,
    num_04: u8,
    num_ff: u8,
    last_wlength: u16,
}

impl<F> OSGuesser<F>
where
    F: FnOnce(OS),
{
    pub fn new(callback: F) -> Self {
        Self {
            detected_cb: Some(callback),
            count: 0,
            num_02: 0,
            num_04: 0,
            num_ff: 0,
            last_wlength: 0,
        }
    }

    pub fn wrap_driver<'d, D>(mut self, driver: D) -> impl embassy_usb::driver::Driver<'d>
    where
        D: embassy_usb::driver::Driver<'d>,
        F: 'd,
    {
        Sniffer::new(driver, move |r: embassy_usb::control::Request| {
            if r.request_type == embassy_usb::control::RequestType::Standard
                && r.recipient == embassy_usb::control::Recipient::Device
                && r.request == embassy_usb::control::Request::GET_DESCRIPTOR
                && r.descriptor_type_index().0 == embassy_usb::descriptor::descriptor_type::STRING
            {
                self.handle(r.length);
            }
        })
    }

    fn dispatch(&mut self, guess: OS) {
        if let Some(cb) = self.detected_cb.take() {
            info!("Guessed OS: {:?}", guess);

            cb(guess);
        }
    }

    fn handle(&mut self, wlength: u16) {
        if self.detected_cb.is_none() {
            return;
        }

        info!(
            "Guessing OS with wlength: {}. Current state: (count: {}, 02: {}, 04: {}, ff: {})",
            wlength, self.count, self.num_02, self.num_04, self.num_ff
        );

        self.count += 1;

        match wlength {
            0x2 => self.num_02 += 1,
            0x4 => self.num_04 += 1,
            0xff => self.num_ff += 1,
            _ => (),
        };

        self.last_wlength = wlength;

        if self.count < 3 {
            return;
        }

        if self.num_ff >= 2 && self.num_04 >= 1 {
            self.dispatch(OS::Windows);
            return;
        }

        if self.count == self.num_ff {
            self.dispatch(OS::Linux);
            return;
        }

        if self.count == 4 && self.num_ff == 0 && self.num_02 == 2 {
            self.dispatch(OS::MacOS);
            return;
        }

        if self.count == 5 && self.last_wlength == 0xff && self.num_ff == 1 && self.num_02 == 2 {
            self.dispatch(OS::MacOS);
            return;
        }

        if self.count > 10 {
            self.detected_cb = None;
            return;
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OS {
    Windows,
    Linux,
    MacOS,
    Unknown,
}
