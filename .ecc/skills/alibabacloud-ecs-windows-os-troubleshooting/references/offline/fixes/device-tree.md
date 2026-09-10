# Device Enumeration Tree Diagnostic Fix

## Fix Recommendations

### Root cause: Device instance disabled

**Fix operation**: Clear the CONFIGFLAG_DISABLED flag from ConfigFlags

```powershell
$bootLetter = '<BootLetter>'
$sysHive = "${bootLetter}:\Windows\System32\config\SYSTEM"
$exitCode = 0

# Input: list of disabled device paths found in diagnostic steps (format: Bus\Port\Instance)
$devicePaths = @('<SCSI\Disk&Ven_...\...>', '<Storage\...>')

try {
    & reg load "HKLM\_SYSTEM" $sysHive
    if ($LASTEXITCODE -ne 0) { throw "Failed to load SYSTEM hive." }

    $select = [Microsoft.Win32.Registry]::LocalMachine.OpenSubKey("_SYSTEM\Select", $false)
    $currentControlSet = "_SYSTEM\ControlSet00$($select.GetValue('Current'))"
    $select.Close(); $select.Dispose()

    $enumBase = [Microsoft.Win32.Registry]::LocalMachine.OpenSubKey(
        "$currentControlSet\Enum", $true)
    if (-not $enumBase) { throw "Cannot open Enum key" }

    foreach ($devPath in $devicePaths) {
        $subKey = $enumBase.OpenSubKey($devPath, $true)
        if ($subKey) {
            $flags = $subKey.GetValue('ConfigFlags')
            if ($flags -band 1) {
                $newFlags = $flags -band (-bnot 1)
                $subKey.SetValue('ConfigFlags', $newFlags, [Microsoft.Win32.RegistryValueKind]::DWord)
                Write-Host "Enabled device: $devPath (ConfigFlags $flags -> $newFlags)"
            }
            $subKey.Close(); $subKey.Dispose()
        } else {
            Write-Host "WARNING: Device not found: $devPath"
        }
    }
    $enumBase.Close(); $enumBase.Dispose()

    Write-Host "Device tree fix completed."
} catch {
    Write-Host "Fix device tree failed: $($_.Exception.Message)"
    $exitCode = 1
} finally {
    [System.GC]::Collect()
    [System.GC]::WaitForPendingFinalizers()
    & reg unload "HKLM\_SYSTEM"
    if ($exitCode -ne 0) { Exit $exitCode }
}
```

**Verification**: Re-read the ConfigFlags of the corresponding device path to confirm bit0 has been cleared

**Risk notes**: Enabling the wrong device may cause resource conflicts. It is recommended to only operate on SCSI/Storage class devices
