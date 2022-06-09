mod compositor;
mod xdg_shell;

use crate::Hutch;

use smithay::{
    delegate_seat,
    delegate_output,
    delegate_data_device,
    wayland::{
        seat::{SeatHandler, SeatState},
        data_device::{
            ClientDndGrabHandler,
            DataDeviceHandler,
            ServerDndGrabHandler
        },
    }
};

impl SeatHandler for Hutch {
    fn seat_state(&mut self) -> &mut SeatState<Hutch> {
        &mut self.seat_state
    }
}

delegate_seat!(Hutch);

impl DataDeviceHandler for Hutch {
    fn data_device_state(&self) -> &smithay::wayland::data_device::DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for Hutch {}
impl ServerDndGrabHandler for Hutch {}

delegate_data_device!(Hutch);

delegate_output!(Hutch);
