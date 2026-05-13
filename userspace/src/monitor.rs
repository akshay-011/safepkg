use anyhow::Result;
use crate::engine::SecurityEngine;

#[cfg(target_os = "linux")]
use safepkg_common::ExecEvent;
#[cfg(target_os = "linux")]
use aya::{Bpf, maps::perf::AsyncPerfEventArray, util::online_cpus};
#[cfg(target_os = "linux")]
use bytes::BytesMut;

/// The monitor component responsible for loading eBPF and handling events.
pub struct Monitor {
    #[allow(dead_code)]
    engine: SecurityEngine,
}

impl Monitor {
    pub fn new(engine: SecurityEngine) -> Self {
        Self { engine }
    }

    /// Loads the eBPF program and starts the event loop.
    #[cfg(target_os = "linux")]
    pub async fn start(&self, bpf_data: &[u8]) -> Result<()> {
        let mut bpf = Bpf::load(bpf_data)?;
        
        let mut events: AsyncPerfEventArray<_> = bpf.take_map("EVENTS").unwrap().try_into()?;

        for cpu_id in online_cpus()? {
            let mut buf = events.open(cpu_id, None)?;

            tokio::spawn(async move {
                let mut buffers = (0..10)
                    .map(|_| BytesMut::with_capacity(1024))
                    .collect::<Vec<_>>();

                loop {
                    let events = buf.read_events(&mut buffers).await.unwrap();
                    for i in 0..events.read {
                        let data = &buffers[i];
                        // Safety: ExecEvent is Pod
                        let event = unsafe { &*(data.as_ptr() as *const ExecEvent) };
                        
                        let command = std::str::from_utf8(&event.command)
                            .unwrap_or("unknown")
                            .trim_matches(char::from(0));
                        
                        println!("Detected execution: {} (PID: {})", command, event.pid);
                    }
                }
            });
        }

        Ok(())
    }

    /// Mock start for non-Linux platforms.
    #[cfg(not(target_os = "linux"))]
    #[allow(dead_code)]
    pub async fn start(&self, _bpf_data: &[u8]) -> Result<()> {
        println!("⚠️ eBPF monitoring is only supported on Linux. Skipping...");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Config;

    #[test]
    fn test_monitor_init() {
        let config = Config::default();
        let engine = SecurityEngine::new(config);
        let _monitor = Monitor::new(engine);
    }
}
