use drm::control::Device as ControlDevice;
use drm::Device;

use crate::dpi::{PhysicalPosition, PhysicalSize};

pub mod event_loop;
mod input_to_vk;
pub mod window;
pub use drm::SystemError;
pub use event_loop::EventLoop;
pub use event_loop::EventLoopProxy;
pub use event_loop::EventLoopWindowTarget;
pub use window::Window;

#[derive(Debug)]
/// A simple wrapper for a device node.
pub struct Card(std::fs::File);

/// Implementing `AsRawFd` is a prerequisite to implementing the traits found
/// in this crate. Here, we are just calling `as_raw_fd()` on the inner File.
impl std::os::unix::io::AsRawFd for Card {
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.0.as_raw_fd()
    }
}

/// With `AsRawFd` implemented, we can now implement `drm::Device`.
impl Device for Card {}
impl ControlDevice for Card {}

/// Simple helper methods for opening a `Card`.
impl Card {
    pub fn open(path: &str) -> Result<Self, std::io::Error> {
        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        options.write(true);
        Ok(Card(options.open(path)?))
    }

    pub fn open_global() -> Result<Self, std::io::Error> {
        Self::open("/dev/dri/card0")
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId;

impl DeviceId {
    pub const unsafe fn dummy() -> Self {
        DeviceId
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorHandle(drm::control::connector::Info);

impl PartialOrd for MonitorHandle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.interface_id().cmp(&other.0.interface_id()))
    }
}

impl Ord for MonitorHandle {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.interface_id().cmp(&other.0.interface_id())
    }
}

impl MonitorHandle {
    #[inline]
    pub fn name(&self) -> Option<String> {
        Some(String::from("card0"))
    }

    #[inline]
    pub fn native_identifier(&self) -> u32 {
        self.0.interface_id()
    }

    #[inline]
    pub fn size(&self) -> PhysicalSize<u32> {
        let size = self.0.modes()[0].size();
        PhysicalSize::new(size.0 as u32, size.1 as u32)
    }

    #[inline]
    pub fn position(&self) -> PhysicalPosition<i32> {
        PhysicalPosition::new(0, 0)
    }

    #[inline]
    pub fn scale_factor(&self) -> f64 {
        1.0
    }

    #[inline]
    pub fn video_modes(&self) -> impl Iterator<Item = crate::monitor::VideoMode> {
        let modes = self.0.modes().to_vec();
        let monitor = self.0.clone();
        modes.into_iter().map(move |f| crate::monitor::VideoMode {
            video_mode: crate::platform_impl::VideoMode::Drm(VideoMode(f, monitor.clone())),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VideoMode(drm::control::Mode, drm::control::connector::Info);

impl VideoMode {
    #[inline]
    pub fn size(&self) -> PhysicalSize<u32> {
        let size = self.0.size();
        PhysicalSize::new(size.0 as u32, size.1 as u32)
    }

    #[inline]
    pub fn bit_depth(&self) -> u16 {
        32
    }

    #[inline]
    pub fn refresh_rate(&self) -> u16 {
        self.0.vrefresh() as u16
    }

    #[inline]
    pub fn monitor(&self) -> crate::monitor::MonitorHandle {
        crate::monitor::MonitorHandle {
            inner: crate::platform_impl::MonitorHandle::Drm(MonitorHandle(self.1.clone())),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId;

impl WindowId {
    pub const unsafe fn dummy() -> Self {
        Self
    }
}
