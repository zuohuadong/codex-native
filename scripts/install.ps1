<#
.SYNOPSIS
    Volt Desktop (Native AI Coding Assistant) Windows One-Click Installer
.DESCRIPTION
    Installs Volt into %LOCALAPPDATA%\Programs\Volt, creates Desktop and Start Menu shortcuts,
    configures user PATH, and registers the uninstaller into Windows Add/Remove Programs.
    No Administrator (UAC) permissions required!
.EXAMPLE
    irm https://raw.githubusercontent.com/zuohuadong/codex-native/main/scripts/install.ps1 | iex
#>

[CmdletBinding()]
param(
    [string]$Version = "latest",
    [switch]$NoDesktopShortcut = $false,
    [switch]$NoStartMenuShortcut = $false,
    [switch]$NoPath = $false,
    [switch]$NoLaunch = $false,
    [string]$LocalExePath = ""
)

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "      Volt Native AI Coding Assistant - Windows 安装     " -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host ""

# 1. Determine Target Installation Path
$AppName = "Volt"
$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\$AppName"
$TargetExe = Join-Path $InstallDir "volt.exe"

Write-Host ">> 目标安装目录: $InstallDir" -ForegroundColor Gray

# 2. Prepare Source Binary
if (-not [string]::IsNullOrWhiteSpace($LocalExePath) -and -not (Test-Path -LiteralPath $LocalExePath -PathType Leaf)) {
    throw "Local executable does not exist: $LocalExePath"
}
$TempExe = $null
$TempChecksum = $null
try {
if (-not [string]::IsNullOrWhiteSpace($LocalExePath)) {
    $SourceExe = (Resolve-Path $LocalExePath).Path
    Write-Host ">> 使用本地预编译文件: $SourceExe" -ForegroundColor Green
} elseif (![string]::IsNullOrEmpty($PSScriptRoot) -and (Test-Path (Join-Path $PSScriptRoot "..\target\release\volt.exe"))) {
    $SourceExe = (Resolve-Path (Join-Path $PSScriptRoot "..\target\release\volt.exe")).Path
    Write-Host ">> 使用工作区构建文件: $SourceExe" -ForegroundColor Green
} elseif (![string]::IsNullOrEmpty($PSScriptRoot) -and (Test-Path (Join-Path $PSScriptRoot "volt.exe"))) {
    $SourceExe = (Resolve-Path (Join-Path $PSScriptRoot "volt.exe")).Path
    Write-Host ">> 使用同级目录二进制文件: $SourceExe" -ForegroundColor Green
} else {
    # Download from GitHub Releases
    Write-Host ">> 正在从远程发布源获取最新 Windows x64 二进制文件..." -ForegroundColor Yellow
    $Tag = $Version
    if ($Version -eq "latest") {
        $Release = Invoke-RestMethod -Uri "https://api.github.com/repos/zuohuadong/codex-native/releases/latest"
        $Tag = $Release.tag_name
    } elseif (-not $Tag.StartsWith('v')) { $Tag = "v$Tag" }
    if ($Tag -notmatch '^v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$') { throw "Invalid release tag: $Tag" }
    $ReleaseBaseUrl = "https://github.com/zuohuadong/codex-native/releases/download/$Tag"
    $DownloadUrl = "$ReleaseBaseUrl/volt-windows-x64.exe"
    $ChecksumUrl = "$ReleaseBaseUrl/volt-windows-x64.exe.sha256"
    $TempExe = Join-Path $env:TEMP "volt-setup-$([guid]::NewGuid().ToString('N')).exe"
    $TempChecksum = "$TempExe.sha256"
    
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempExe -UseBasicParsing
        Invoke-WebRequest -Uri $ChecksumUrl -OutFile $TempChecksum -UseBasicParsing
        $ExpectedHash = (Get-Content -Path $TempChecksum -Raw).Trim().Split(' ')[0].ToLowerInvariant()
        $ActualHash = (Get-FileHash -Path $TempExe -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($ExpectedHash -notmatch '^[0-9a-f]{64}$' -or $ExpectedHash -ne $ActualHash) {
            throw "下载文件 SHA256 校验失败"
        }
        $SourceExe = $TempExe
    } catch {
        Write-Warning "从 GitHub 下载失败: $_"
        Write-Host "若未联网或处于内网，请先执行 `cargo build --release -p pi-desktop` 并在本地运行本脚本。" -ForegroundColor Yellow
        throw
    }
}

# 3. Create Install Directory & Copy Executable
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

Write-Host ">> 正在部署 volt.exe 到系统用户程序库..." -ForegroundColor Cyan
if ([IO.Path]::GetFullPath($SourceExe) -ine [IO.Path]::GetFullPath($TargetExe)) {
    Copy-Item -LiteralPath $SourceExe -Destination $TargetExe -Force
}

# 4. Create Desktop Shortcut
if (-not $NoDesktopShortcut) {
    $DesktopPath = [Environment]::GetFolderPath("Desktop")
    $ShortcutFile = Join-Path $DesktopPath "$AppName.lnk"
    
    Write-Host ">> 创建桌面快捷方式: $ShortcutFile" -ForegroundColor Cyan
    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($ShortcutFile)
    $Shortcut.TargetPath = $TargetExe
    $Shortcut.WorkingDirectory = $InstallDir
    $Shortcut.Description = "Volt Native AI Coding Assistant"
    $Shortcut.Save()
}

# 5. Create Start Menu Shortcut
if (-not $NoStartMenuShortcut) {
    $StartMenuPath = [Environment]::GetFolderPath("Programs")
    $StartMenuFile = Join-Path $StartMenuPath "$AppName.lnk"
    
    Write-Host ">> 创建开始菜单快捷方式: $StartMenuFile" -ForegroundColor Cyan
    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($StartMenuFile)
    $Shortcut.TargetPath = $TargetExe
    $Shortcut.WorkingDirectory = $InstallDir
    $Shortcut.Description = "Volt Native AI Coding Assistant"
    $Shortcut.Save()
}

# 6. Add to User Environment PATH
if (-not $NoPath) {
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $Paths = $UserPath -split ';' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
    
    if ($Paths -notcontains $InstallDir) {
        Write-Host ">> 配置用户环境变量 PATH..." -ForegroundColor Cyan
        $NewPath = if ([string]::IsNullOrWhiteSpace($UserPath)) { $InstallDir } else { "$UserPath;$InstallDir" }
        [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
        
        # Broadcast environment change
        if (-not ([System.Management.Automation.PSTypeName]'Win32.Env').Type) {
            Add-Type -Namespace Win32 -Name Env -MemberDefinition @"
                [System.Runtime.InteropServices.DllImport("user32.dll", SetLastError = true, CharSet = System.Runtime.InteropServices.CharSet.Auto)]
                public static extern System.IntPtr SendMessageTimeout(
                    System.IntPtr hWnd, uint Msg, System.UIntPtr wParam, string lParam,
                    uint fuFlags, uint uTimeout, out System.UIntPtr lpdwResult);
"@
        }
        $result = [System.UIntPtr]::Zero
        [Win32.Env]::SendMessageTimeout([System.IntPtr]0xffff, 0x001A, [System.UIntPtr]::Zero, "Environment", 2, 5000, [ref]$result) | Out-Null
    } else {
        Write-Host ">> 用户 PATH 中已存在 Volt，跳过添加。" -ForegroundColor Gray
    }
}

# 7. Register in Windows Add/Remove Programs
Write-Host ">> 向 Windows 控制面板注册卸载信息..." -ForegroundColor Cyan
$UninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$AppName"
if (-not (Test-Path $UninstallKey)) {
    New-Item -Path $UninstallKey -Force | Out-Null
}

Set-ItemProperty -Path $UninstallKey -Name "DisplayName" -Value "Volt Desktop (Native AI Coding Assistant)"
Set-ItemProperty -Path $UninstallKey -Name "DisplayVersion" -Value "0.1.0"
Set-ItemProperty -Path $UninstallKey -Name "Publisher" -Value "Volt Team"
Set-ItemProperty -Path $UninstallKey -Name "DisplayIcon" -Value "$TargetExe,0"
Set-ItemProperty -Path $UninstallKey -Name "InstallLocation" -Value $InstallDir
Set-ItemProperty -Path $UninstallKey -Name "UninstallString" -Value "`"$TargetExe`" --uninstall"
Set-ItemProperty -Path $UninstallKey -Name "NoModify" -Value 1 -Type DWord
Set-ItemProperty -Path $UninstallKey -Name "NoRepair" -Value 1 -Type DWord

Write-Host ""
Write-Host "========================================================" -ForegroundColor Green
Write-Host "  ✓ Volt Windows 自安装已全部就绪！                     " -ForegroundColor Green
Write-Host "========================================================" -ForegroundColor Green
if (-not $NoDesktopShortcut) { Write-Host "· 桌面快捷方式: 已生成" -ForegroundColor White }
if (-not $NoStartMenuShortcut) { Write-Host "· 开始菜单快捷方式已生成" -ForegroundColor White }
if (-not $NoPath) { Write-Host "· 用户 PATH 已配置" -ForegroundColor White }
Write-Host ""

# 8. Auto-Launch Application
if (-not $NoLaunch) {
    Write-Host ">> 正在启动 Volt..." -ForegroundColor Cyan
    Start-Process -FilePath $TargetExe
}

} finally {
    foreach ($TempPath in @($TempExe, $TempChecksum)) {
        if ($TempPath -and (Test-Path -LiteralPath $TempPath)) { Remove-Item -LiteralPath $TempPath -Force }
    }
}
