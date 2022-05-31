use smithay::{
    delegate_shm,
    delegate_compositor,
    backend::renderer::utils::on_commit_buffer_handler,
    wayland::{
        shm::ShmState,
        buffer::BufferHandler,
        compositor::{
            CompositorHandler,
            CompositorState
        },
    },
    reexports::wayland_server::{
        DisplayHandle,
        protocol::{
            wl_buffer,
            wl_surface::WlSurface
        },
    },
};

use crate::Hutch;

impl CompositorHandler for Hutch {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn commit(&mut self, dh: &DisplayHandle, surface: &WlSurface) {
        on_commit_buffer_handler(dh, surface);
        self.space.commit(surface);
    }
}

impl BufferHandler for Hutch {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl AsRef<ShmState> for Hutch {
    fn as_ref(&self) -> &ShmState {
        &self.shm_state
    }
}

// TODO: Finish DMA-BUF implementation
// TODO: Handle layershell for overlays such as MangoHud?

delegate_compositor!(Hutch);
delegate_shm!(Hutch);
