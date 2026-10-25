use super::InstallInfo;
use anyhow::{Context, Result};
use std::path::Path;

pub(super) fn create_windows_shortcut(
    target_exe: &Path,
    shortcut_path: &Path,
    working_dir: &Path,
    description: &str,
) -> Result<()> {
    let target_str = target_exe.to_string_lossy().replace('\'', "''");
    let shortcut_str = shortcut_path.to_string_lossy().replace('\'', "''");
    let working_dir_str = working_dir.to_string_lossy().replace('\'', "''");

    let script = format!(
        "$WshShell = New-Object -ComObject WScript.Shell; \
         $Shortcut = $WshShell.CreateShortcut('{shortcut}'); \
         $Shortcut.TargetPath = '{target}'; \
         $Shortcut.WorkingDirectory = '{workdir}'; \
         $Shortcut.Description = '{desc}'; \
         $Shortcut.Save()",
        shortcut = shortcut_str,
        target = target_str,
        workdir = working_dir_str,
        desc = description.replace('\'', "''")
    );

    run_powershell(&script, None).map(|_| ())
}

pub(super) fn add_to_windows_path(dir: &Path) -> Result<()> {
    update_path(
        dir,
        "if ($paths -notcontains $env:VOLT_INSTALL_DIR) { $paths += $env:VOLT_INSTALL_DIR }",
    )
}

pub(super) fn remove_from_windows_path(dir: &Path) -> Result<()> {
    update_path(dir, "$paths = @($paths | Where-Object { $_.Trim().TrimEnd('\\') -ine $env:VOLT_INSTALL_DIR.TrimEnd('\\') })")
}

fn update_path(dir: &Path, transform: &str) -> Result<()> {
    let script = format!(
        r#"
$key = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey('Environment')
try {{
    $old = $key.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
    $kind = if ($key.GetValueNames() -contains 'Path') {{ $key.GetValueKind('Path') }} else {{ [Microsoft.Win32.RegistryValueKind]::ExpandString }}
    $paths = @($old -split ';' | Where-Object {{ $_ -ne '' }})
    {transform}
    $new = $paths -join ';'
    if ($new -cne $old) {{ $key.SetValue('Path', $new, $kind) }}
}} finally {{ $key.Dispose() }}
Add-Type -Namespace Volt -Name EnvironmentBroadcast -MemberDefinition '[System.Runtime.InteropServices.DllImport("user32.dll", CharSet = System.Runtime.InteropServices.CharSet.Auto, SetLastError = true)] public static extern System.IntPtr SendMessageTimeout(System.IntPtr h, uint m, System.UIntPtr w, string l, uint f, uint t, out System.UIntPtr r);'
$result = [UIntPtr]::Zero
[Volt.EnvironmentBroadcast]::SendMessageTimeout([IntPtr]0xffff, 0x001A, [UIntPtr]::Zero, 'Environment', 2, 5000, [ref]$result) | Out-Null
"#
    );
    run_powershell(&script, Some(dir)).map(|_| ())
}

pub(super) fn register_windows_uninstall(info: &InstallInfo) -> Result<()> {
    let key = "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Volt";
    let exe_str = info.executable_path.to_string_lossy();
    let dir_str = info.install_dir.to_string_lossy();
    let icon_str = format!("{},0", exe_str);
    let uninstall_str = format!("\"{}\" --uninstall", exe_str);

    let entries = [
        (
            "DisplayName",
            "REG_SZ",
            "Volt Desktop (Native AI Coding Assistant)",
        ),
        ("DisplayVersion", "REG_SZ", env!("CARGO_PKG_VERSION")),
        ("Publisher", "REG_SZ", "Volt Native"),
        ("DisplayIcon", "REG_SZ", &icon_str),
        ("InstallLocation", "REG_SZ", &dir_str),
        ("UninstallString", "REG_SZ", &uninstall_str),
        ("NoModify", "REG_DWORD", "1"),
        ("NoRepair", "REG_DWORD", "1"),
    ];

    for (name, reg_type, value) in entries {
        let status = std::process::Command::new("reg")
            .args(["add", key, "/v", name, "/t", reg_type, "/d", value, "/f"])
            .status()
            .with_context(|| format!("写入卸载注册表值失败: {}", name))?;
        if !status.success() {
            anyhow::bail!("写入卸载注册表值失败: {} (exit: {})", name, status);
        }
    }

    Ok(())
}

pub(super) fn unregister_windows_uninstall() -> Result<()> {
    run_powershell("$path = 'Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Volt'; if ([Microsoft.Win32.Registry]::CurrentUser.GetSubKeyNames() -contains 'Software') { [Microsoft.Win32.Registry]::CurrentUser.DeleteSubKeyTree($path, $false) }", None).map(|_| ())
}

pub(super) fn schedule_self_delete(dir: &Path) -> Result<()> {
    // Wait for this process, then remove only our executable. Preserve other files.
    let script = format!(
        r#"$ErrorActionPreference = 'Stop'; try {{
        Wait-Process -Id {} -ErrorAction SilentlyContinue
        $exe = Join-Path $env:VOLT_INSTALL_DIR 'volt.exe'
        if (Test-Path -LiteralPath $exe) {{ Remove-Item -LiteralPath $exe -Force }}
        if ((Test-Path -LiteralPath $env:VOLT_INSTALL_DIR) -and @(Get-ChildItem -LiteralPath $env:VOLT_INSTALL_DIR -Force).Count -eq 0) {{ Remove-Item -LiteralPath $env:VOLT_INSTALL_DIR }}
    }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 1 }}"#,
        std::process::id()
    );
    std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .env("VOLT_INSTALL_DIR", dir)
        .spawn()
        .context("启动自删除任务失败")?;
    Ok(())
}

fn run_powershell(script: &str, dir: Option<&Path>) -> Result<String> {
    let guarded = format!("$ErrorActionPreference = 'Stop'; try {{ {script} }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 1 }}");
    let mut command = std::process::Command::new("powershell");
    command.args(["-NoProfile", "-NonInteractive", "-Command", &guarded]);
    if let Some(dir) = dir {
        command.env("VOLT_INSTALL_DIR", dir);
    }
    let output = command.output().context("无法启动 Windows 集成命令")?;
    if !output.status.success() {
        anyhow::bail!(
            "Windows integration failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::run_powershell;

    #[test]
    fn terminating_and_nonterminating_errors_fail_installation() {
        for script in [
            "throw 'installer regression'",
            "Write-Error 'installer regression'",
            "exit 3",
        ] {
            assert!(run_powershell(script, None).is_err());
        }
        assert!(run_powershell("Write-Output 'success'", None)
            .unwrap()
            .contains("success"));
    }
}
