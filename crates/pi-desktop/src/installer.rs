use anyhow::{Context, Result};
#[cfg(windows)]
mod windows;
use std::path::PathBuf;
#[cfg(windows)]
use windows::*;

/// Information about the Volt application installation.
#[allow(dead_code)]
pub struct InstallInfo {
    pub app_name: &'static str,
    pub executable_name: &'static str,
    pub install_dir: PathBuf,
    pub executable_path: PathBuf,
    pub desktop_shortcut: Option<PathBuf>,
    pub start_menu_shortcut: Option<PathBuf>,
}

impl InstallInfo {
    pub fn new() -> Result<Self> {
        let app_name = "Volt";
        let executable_name = if cfg!(windows) { "volt.exe" } else { "volt" };

        let install_dir = if cfg!(windows) {
            let local_app_data = std::env::var("LOCALAPPDATA")
                .or_else(|_| std::env::var("USERPROFILE").map(|p| format!("{}\\AppData\\Local", p)))
                .context("无法定位用户安装目录")?;
            PathBuf::from(local_app_data)
                .join("Programs")
                .join(app_name)
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".local").join("bin")
        };

        let executable_path = install_dir.join(executable_name);

        let desktop_shortcut = if cfg!(windows) {
            std::env::var("USERPROFILE").ok().map(|p| {
                PathBuf::from(p)
                    .join("Desktop")
                    .join(format!("{}.lnk", app_name))
            })
        } else {
            None
        };

        let start_menu_shortcut = if cfg!(windows) {
            std::env::var("APPDATA").ok().map(|p| {
                PathBuf::from(p)
                    .join("Microsoft\\Windows\\Start Menu\\Programs")
                    .join(format!("{}.lnk", app_name))
            })
        } else {
            None
        };

        Ok(Self {
            app_name,
            executable_name,
            install_dir,
            executable_path,
            desktop_shortcut,
            start_menu_shortcut,
        })
    }
}

/// Check if the currently running executable is already installed in the target directory.
pub fn is_running_installed() -> bool {
    let current_exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(_) => return false,
    };

    let info = match InstallInfo::new() {
        Ok(info) => info,
        Err(_) => return false,
    };

    // Compare canonical paths if available, otherwise fallback to lowercase string comparison
    if let (Ok(c1), Ok(c2)) = (
        current_exe.canonicalize(),
        info.executable_path.canonicalize(),
    ) {
        if c1 == c2 {
            return true;
        }
    }

    // Compare canonical or lowercase paths on Windows
    let cur_str = current_exe.to_string_lossy().to_lowercase();
    let target_str = info.executable_path.to_string_lossy().to_lowercase();
    cur_str == target_str
}

/// Handle CLI flags (--install, --uninstall, --portable, --status).
/// Returns Ok(true) if the installer handled the command and the application should exit.
/// Returns Ok(false) if normal application execution should continue.
pub fn handle_cli_args() -> Result<bool> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        // If on Windows and not in install directory and not in development (target/debug or cargo run),
        // we can check if the user launched directly from a download folder.
        #[cfg(windows)]
        {
            let is_spawned_by_installer = std::env::var("VOLT_SPAWNED_BY_INSTALLER").is_ok();
            if !is_spawned_by_installer && !is_running_installed() && !is_dev_environment() {
                println!("========================================================");
                println!("           Volt Native AI Assistant - Windows           ");
                println!("========================================================");
                println!("检测到程序运行于非安装路径。");
                println!("正在自动执行自安装（%LOCALAPPDATA%\\Programs\\Volt）...");
                if let Err(e) = install(false) {
                    eprintln!("自动安装失败: {:#}", e);
                } else {
                    println!("自安装完成！正在启动安装副本...");
                    let info = InstallInfo::new()?;
                    std::process::Command::new(&info.executable_path)
                        .env("VOLT_SPAWNED_BY_INSTALLER", "1")
                        .spawn()
                        .context("启动安装副本失败")?;
                    return Ok(true);
                }
            }
        }
        return Ok(false);
    }

    let flag = args[1].as_str();
    match flag {
        "--install" | "-i" | "/install" => {
            let create_desktop = !args.iter().any(|a| a == "--no-desktop");
            install(create_desktop)?;
            Ok(true)
        }
        "--uninstall" | "-u" | "/uninstall" => {
            uninstall()?;
            Ok(true)
        }
        "--portable" | "-p" => {
            // Run without self-installation
            Ok(false)
        }
        "--install-status" => {
            print_status()?;
            Ok(true)
        }
        "--help" | "-h" | "/?" => {
            print_help();
            Ok(true)
        }
        _ => Ok(false),
    }
}

#[allow(dead_code)]
fn is_dev_environment() -> bool {
    if let Ok(exe) = std::env::current_exe() {
        let path = exe.to_string_lossy().to_lowercase();
        if path.contains("target\\debug")
            || path.contains("target/debug")
            || path.contains("target\\release")
            || path.contains("target/release")
        {
            return true;
        }
    }
    false
}

fn print_help() {
    println!("Volt Desktop (Native AI Coding Assistant)");
    println!();
    println!("Usage:");
    println!("  volt [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --install, -i          Perform Windows self-installation (copy exe, create shortcuts, add PATH)");
    println!("  --no-desktop           When installing, do not create a Desktop shortcut");
    println!("  --uninstall, -u        Cleanly uninstall Volt from Windows (remove files, shortcuts, registry)");
    println!("  --portable, -p         Run in portable mode without installation checks");
    println!("  --install-status       Show current installation status");
    println!("  --help, -h             Show this help message");
}

