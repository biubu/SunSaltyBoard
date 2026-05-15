#[cfg(windows)]
use std::env;
#[cfg(windows)]
use std::process::Command;
#[cfg(target_os = "linux")]
use std::env;
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::path::PathBuf;

pub fn setup_autostart(enabled: bool) {
    #[cfg(windows)]
    {
        setup_autostart_windows(enabled);
    }

    #[cfg(target_os = "linux")]
    {
        setup_autostart_linux(enabled);
    }

    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let _ = enabled;
    }
}

#[cfg(windows)]
fn setup_autostart_windows(enabled: bool) {
    let exe_path = match env::current_exe() {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(e) => {
            log::error!("Failed to get current exe path: {}", e);
            return;
        }
    };

    let result = if enabled {
        Command::new("reg")
            .args([
                "add",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "SunSaltyBoard",
                "/t",
                "REG_SZ",
                "/d",
                &exe_path,
                "/f",
            ])
            .output()
    } else {
        Command::new("reg")
            .args([
                "delete",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "SunSaltyBoard",
                "/f",
            ])
            .output()
    };

    match result {
        Ok(output) => {
            if output.status.success() {
                log::info!("Auto-start {} on Windows", if enabled { "enabled" } else { "disabled" });
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::error!("Failed to {} auto-start on Windows: {}", if enabled { "enable" } else { "disable" }, stderr);
            }
        }
        Err(e) => {
            log::error!("Failed to run reg command for auto-start: {}", e);
        }
    }
}

#[cfg(target_os = "linux")]
fn setup_autostart_linux(enabled: bool) {
    let home = match env::var("HOME") {
        Ok(h) => PathBuf::from(h),
        Err(e) => {
            log::error!("Failed to get HOME directory: {}", e);
            return;
        }
    };
    let mut autostart_dir = home.join(".config").join("autostart");

    let desktop_file = autostart_dir.join("SunSaltyBoard.desktop");

    if enabled {
        if let Err(e) = fs::create_dir_all(&autostart_dir) {
            log::error!("Failed to create autostart directory: {}", e);
            return;
        }

        let exe_path = match env::current_exe() {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(e) => {
                log::error!("Failed to get current exe path: {}", e);
                return;
            }
        };

        let desktop_entry = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=SunSaltyBoard\n\
             Comment=Clipboard Manager\n\
             Exec={}\n\
             Hidden=false\n\
             NoDisplay=false\n\
             X-GNOME-Autostart-enabled=true\n",
            exe_path
        );

        if let Err(e) = fs::write(&desktop_file, desktop_entry) {
            log::error!("Failed to write desktop entry file: {}", e);
        } else {
            log::info!("Auto-start enabled on Linux");
        }
    } else {
        if desktop_file.exists() {
            if let Err(e) = fs::remove_file(&desktop_file) {
                log::error!("Failed to remove desktop entry file: {}", e);
            } else {
                log::info!("Auto-start disabled on Linux");
            }
        }
    }
}
