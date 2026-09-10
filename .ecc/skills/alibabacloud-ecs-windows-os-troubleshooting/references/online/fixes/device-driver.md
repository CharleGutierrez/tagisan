# Device Driver Diagnostic Fix

## Fix Recommendations

### Root cause: Device driver not installed (error code 28)

**Fix operation**:

```powershell
# Try to search and install driver via Windows Update
pnputil /scan-devices
# Or manually install specified driver package
pnputil /add-driver <DriverInfFile> /install
```

**Verification**:

```powershell
Get-CimInstance Win32_PnPEntity | Where-Object { $_.ConfigManagerErrorCode -ne 0 } | Select-Object Name, ConfigManagerErrorCode | Format-Table -AutoSize
```

Expected result: No abnormal devices or target device error code has cleared

**Risk notes**:

- **Session impact**: May require reboot after driver installation; reboot will interrupt all connections.
- **Persistence scope**: Permanently installed; preserved across reboots.
- **Rollback command**: `pnputil /delete-driver <DriverInfFile> /uninstall` (roll back to original driver after uninstall).
- **Note**: Installing incompatible drivers may cause BSOD; it is recommended to prioritize drivers provided by the device manufacturer.

### Root cause: Driver installation blocked by policy

**Fix operation**:

See -> [cloud-driver.md](references/online/fixes/cloud-driver.md) "Root cause: Driver installation blocked by policy" fix.

### Root cause: Driver service not started properly

**Fix operation**:

```powershell
# Start non-running driver service (replace <ServiceName> with actual service name)
Start-Service -Name '<ServiceName>' -ErrorAction Stop
```

**Verification**:

```powershell
Get-Service -Name '<ServiceName>' | Select-Object Name, Status, StartType | Format-Table -AutoSize
```

Expected result: Status is Running

**Risk notes**:

- **Session impact**: None; starting a service does not affect existing connections.
- **Persistence scope**: Service start is a temporary operation; if combined with StartupType modification it is preserved across reboots.
- **Rollback command**: `Stop-Service -Name '<ServiceName>'`
- **Note**: Before starting the driver service, confirm the driver files are intact; starting a corrupted driver may cause BSOD.
