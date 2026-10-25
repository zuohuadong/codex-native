; Volt Windows NSIS Installer Script
; Build with: makensis scripts/installer.nsi

!define PRODUCT_NAME "Volt Desktop"
!define PRODUCT_VERSION "0.1.0"
!define PRODUCT_PUBLISHER "Volt Native Team"
!define PRODUCT_EXE "volt.exe"
!define REG_UNINSTALL "Software\Microsoft\Windows\CurrentVersion\Uninstall\Volt"

SetCompressor /SOLID lzma
Unicode True
RequestExecutionLevel user ; Install to User directory without requiring UAC admin elevation

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "..\target\Volt-Setup-${PRODUCT_VERSION}-x64.exe"
InstallDir "$LOCALAPPDATA\Programs\Volt"

Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Section "MainSection" SEC01
    SetOutPath "$INSTDIR"
    File "..\target\release\volt.exe"
    
    ; Create Start Menu Shortcuts
    CreateDirectory "$SMPROGRAMS\Volt"
    CreateShortCut "$SMPROGRAMS\Volt\Volt.lnk" "$INSTDIR\${PRODUCT_EXE}" "" "$INSTDIR\${PRODUCT_EXE}" 0
    CreateShortCut "$DESKTOP\Volt.lnk" "$INSTDIR\${PRODUCT_EXE}" "" "$INSTDIR\${PRODUCT_EXE}" 0
    
    ; Create Uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"
    
    ; Write Uninstall Registry Keys (HKCU)
    WriteRegStr HKCU "${REG_UNINSTALL}" "DisplayName" "${PRODUCT_NAME}"
    WriteRegStr HKCU "${REG_UNINSTALL}" "DisplayVersion" "${PRODUCT_VERSION}"
    WriteRegStr HKCU "${REG_UNINSTALL}" "Publisher" "${PRODUCT_PUBLISHER}"
    WriteRegStr HKCU "${REG_UNINSTALL}" "DisplayIcon" "$INSTDIR\${PRODUCT_EXE},0"
    WriteRegStr HKCU "${REG_UNINSTALL}" "InstallLocation" "$INSTDIR"
    WriteRegStr HKCU "${REG_UNINSTALL}" "UninstallString" "$INSTDIR\uninstall.exe"
    WriteRegDWORD HKCU "${REG_UNINSTALL}" "NoModify" 1
    WriteRegDWORD HKCU "${REG_UNINSTALL}" "NoRepair" 1
SectionEnd

Section "Uninstall"
    ; Remove Shortcuts
    Delete "$DESKTOP\Volt.lnk"
    Delete "$SMPROGRAMS\Volt\Volt.lnk"
    RMDir "$SMPROGRAMS\Volt"
    
    ; Remove Installed Files
    Delete "$INSTDIR\${PRODUCT_EXE}"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"
    
    ; Remove Registry Entry
    DeleteRegKey HKCU "${REG_UNINSTALL}"
SectionEnd
