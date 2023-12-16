pub struct Sniffer<D, F> {
    pub(crate) inner: D,
    pub(crate) cb: F,
}

impl<D, F> Sniffer<D, F> {
    pub fn new(driver: D, callback: F) -> Self {
        Self {
            inner: driver,
            cb: callback,
        }
    }
}

pub struct SniffedControlPipe<P, F> {
    pub(crate) pipe: P,
    pub(crate) cb: F,
}

impl<P, F> SniffedControlPipe<P, F> {
    pub(crate) fn new(pipe: P, cb: F) -> Self {
        Self { pipe, cb }
    }
}

impl<P, F> embassy_usb::driver::ControlPipe for SniffedControlPipe<P, F>
where
    F: FnMut(embassy_usb::control::Request),
    P: embassy_usb::driver::ControlPipe,
{
    fn max_packet_size(&self) -> usize {
        self.pipe.max_packet_size()
    }

    async fn setup(&mut self) -> [u8; 8] {
        let buf = self.pipe.setup().await;
        let r = embassy_usb::control::Request::parse(&buf);
        (self.cb)(r);
        buf
    }

    async fn data_out(
        &mut self,
        buf: &mut [u8],
        first: bool,
        last: bool,
    ) -> Result<usize, embassy_usb::driver::EndpointError> {
        self.pipe.data_out(buf, first, last).await
    }

    async fn data_in(
        &mut self,
        data: &[u8],
        first: bool,
        last: bool,
    ) -> Result<(), embassy_usb::driver::EndpointError> {
        self.pipe.data_in(data, first, last).await
    }

    async fn accept(&mut self) {
        self.pipe.accept().await
    }

    async fn reject(&mut self) {
        self.pipe.reject().await
    }

    async fn accept_set_address(&mut self, addr: u8) {
        self.pipe.accept_set_address(addr).await
    }
}

impl<'d, D, F> embassy_usb::driver::Driver<'d> for Sniffer<D, F>
where
    D: embassy_usb::driver::Driver<'d>,
    F: FnMut(embassy_usb::control::Request) + 'd,
{
    type EndpointOut = D::EndpointOut;

    type EndpointIn = D::EndpointIn;

    type ControlPipe = SniffedControlPipe<D::ControlPipe, F>;

    type Bus = D::Bus;

    fn alloc_endpoint_out(
        &mut self,
        ep_type: embassy_usb::driver::EndpointType,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointOut, embassy_usb::driver::EndpointAllocError> {
        self.inner
            .alloc_endpoint_out(ep_type, max_packet_size, interval_ms)
    }

    fn alloc_endpoint_in(
        &mut self,
        ep_type: embassy_usb::driver::EndpointType,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointIn, embassy_usb::driver::EndpointAllocError> {
        self.inner
            .alloc_endpoint_in(ep_type, max_packet_size, interval_ms)
    }

    fn start(self, control_max_packet_size: u16) -> (Self::Bus, Self::ControlPipe) {
        let (bus, control_pipe) = self.inner.start(control_max_packet_size);
        (bus, SniffedControlPipe::new(control_pipe, self.cb))
    }
}
