use smithay::{
    desktop::{Space, WindowSurfaceType},
    reexports::{
        calloop::{generic::Generic, EventLoop, Interest, LoopSignal, Mode, PostAction},
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::wl_surface::WlSurface,
            Display,
        },
    },
    utils::{Logical, Point},
    wayland::{
        compositor::CompositorState,
        data_device::DataDeviceState,
        output::OutputManagerState,
        seat::{Seat, SeatState},
        shell::xdg::XdgShellState,
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};

use slog::Logger;

use crate::CalloopData;

pub struct Hutch {
    pub pointer_location: Point<f64, Logical>,

    pub start_time: std::time::Instant,
    pub socket_name: std::ffi::OsString,

    pub space: Space,
    pub loop_signal: LoopSignal,
    pub log: slog::Logger,

    // Smithay State
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub seat_state: SeatState<Self>,
    pub data_device_state: DataDeviceState,

    pub seat: Seat<Self>,
}

impl Hutch {
    pub fn new(
        evloop: &mut EventLoop<CalloopData>,
        display: &mut Display<Self>,
        log: Logger
    ) -> Self {
        let start_time = std::time::Instant::now();

        let dh = display.handle();

        let compositor_state: CompositorState = CompositorState::new::<Self, _>(&dh, log.clone());
        let xdg_shell_state: XdgShellState = XdgShellState::new::<Self, _>(&dh, log.clone()).0;
        let shm_state: ShmState = ShmState::new::<Self, _>(&dh, vec![], log.clone());
        let output_manager_state: OutputManagerState = OutputManagerState::new_with_xdg_output::<Self>(&dh);
        let seat_state: SeatState<Self> = SeatState::new();
        let data_device_state: DataDeviceState = DataDeviceState::new::<Self, _>(&dh, log.clone());

        // A seat is a group of keyboards, pointer and touch devices.
        // A seat typically has a pointer and maintains a keyboard focus and a pointer focus.
        let mut seat: Seat<Self> = Seat::new(&dh, "winit", log.clone());

        // Notify clients that we have a keyboard, for the sake of the example we assume that keyboard is always present.
        // You may want to track keyboard hot-plug in real compositor.
        seat
            .add_keyboard(Default::default(), 300, 80, |_, _| {})
            .expect("Failed to register new seat.");

        // Notify clients that we have a pointer (mouse)
        // Here we assume that there is always pointer plugged in
        seat.add_pointer(|_| {});

        // A space represents a two-dimensional plane. Windows and Outputs can be mapped onto it.
        //
        // Windows get a position and stacking order through mapping.
        // Outputs become views of a part of the Space and can be rendered via Space::render_output.
        let space: Space = Space::new(log.clone());

        let socket_name: std::ffi::OsString = Self::socket_listener(display, evloop, log.clone());

        // Get the loop signal, used to stop the event loop
        let loop_signal: LoopSignal = evloop.get_signal();

        Self {
            pointer_location: Default::default(),

            start_time,

            space,
            loop_signal,
            socket_name,

            log,
            compositor_state,
            xdg_shell_state,
            shm_state,
            output_manager_state,
            seat_state,
            data_device_state,
            seat,
        }
    }

    fn socket_listener(
        display: &mut Display<Self>,
        evloop: &mut EventLoop<CalloopData>,
        log: slog::Logger,
    ) -> std::ffi::OsString {
        // Creates a new listening socket, automatically choosing the next available `wayland` socket name.
        let socket = ListeningSocketSource::new_auto(log).unwrap();

        // Get the name of the listening socket.
        // Clients will connect to this socket.
        let socket_name = socket
            .socket_name()
            .to_os_string();

        let handle = evloop.handle();

        evloop
            .handle()
            .insert_source(socket, move |client_stream, _, state| {
                state.display
                    .handle()
                    .insert_client(client_stream, std::sync::Arc::new(ClientState))
                    .unwrap();
            })
            .expect("Failed to init the wayland event source.");

        // You also need to add the display itself to the event loop, so that client events will be processed by wayland-server.
        handle
            .insert_source(
                Generic::new(display.backend().poll_fd(), Interest::READ, Mode::Level),
                |_, _, state| {
                    state.display.dispatch_clients(&mut state.state).unwrap();
                    Ok(PostAction::Continue)
                },
            )
            .unwrap();

        socket_name
    }

    pub fn surface_under_pointer(&self) -> Option<(WlSurface, Point<i32, Logical>)> {
        let pos = self.pointer_location;

        self.space
            .surface_under(pos, WindowSurfaceType::all())
            .map(|(_, surface, location)| (surface, location))
    }
}

pub struct ClientState;

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}