fn print_status() -> Result<()> {
    let info = InstallInfo::new()?;
    println!("Volt Installation Status:");
    println!("  Target Directory:  {}", info.install_dir.display());
    println!("  Executable Path:   {}", info.executable_path.display());
    println!(
        "  Installed:         {}",
        if info.executable_path.exists() {
            "YES"
        } else {
            "NO"
        }
    );
    println!(
        "  Running Installed: {}",
        if is_running_installed() { "YES" } else { "NO" }
    );
    if let Some(desktop) = &info.desktop_shortcut {
        println!(
            "  Desktop Shortcut:  {} ({})",
            desktop.display(),
            if desktop.exists() { "Found" } else { "None" }
        );
    }
    if let Some(start) = &info.start_menu_shortcut {
        println!(
            "  Start Menu:        {} ({})",
            start.display(),
            if start.exists() { "Found" } else { "None" }
        );
    }
    Ok(())
}

/// Perform Windows Self-Installation
pub fn install(#[allow(unused_variables)] create_desktop: bool) -> Result<()> {
    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;
    let info = InstallInfo::new()?;

    println!("========================================================");
    println!("   Volt Windows Self-Installation (自安装执行中...)      ");
    println!("========================================================");
    println!("目标安装目录: {}", info.install_dir.display());

    // 1. Create target directory
    std::fs::create_dir_all(&info.install_dir)
        .with_context(|| format!("创建目标目录失败: {}", info.install_dir.display()))?;

    // 2. Copy binary if not already running in place (case-insensitive check for Windows)
    let is_same_binary = current_exe == info.executable_path
        || (cfg!(windows)
            && current_exe
                .to_string_lossy()
                .eq_ignore_ascii_case(&info.executable_path.to_string_lossy()));

    if !is_same_binary {
        println!("正在复制二进制文件到安装目录...");
        std::fs::copy(&current_exe, &info.executable_path)
            .with_context(|| format!("复制文件到 {} 失败", info.executable_path.display()))?;
    } else {
        println!("程序已位于目标目录中，无需重复复制。");
    }

    #[cfg(windows)]
    {
        // 3. Create Start Menu shortcut
        if let Some(start_menu) = &info.start_menu_shortcut {
            if let Some(parent) = start_menu.parent() {
                std::fs::create_dir_all(parent).context("创建快捷方式目录失败")?;
            }
            println!("正在创建开始菜单快捷方式: {}", start_menu.display());
            create_windows_shortcut(
                &info.executable_path,
                start_menu,
                &info.install_dir,
                "Volt Native AI Assistant",
            )?;
        }

        // 4. Create Desktop shortcut
        if create_desktop {
            if let Some(desktop) = &info.desktop_shortcut {
                println!("正在创建桌面快捷方式: {}", desktop.display());
                create_windows_shortcut(
                    &info.executable_path,
                    desktop,
                    &info.install_dir,
                    "Volt Native AI Assistant",
                )?;
            }
        }

        // 5. Add to User PATH
        println!("正在配置用户环境变量 PATH...");
        add_to_windows_path(&info.install_dir)?;

        // 6. Register in Windows Add/Remove Programs (Registry)
        println!("正在向 Windows 注册表注册卸载项...");
        register_windows_uninstall(&info)?;
    }

    println!("========================================================");
    println!("✓ Volt 自安装成功完成！");
    println!("· 可通过桌面图标、开始菜单或在终端输入 `volt` 直接启动。");
    println!("========================================================");
    Ok(())
}

/// Perform Clean Uninstallation
pub fn uninstall() -> Result<()> {
    let info = InstallInfo::new()?;

    println!("========================================================");
    println!("   Volt Windows Clean Uninstall (正在卸载...)          ");
    println!("========================================================");

    #[cfg(windows)]
    {
        // 1. Remove Desktop shortcut
        if let Some(desktop) = &info.desktop_shortcut {
            if desktop.exists() {
                println!("正在删除桌面快捷方式...");
                std::fs::remove_file(desktop).context("删除安装文件失败")?;
            }
        }

        // 2. Remove Start Menu shortcut
        if let Some(start_menu) = &info.start_menu_shortcut {
            if start_menu.exists() {
                println!("正在删除开始菜单快捷方式...");
                std::fs::remove_file(start_menu).context("删除安装文件失败")?;
            }
        }

        // 3. Remove from User PATH
        println!("正在从 PATH 移除 Volt 路径...");
        remove_from_windows_path(&info.install_dir)?;

        // 4. Remove Registry key
        println!("正在注销 Windows 卸载项...");
        unregister_windows_uninstall()?;

        // 5. Schedule deletion of install directory
        println!("正在清理程序文件与目录...");
        schedule_self_delete(&info.install_dir)?;
    }

    #[cfg(not(windows))]
    {
        if info.executable_path.exists() {
            std::fs::remove_file(&info.executable_path).context("删除安装文件失败")?;
        }
        println!("已移除安装副本: {}", info.executable_path.display());
    }

    #[cfg(windows)]
    println!("系统集成已移除；程序文件删除任务已启动，退出后完成。");
    #[cfg(not(windows))]
    println!("✓ Volt 安装副本已移除。");
    Ok(())
}

// ============================================================================
// Windows Platform Specific Implementations
// ============================================================================
