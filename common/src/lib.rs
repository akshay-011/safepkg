#[repr(C)]
#[derive(Clone, Copy)]
pub struct ExecEvent {
    pub pid: u32,
    pub ppid: u32,
    pub command: [u8; 16],
}

#[cfg(feature = "user")]
#[cfg(target_os = "linux")] unsafe impl aya::Pod for ExecEvent {}
