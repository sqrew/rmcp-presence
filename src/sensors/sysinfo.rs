//! System information sensors - CPU, memory, disk, processes, temps, users

use crate::shared::{format_bytes, format_duration, internal_error};
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sysinfo::{
    Components, CpuRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind, System, Users,
};

// === Parameter Types ===

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TopProcessesParams {
    #[schemars(description = "Number of top processes to show (default 10)")]
    #[serde(default)]
    pub count: Option<usize>,
    #[schemars(description = "Sort by: 'cpu' or 'memory' (default 'cpu')")]
    #[serde(default)]
    pub sort_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FindProcessParams {
    #[schemars(description = "Process name to search for (case-insensitive, partial match)")]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ProcessIdParams {
    #[schemars(description = "Process ID (PID) to get details for")]
    pub pid: u32,
}

// === Tool Functions ===

pub async fn get_system_info() -> Result<CallToolResult, McpError> {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );

    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_all();

    let disks = Disks::new_with_refreshed_list();

    let cpu_count = sys.cpus().len();
    let cpu_usage: f32 = sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / cpu_count as f32;
    let cpu_name = sys.cpus().first().map(|c| c.brand()).unwrap_or("Unknown");

    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    let mem_percent = (used_mem as f64 / total_mem as f64 * 100.0) as u64;

    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();

    let mut total_disk: u64 = 0;
    let mut free_disk: u64 = 0;
    for disk in disks.iter() {
        total_disk += disk.total_space();
        free_disk += disk.available_space();
    }

    let uptime_secs = System::uptime();
    let uptime_hours = uptime_secs / 3600;
    let uptime_mins = (uptime_secs % 3600) / 60;

    let load = System::load_average();

    let output = format!(
        "System Information:\n\
         \n\
         CPU: {} ({} cores)\n\
         CPU Usage: {:.1}%\n\
         \n\
         Memory: {} / {} ({:.0}%)\n\
         Swap: {} / {}\n\
         \n\
         Disk: {} / {} free\n\
         \n\
         Uptime: {}h {}m\n\
         Load Average: {:.2} {:.2} {:.2} (1m 5m 15m)",
        cpu_name,
        cpu_count,
        cpu_usage,
        format_bytes(used_mem),
        format_bytes(total_mem),
        mem_percent,
        format_bytes(used_swap),
        format_bytes(total_swap),
        format_bytes(free_disk),
        format_bytes(total_disk),
        uptime_hours,
        uptime_mins,
        load.one,
        load.five,
        load.fifteen
    );

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn get_top_processes(params: TopProcessesParams) -> Result<CallToolResult, McpError> {
    let mut sys = System::new_all();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_all();

    let count = params.count.unwrap_or(10);
    let sort_by = params.sort_by.unwrap_or_else(|| "cpu".to_string());

    let mut processes: Vec<_> = sys.processes().values().collect();

    match sort_by.as_str() {
        "memory" | "mem" => {
            processes.sort_by(|a, b| b.memory().cmp(&a.memory()));
        }
        _ => {
            processes.sort_by(|a, b| {
                b.cpu_usage()
                    .partial_cmp(&a.cpu_usage())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }

    let mut output = format!("Top {} processes by {}:\n\n", count, sort_by);
    output.push_str(&format!(
        "{:<8} {:<10} {:<10} {}\n",
        "PID", "CPU%", "Memory", "Name"
    ));
    output.push_str(&format!("{:-<50}\n", ""));

    for proc in processes.iter().take(count) {
        output.push_str(&format!(
            "{:<8} {:<10.1} {:<10} {}\n",
            proc.pid(),
            proc.cpu_usage(),
            format_bytes(proc.memory()),
            proc.name().to_string_lossy()
        ));
    }

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn find_process(params: FindProcessParams) -> Result<CallToolResult, McpError> {
    let mut sys = System::new_all();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_all();

    let search = params.name.to_lowercase();
    let mut matches: Vec<_> = sys
        .processes()
        .values()
        .filter(|p| {
            p.name()
                .to_string_lossy()
                .to_lowercase()
                .contains(&search)
        })
        .collect();

    matches.sort_by(|a, b| {
        b.cpu_usage()
            .partial_cmp(&a.cpu_usage())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut output = format!("Processes matching '{}':\n\n", params.name);

    if matches.is_empty() {
        output.push_str("No matching processes found.\n");
    } else {
        output.push_str(&format!(
            "{:<8} {:<10} {:<10} {}\n",
            "PID", "CPU%", "Memory", "Name"
        ));
        output.push_str(&format!("{:-<50}\n", ""));

        for proc in matches.iter().take(20) {
            output.push_str(&format!(
                "{:<8} {:<10.1} {:<10} {}\n",
                proc.pid(),
                proc.cpu_usage(),
                format_bytes(proc.memory()),
                proc.name().to_string_lossy()
            ));
        }

        if matches.len() > 20 {
            output.push_str(&format!("\n... and {} more matches\n", matches.len() - 20));
        }

        output.push_str(&format!("\nTotal matches: {}\n", matches.len()));
    }

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn get_process_details(params: ProcessIdParams) -> Result<CallToolResult, McpError> {
    let mut sys = System::new_all();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_all();

    let pid = sysinfo::Pid::from_u32(params.pid);

    let proc = sys
        .process(pid)
        .ok_or_else(|| internal_error(format!("Process {} not found", params.pid)))?;

    let mut output = format!("Process Details (PID {}):\n\n", params.pid);

    output.push_str(&format!("Name: {}\n", proc.name().to_string_lossy()));
    output.push_str(&format!("Status: {:?}\n", proc.status()));
    output.push_str(&format!("CPU Usage: {:.1}%\n", proc.cpu_usage()));
    output.push_str(&format!("Memory: {}\n", format_bytes(proc.memory())));
    output.push_str(&format!(
        "Virtual Memory: {}\n",
        format_bytes(proc.virtual_memory())
    ));

    if let Some(parent) = proc.parent() {
        output.push_str(&format!("Parent PID: {}\n", parent));
    }

    let run_time = proc.run_time();
    output.push_str(&format!("Running for: {}\n", format_duration(run_time)));

    if let Some(exe) = proc.exe() {
        output.push_str(&format!("Executable: {}\n", exe.display()));
    }

    if let Some(cwd) = proc.cwd() {
        output.push_str(&format!("Working Dir: {}\n", cwd.display()));
    }

    let cmd = proc.cmd();
    if !cmd.is_empty() {
        let cmd_str: Vec<_> = cmd.iter().map(|s| s.to_string_lossy()).collect();
        let cmd_display = cmd_str.join(" ");
        if cmd_display.len() > 200 {
            output.push_str(&format!("Command: {}...\n", &cmd_display[..200]));
        } else {
            output.push_str(&format!("Command: {}\n", cmd_display));
        }
    }

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn list_processes() -> Result<CallToolResult, McpError> {
    let mut sys = System::new_all();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_all();

    let mut processes: Vec<_> = sys.processes().values().collect();
    processes.sort_by(|a, b| {
        b.cpu_usage()
            .partial_cmp(&a.cpu_usage())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut output = String::from("All Running Processes:\n\n");
    output.push_str(&format!(
        "{:<8} {:<10} {:<10} {}\n",
        "PID", "CPU%", "Memory", "Name"
    ));
    output.push_str(&format!("{:-<60}\n", ""));

    for proc in processes.iter().take(50) {
        output.push_str(&format!(
            "{:<8} {:<10.1} {:<10} {}\n",
            proc.pid(),
            proc.cpu_usage(),
            format_bytes(proc.memory()),
            proc.name().to_string_lossy()
        ));
    }

    if processes.len() > 50 {
        output.push_str(&format!(
            "\n... and {} more processes\n",
            processes.len() - 50
        ));
    }

    output.push_str(&format!("\nTotal processes: {}\n", processes.len()));

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn get_network_stats() -> Result<CallToolResult, McpError> {
    let networks = Networks::new_with_refreshed_list();

    let mut output = String::from("Network Interface Statistics:\n\n");

    if networks.iter().count() == 0 {
        output.push_str("No network interfaces found.\n");
    } else {
        for (name, data) in networks.iter() {
            output.push_str(&format!("{}:\n", name));
            output.push_str(&format!(
                "  Received: {}\n",
                format_bytes(data.total_received())
            ));
            output.push_str(&format!(
                "  Transmitted: {}\n",
                format_bytes(data.total_transmitted())
            ));
            output.push_str(&format!("  Packets In: {}\n", data.total_packets_received()));
            output.push_str(&format!(
                "  Packets Out: {}\n",
                data.total_packets_transmitted()
            ));
            output.push_str(&format!("  Errors In: {}\n", data.total_errors_on_received()));
            output.push_str(&format!(
                "  Errors Out: {}\n",
                data.total_errors_on_transmitted()
            ));
            output.push('\n');
        }
    }

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn get_component_temps() -> Result<CallToolResult, McpError> {
    let components = Components::new_with_refreshed_list();

    let mut output = String::from("Component Temperatures:\n\n");

    if components.iter().count() == 0 {
        output.push_str("No temperature sensors found.\n");
    } else {
        for component in components.iter() {
            if let Some(temp) = component.temperature() {
                output.push_str(&format!("{}: {:.1}°C", component.label(), temp));
                if let Some(max) = component.max() {
                    output.push_str(&format!(" (max: {:.1}°C)", max));
                }
                if let Some(critical) = component.critical() {
                    output.push_str(&format!(" (critical: {:.1}°C)", critical));
                }
                output.push('\n');
            }
        }
    }

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn get_disk_info() -> Result<CallToolResult, McpError> {
    let disks = Disks::new_with_refreshed_list();

    let mut output = String::from("Disk Usage:\n\n");

    for disk in disks.iter() {
        let total = disk.total_space();
        let free = disk.available_space();
        let used = total - free;
        let percent = if total > 0 {
            (used as f64 / total as f64 * 100.0) as u64
        } else {
            0
        };

        output.push_str(&format!(
            "{} ({})\n  {} / {} ({:.0}% used)\n  Mount: {}\n\n",
            disk.name().to_string_lossy(),
            disk.file_system().to_string_lossy(),
            format_bytes(used),
            format_bytes(total),
            percent,
            disk.mount_point().display()
        ));
    }

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn get_users() -> Result<CallToolResult, McpError> {
    let users = Users::new_with_refreshed_list();

    let mut output = String::from("System Users:\n\n");

    if users.iter().count() == 0 {
        output.push_str("No users found.\n");
    } else {
        for user in users.iter() {
            output.push_str(&format!("{}\n", user.name()));
            output.push_str(&format!("  UID: {:?}\n", user.id()));
            output.push_str(&format!("  GID: {:?}\n", user.group_id()));
            let groups = user.groups();
            if !groups.is_empty() {
                let group_names: Vec<_> = groups.iter().map(|g| g.name().to_string()).collect();
                output.push_str(&format!("  Groups: {}\n", group_names.join(", ")));
            }
            output.push('\n');
        }
        output.push_str(&format!("Total users: {}\n", users.iter().count()));
    }

    Ok(CallToolResult::success(vec![Content::text(output)]))
}
