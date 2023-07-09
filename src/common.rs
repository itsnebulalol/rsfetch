use reqwest::blocking::get;
use serde_json::from_str;
use std::env;
use std::fs::{read_dir, read_to_string, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn get_uname() -> String {
    let uname = std::process::Command::new("uname")
        .output()
        .expect("Failed to execute command")
        .stdout;

    return String::from_utf8_lossy(&uname).trim().to_string();
}

pub fn get_user() -> String {
    match env::var("USER") {
        Ok(val) => return val,
        Err(_e) => return "Unknown".into(),
    }
}

pub fn get_os_version(os: &str) -> String {
    if os == "Darwin" {
        let product_name = std::process::Command::new("sw_vers")
            .arg("-productName")
            .output()
            .expect("Failed to execute command")
            .stdout;

        let product_version = std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .expect("Failed to execute command")
            .stdout;

        let build_version = std::process::Command::new("sw_vers")
            .arg("-buildVersion")
            .output()
            .expect("Failed to execute command")
            .stdout;

        return format!(
            "{} {} ({})",
            String::from_utf8_lossy(&product_name).trim(),
            String::from_utf8_lossy(&product_version).trim(),
            String::from_utf8_lossy(&build_version).trim()
        );
    } else if os == "Linux" {
        let mut kernel = String::new();
        if let Ok(file) = File::open("/proc/version") {
            let reader = BufReader::new(file);
            if let Some(version_line) = reader.lines().next() {
                if let Ok(version) = version_line {
                    kernel = version.split(' ').collect::<Vec<&str>>()[2].to_string();
                }
            }
        }

        let mut distro = String::new();
        if let Ok(file) = File::open("/etc/os-release") {
            let reader = BufReader::new(file);
            for line_result in reader.lines() {
                if let Ok(line) = line_result {
                    if line.starts_with("PRETTY_NAME") {
                        distro = line
                            .replace("PRETTY_NAME=\"", "")
                            .replace("\"", "")
                            .replace("\n", "");
                        break;
                    }
                }
            }
        }

        return format!("{} (Linux {})", distro.trim(), kernel.trim());
    } else {
        return format!("Unknown {}", os);
    }
}

pub fn get_model(os: &str) -> String {
    let oem_info = vec![
        "To be filled by O.E.M",
        r"To Be Filled*",
        r"OEM*",
        "Not Applicable",
        "System Product Name",
        "System Version",
        "Undefined",
        "Default string",
        "Not Specified",
        "Type1ProductConfigId",
        "INVALID",
        "All Series",
        "\u{fffd}",
    ];
    let mut model = String::new();

    if os == "Linux" {
        let board_vendor_path = Path::new("/sys/devices/virtual/dmi/id/board_vendor");
        let board_name_path = Path::new("/sys/devices/virtual/dmi/id/board_name");

        if board_vendor_path.exists() || board_name_path.exists() {
            let board_vendor = read_to_string(board_vendor_path).unwrap();
            let board_name = read_to_string(board_name_path).unwrap();
            model = format!("{} {}", board_vendor.trim(), board_name.trim());
        }

        let product_name_path = Path::new("/sys/devices/virtual/dmi/id/product_name");
        let product_version_path = Path::new("/sys/devices/virtual/dmi/id/product_version");

        if product_name_path.exists() || product_version_path.exists() {
            let product_name = read_to_string(product_name_path).unwrap();
            let product_version = read_to_string(product_version_path).unwrap();
            model = format!("{} {}", product_name.trim(), product_version.trim());
        }

        let devtree_model_path = Path::new("/sys/firmware/devicetree/base/model");

        if devtree_model_path.exists() {
            let devtree_model = read_to_string(devtree_model_path).unwrap();
            model = devtree_model.trim().to_string();
        }

        let sysinfo_model_path = Path::new("/tmp/sysinfo/model");

        if sysinfo_model_path.exists() {
            let sysinfo_model = read_to_string(sysinfo_model_path).unwrap();
            model = sysinfo_model.trim().to_string();
        }
    } else if os == "Darwin" {
        let prod_name = String::from_utf8_lossy(
            &std::process::Command::new("sw_vers")
                .arg("-productName")
                .output()
                .expect("Failed to execute command")
                .stdout,
        )
        .trim()
        .to_string();
        let machine_id: String;

        if prod_name == "iPhone OS" {
            machine_id = String::from_utf8_lossy(
                &std::process::Command::new("uname")
                    .arg("-m")
                    .output()
                    .expect("Failed to execute command")
                    .stdout,
            )
            .trim()
            .to_string();
        } else {
            machine_id = String::from_utf8_lossy(
                &std::process::Command::new("sysctl")
                    .arg("-n")
                    .arg("hw.model")
                    .output()
                    .expect("Failed to execute command")
                    .stdout,
            )
            .trim()
            .to_string();

            let kexts = String::from_utf8_lossy(
                &std::process::Command::new("kextstat")
                    .output()
                    .expect("Failed to execute command")
                    .stdout,
            )
            .to_string();
            let is_hackintosh = kexts.contains("FakeSMC") || kexts.contains("VirtualSMC");

            if is_hackintosh {
                model.push_str(" (Hackintosh)");
            }
        }

        let model_res = get(&format!(
            "https://di-api.reincubate.com/v1/apple-identifiers/{}/",
            machine_id
        ));

        if let Ok(res) = model_res {
            if res.status().is_success() {
                let res_content = res.text().unwrap();
                let j: serde_json::Value = from_str(&res_content).unwrap();
                if let Some(str_val) = j["product"]["sku"].as_str() {
                    model = str_val.to_string();
                }
            }
        }
    }

    for info in &oem_info {
        model = model.replace(info, "");
    }

    if model.contains("Standard PC") {
        return format!("{} (KVM)", model.trim());
    }

    if model.is_empty() {
        model = "Unknown".to_string();
    }

    return model.trim().to_string();
}

pub fn get_cpu_info() -> String {
    if cfg!(target_os = "linux") {
        let contents =
            read_to_string("/proc/cpuinfo").expect("Something went wrong reading the file");

        for line in contents.lines() {
            if line.starts_with("model name") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    return parts[1].trim().to_string();
                }
            }
        }
    } else if cfg!(target_os = "macos") {
        if let Ok(output) = std::process::Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output()
        {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    String::from("Unknown")
}

fn in_path(cmd: &str) -> bool {
    if let Ok(paths) = env::var("PATH") {
        let paths: Vec<&str> = paths.split(':').collect();

        for path in paths {
            let full_path = Path::new(path).join(cmd);

            if full_path.exists() {
                return true;
            }
        }
    }

    return false;
}

fn file_count(directory: &str) -> usize {
    read_dir(directory).map(|files| files.count()).unwrap_or(0)
}

pub fn get_packages() -> String {
    let mut packages = String::new();

    if in_path("pacman") {
        let count = file_count("/var/lib/pacman/local");
        packages.push_str(&format!(
            "{}{} pacman",
            if packages.is_empty() { "" } else { ", " },
            count
        ));
    }

    if in_path("rpm") {
        let output = &std::process::Command::new("rpm")
            .arg("-qa")
            .output()
            .expect("Failed to execute command");

        let count = String::from_utf8_lossy(&output.stdout).lines().count();

        packages.push_str(&format!(
            "{}{} rpm",
            if packages.is_empty() { "" } else { ", " },
            count
        ));
    }

    if in_path("emerge") {
        let count = file_count("/var/db/pkg");
        packages.push_str(&format!(
            "{}{} emerge",
            if packages.is_empty() { "" } else { ", " },
            count
        ));
    }

    if in_path("xbps-query") {
        let output = &std::process::Command::new("xbps-query")
            .arg("-l")
            .output()
            .expect("Failed to execute command");

        let count = String::from_utf8_lossy(&output.stdout).lines().count();

        packages.push_str(&format!(
            "{}{} xbps",
            if packages.is_empty() { "" } else { ", " },
            count
        ));
    }

    if in_path("dpkg") {
        let output = &std::process::Command::new("dpkg")
            .arg("-l")
            .output()
            .expect("Failed to execute command");

        let count = String::from_utf8_lossy(&output.stdout).lines().count();

        if count != 0 {
            packages.push_str(&format!(
                "{}{} dpkg",
                if packages.is_empty() { "" } else { ", " },
                count
            ));
        }
    }

    if in_path("brew") {
        let output = &std::process::Command::new("brew")
            .arg("list")
            .arg("-1")
            .output()
            .expect("Failed to execute command");

        let count = String::from_utf8_lossy(&output.stdout).lines().count();

        packages.push_str(&format!(
            "{}{} brew",
            if packages.is_empty() { "" } else { ", " },
            count
        ));
    }

    if in_path("port") {
        let output = &std::process::Command::new("port")
            .arg("installed")
            .output()
            .expect("Failed to execute command");

        let count = String::from_utf8_lossy(&output.stdout).lines().count();

        packages.push_str(&format!(
            "{}{} port",
            if packages.is_empty() { "" } else { ", " },
            count
        ));
    }

    if packages.is_empty() {
        return String::from("Unknown");
    }

    return packages;
}

pub fn get_memory_usage() -> String {
    match sys_info::mem_info() {
        Ok(info) => {
            let total_mem_mb: f64 = info.total as f64 / 1024.0; // Convert KB to MB
            let used_mem_mb: f64 = total_mem_mb - (info.free as f64 / 1024.0); // Subtract free mem from total mem
            let percent_used: f64 = (used_mem_mb / total_mem_mb) * 100.0; // Calculate the percentage
            return format!(
                "{:.2} MB / {:.2} MB ({:.1}% used)",
                used_mem_mb, total_mem_mb, percent_used
            );
        }
        Err(e) => return format!("Unknown: {}", e),
    }
}

pub fn get_uptime() -> String {
    match uptime_lib::get() {
        Ok(uptime) => {
            let n_secs = uptime.as_secs();
            let duration = chrono::Duration::seconds(n_secs as i64);

            let days = duration.num_days();
            let hours = duration.num_hours() % 24;
            let minutes = duration.num_minutes() % 60;
            let seconds = duration.num_seconds() % 60;

            return format!("{} days, {:02}:{:02}:{:02}", days, hours, minutes, seconds);
        }
        Err(err) => return format!("Unknown: {}", err),
    }
}

pub fn get_gpu_info() -> String {
    let os = std::env::consts::OS;

    if os == "macos" {
        let prod_name = &std::process::Command::new("sw_vers")
            .arg("-productName")
            .output()
            .expect("Failed to execute command");

        if String::from_utf8_lossy(&prod_name.stdout) == "iPhone OS" {
            // Handle iPhone OS case
        } else {
            let gpu_info = &std::process::Command::new("system_profiler")
                .arg("SPDisplaysDataType")
                .output()
                .expect("Failed to execute command");

            for line in String::from_utf8_lossy(&gpu_info.stdout)
                .lines()
                .collect::<Vec<_>>()
            {
                let parts: Vec<&str> = line.split("Chipset Model: ").collect();
                if parts.len() > 1 {
                    let gpu = parts[1];
                    return gpu.trim().to_string();
                }
            }
        }
    } else {
        let lspci = &std::process::Command::new("lspci")
            .output()
            .expect("Failed to execute command");

        for line in String::from_utf8_lossy(&lspci.stdout)
            .lines()
            .collect::<Vec<_>>()
        {
            if line.contains("Display") || line.contains("3D") || line.contains("VGA") {
                let gpu = line
                    .split(": ")
                    .nth(1)
                    .unwrap()
                    .split(" (rev")
                    .next()
                    .unwrap();
                return gpu.to_string();
            }
        }
    }

    return String::from("Unknown");
}

pub fn get_shell() -> String {
    let shell = env::var("SHELL").unwrap();
    let shell_clean = shell.split("/").last().unwrap();

    match shell_clean {
        "sh" | "ash" | "dash" | "es" => shell_clean.to_string(),
        "bash" => {
            let output = &std::process::Command::new(&shell)
                .arg("--version")
                .output()
                .expect("Failed to execute command");

            format!(
                "bash {}",
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap()
                    .split("version ")
                    .nth(1)
                    .unwrap()
                    .split(" (")
                    .next()
                    .unwrap()
            )
        }
        "zsh" => {
            let output = &std::process::Command::new(&shell)
                .arg("--version")
                .output()
                .expect("Failed to execute command");

            format!(
                "{}",
                String::from_utf8_lossy(&output.stdout)
                    .split(" (")
                    .next()
                    .unwrap()
            )
        }
        _ => {
            let output = &std::process::Command::new(&shell)
                .arg("--version")
                .output()
                .expect("Failed to execute command");

            let version = String::from_utf8_lossy(&output.stdout);
            if shell_clean == version {
                shell_clean.to_string()
            } else {
                format!("{} {}", shell_clean, version)
            }
        }
    }
}
