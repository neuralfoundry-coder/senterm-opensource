use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

pub struct SystemManager {
    pub sys: System,
}

impl SystemManager {
    pub fn new() -> Self {
        Self {
            sys: System::new_with_specifics(
                RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()).with_memory(MemoryRefreshKind::everything()),
            ),
        }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_all();
    }
}
