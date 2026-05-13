#![no_std]
#![no_main]

use aya_ebpf::{
    macros::{kprobe, map},
    maps::PerfEventArray,
    programs::ProbeContext,
    helpers::bpf_get_current_pid_tgid,
};
use safepkg_common::ExecEvent;

#[map]
static mut EVENTS: PerfEventArray<ExecEvent> = PerfEventArray::new(0);

#[kprobe]
pub fn safepkg_exec(ctx: ProbeContext) -> u32 {
    match try_safepkg_exec(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_safepkg_exec(ctx: ProbeContext) -> Result<u32, u32> {
    let pid_tgid = bpf_get_current_pid_tgid();
    let pid = (pid_tgid >> 32) as u32;
    
    // In a real implementation, we would get ppid and command name here
    let event = ExecEvent {
        pid,
        ppid: 0, // Placeholder
        command: [0; 16],
    };

    unsafe { EVENTS.output(&ctx, &event, 0) };
    Ok(0)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
