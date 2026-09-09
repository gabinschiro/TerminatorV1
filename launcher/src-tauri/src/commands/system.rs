use serde::Serialize;
use sysinfo::{System, Disks};

#[derive(Serialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub total_memory_gb: u64,
    pub used_memory_gb: u64,
    pub cpu_name: String,
    pub cpu_cores: usize,
    pub disk_free_gb: u64,
}

#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu = sys.cpus().first().map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Inconnu".to_string());

    let disk_free = Disks::new_with_refreshed_list()
        .iter()
        .map(|d| d.available_space())
        .sum::<u64>()
        / (1024 * 1024 * 1024);

    SystemInfo {
        os_name: System::name().unwrap_or_default(),
        os_version: System::os_version().unwrap_or_default(),
        total_memory_gb: sys.total_memory() / 1024,
        used_memory_gb: sys.used_memory() / 1024,
        cpu_name: cpu,
        cpu_cores: sys.cpus().len(),
        disk_free_gb: disk_free,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_info_is_populated() {
        let info = get_system_info();
        assert!(info.cpu_cores > 0);
        assert!(info.total_memory_gb > 0);
    }
}