# Driver Integrity Diagnostics

## Function Description

Checks VirtIO driver installation status, version, signature, and service configuration; detects Xen driver residual conflicts; verifies binary file existence and dependency chain integrity for Boot-Start/System-Start drivers; checks disk class filter driver compliance; checks NVMe storage controller driver status; verifies boot device PCI/SCSI registry instance chain integrity.

**Input**: Boot partition drive letter, registry HIVE already loaded
**Output**: Root cause list (root_cause / severity / evidence / explanation / fix)

## Step Selection Guide

This file contains 7 diagnostic steps, **no need to execute all**. Select relevant steps based on the user's described problem:

| User Problem | Recommended Steps |
|--------------|-------------------|
| BSOD 0x7B / INACCESSIBLE_BOOT_DEVICE | Step 1 → Step 2 (VirtIO) → Step 4 (Boot-critical drivers) → Step 5 (Filter drivers) → Step 7 (Boot device instances) |
| VirtIO driver missing/version/signature issue | Step 1 → Step 2 (VirtIO) |
| Boot failure after platform migration (Xen/VMware) | Step 2 (VirtIO) → Step 3 (Xen residual) |
| BSOD after third-party security software uninstall | Step 5 (Disk class filter drivers) |
| NVMe-related BSOD (stornvme) | Step 6 (NVMe) → Step 4 (Boot-critical drivers) |
| Boot-critical driver file missing | Step 4 (Boot-critical drivers) |
| BSOD 0x7E/0x7F/0xD1 (driver exception) | Step 2 (VirtIO) → Step 4 (Boot-critical drivers) → Step 5 (Filter drivers) |
| Unknown cause / comprehensive troubleshooting | Step 1 → Step 2 → Step 3 → Step 4 → Step 5 → Step 6 → Step 7 |

## Diagnostic Steps

### Step 1: Driver Installation Policy Check

**Data Collection**:

> Collection target: Check whether system policy has disabled driver installation

```powershell

# Get OS version to determine check path
$cv = Get-ItemProperty "<SoftPath>\Microsoft\Windows NT\CurrentVersion" -ErrorAction SilentlyContinue
$major = $cv.CurrentMajorVersionNumber

# Win8+ (Major >= 6.2): DeviceInstall service parameter
$diParam = Get-ItemProperty "<CcsPath>\Services\DeviceInstall\Parameters" -ErrorAction SilentlyContinue
# Win7/2008: PlugPlay service parameter
$ppParam = Get-ItemProperty "<CcsPath>\Services\PlugPlay\Parameters" -ErrorAction SilentlyContinue

[PSCustomObject]@{
    OSMajor                          = $major
    DeviceInstallDisabled            = $diParam.DeviceInstallDisabled
    PlugPlayDeviceInstallDisabled    = $ppParam.DeviceInstallDisabled
}
```

**Analysis**:

1. OS version >= 6.2 (Win8/2012+): Check `DeviceInstallDisabled` value
2. OS version < 6.2 (Win7/2008): Check `PlugPlayDeviceInstallDisabled` value
3. Corresponding value != 0 → **Root cause**: Driver installation policy disabled, **Severity**: Warning (may affect virtualization driver hot-plug)

### Step 2: VirtIO Driver Check

> **DISM Mandatory Rules**: Part 1 calls `Get-WindowsDriver`, MUST strictly follow [dism.md](references/offline/dism.md) "DISM Mandatory Rules" — after the call, HIVE is unloaded; Part 2 needs to read the registry, so before executing Part 2, you MUST first remount the HIVE per [registry.md](references/offline/registry.md) Step 2 (the remount action is included by default between Part 1 and Part 2 in the script below).
>
> **No Substitution**: Do not substitute `reg query Services\viostor` or file existence checks for Part 1. Reason: `Get-WindowsDriver` returns complete driver package metadata (version, signature status, original inf file name), which is the only data source for detecting the "driver package installed but not properly registered to service" scenario; the registry Services key only reflects service registration status. Part 1 and Part 2 form a "package layer + service layer" dual-source cross-validation; neither can be omitted.

**Data Collection**:

> Collection target: Use DISM to check VirtIO driver package status and version (version determination is based on the best package in the driver store); verify service registration status and binary signature
>
> **Version Determination Principle**: When multiple versions of a driver package coexist, Windows PnP automatically selects the active package at boot time by Rank → Date → Version priority; which version takes effect is guaranteed by the OS internal mechanism. Therefore, **the .sys version pointed to by the service ImagePath is not checked** (offline DISM Reflect Critical only updates service configuration for BootCritical drivers; non-BootCritical driver services may still point to old .sys files, which is a transient offline state; PnP automatically reselects after first boot).

```powershell

# --- Part 1: Driver store query via DISM (ref: dism.md "Standard Disk Cache Pattern") ---
$cacheDir = Join-Path $env:SystemRoot 'Temp\diag-cache'
New-Item -ItemType Directory -Path $cacheDir -Force | Out-Null
$cacheFile = Join-Path $cacheDir 'WindowsDriver.json'
if (Test-Path $cacheFile) {
    $allDrivers = Get-Content $cacheFile -Raw | ConvertFrom-Json
} else {
    $allDrivers = Get-WindowsDriver -Path "<BootLetter>:\"
    $allDrivers | ConvertTo-Json -Depth 4 | Set-Content $cacheFile -Encoding UTF8
}

$virtioInfs = @('viostor','netkvm','balloon','pvpanic','vioser','fwcfg')
$virtioDrivers = $allDrivers | Where-Object {
    $infName = (Split-Path $_.OriginalFileName -Leaf) -replace '\.inf$',''
    $infName -in $virtioInfs
}
$virtioDrivers | Format-List Driver, OriginalFileName, ProviderName, Date, Version

# --- Part 1b: SelectBestDriver per INF (PnP boot selection: Rank -> Date -> Version) ---
# Same-INF packages share Rank in practice, so select by Date (newest) then Version (highest).
# The selected package represents the version PnP will bind after first boot.
$virtioDrivers | Group-Object { (Split-Path $_.OriginalFileName -Leaf) } | ForEach-Object {
    $best = $_.Group | Sort-Object Date, Version -Descending | Select-Object -First 1
    [PSCustomObject]@{ Inf = $_.Name; BestVersion = $best.Version; BestDate = $best.Date }
} | Format-Table -AutoSize

# --- Part 2: Service registry + binary verification ---
$driverMap = @{
    'viostor'      = 'viostor'
    'netkvm'       = 'netkvm'
    'balloon'      = 'balloon'
    'pvpanic'      = 'pvpanic'
    'vioser'       = 'virtioserial'
    # Win10+: 'fwcfg' = 'fwcfg'
}

$driverMap.Keys | ForEach-Object {
    $name = $_
    $svcName = $driverMap[$name]
    $svc = Get-ItemProperty "<CcsPath>\Services\$svcName" -ErrorAction SilentlyContinue
    $imgPath = $svc.ImagePath
    # Resolve ImagePath to offline absolute path
    if ($imgPath) {
        $imgPath = $imgPath.Trim('"')
        $imgPath = $imgPath -replace '/', '\'
        # Strip leading drive letter (e.g. C:\Windows\... → Windows\...)
        if ($imgPath -match '[A-Za-z]:\\(.+)') { $imgPath = $Matches[1] }
        # Strip trailing arguments (e.g. .sys /s → .sys)
        if ($imgPath -match '^([^\s]+)') { $imgPath = $Matches[1] }
        # Resolve %SystemRoot% or \SystemRoot\ prefix
        if ($imgPath -match '(?i).*%?SystemRoot%?\\(.+)') { $imgPath = $Matches[1] }
        # Prepend Windows\ for bare System32 paths (e.g. System32\drivers\viostor.sys → Windows\System32\drivers\viostor.sys)
        if ($imgPath -match '(?i)^System32') { $imgPath = "Windows\$imgPath" }
        $imgPath = "<BootLetter>:\$imgPath"
    }
    $exists = if ($imgPath) { Test-Path $imgPath -ErrorAction SilentlyContinue } else { $false }
    # NOTE: the .sys version at ImagePath is NOT collected — it may be a stale
    # offline state and MUST NOT be used for version judgement (see analysis note above).
    $sig = if ($exists) { Get-AuthenticodeSignature $imgPath } else { $null }
    [PSCustomObject]@{
        Driver   = $name
        Service  = $svcName
        Start    = $svc.Start
        Exists   = $exists
        Signed   = if ($sig) { $sig.Status } else { 'N/A' }
    }
} | Format-Table -AutoSize
```

**Analysis**:

Cross-reference Part 1 (driver store) and Part 2 (service registration) results:

1. No corresponding .inf in driver store and service does not exist:
   - viostor missing → **Root cause**: VirtIO storage driver not installed, **Severity**: Critical
   - netkvm missing → **Root cause**: VirtIO network driver not installed, **Severity**: Warning
   - Others missing → **Severity**: Warning
2. .inf exists in driver store but service does not exist or binary is missing → driver package installed but service registration is abnormal, **Severity**: Critical
3. Service disabled (Start == 4):
   - → **Root cause**: VirtIO driver service disabled, **Severity**: Critical (especially viostor)
4. Invalid signature (Status ≠ Valid):
   - → **Root cause**: VirtIO driver signature invalid, **Severity**: Warning
5. Version outdated determination: Based on the best package per INF selected by Part 1b SelectBestDriver — simulating the PnP boot-time driver selection algorithm (Rank → Date → Version priority; same-INF multi-package Rank is usually the same, so selection is by **newest Date → highest Version**), the best package represents the "version that will actually run after boot". Note that **you cannot simply take the package with the highest version number** (a higher-version package may have worse Rank or older Date, and PnP will not select it), nor can you use the .sys version pointed to by the service ImagePath (offline DISM Reflect Critical only updates service configuration for BootCritical drivers; non-BootCritical driver services like netkvm still point to old .sys files, which is a transient offline state; PnP automatically reselects after first boot — using the service binary version for determination would cause false positives for non-BootCritical drivers). Parse the 4th segment of the best package Version (e.g., 58200 in `100.85.104.58200`), **4th segment < 58017 is considered outdated**:
   - → **Root cause**: VirtIO driver version outdated, **Severity**: Warning

### Step 3: Xen Driver Residual Check

**Data Collection**:

> Collection target: Detect whether Xen driver residuals exist (may conflict with VirtIO)

```powershell

$xenDrivers = @('XenPCI', 'xenvbd', 'xennet', 'xenscsi', 'xenstub')
$xenResults = foreach ($drv in $xenDrivers) {
    $svc = Get-ItemProperty "<CcsPath>\Services\$drv" -ErrorAction SilentlyContinue
    if ($svc) {
        [PSCustomObject]@{
            Name  = $drv
            Start = $svc.Start
            Type  = $svc.Type
        }
    }
}
$xenResults | Format-Table -AutoSize

# Check XenPCI hide_devices parameter
$xenPci = Get-ItemProperty "<CcsPath>\Services\XenPCI\Parameters" -ErrorAction SilentlyContinue
if ($xenPci) {
    $xenPci | Select-Object hide_devices
}
```

**Analysis**:

1. Xen driver residual and XenPCI\Parameters\hide_devices is non-empty:
   - → **Root cause**: Xen driver residual (hide_devices in effect will hide VirtIO devices), **Severity**: Critical
2. Xen driver residual but no hide_devices → **Severity**: Warning (potential conflict)

### Step 4: Boot-Critical Driver Integrity Check

**Data Collection**:

> Collection target: Check binary files and dependencies of Boot-Start (Start=0) and System-Start (Start=1) drivers

```powershell

$services = Get-ChildItem "<CcsPath>\Services" -ErrorAction SilentlyContinue
$bootDrivers = @()
foreach ($svc in $services) {
    $props = Get-ItemProperty $svc.PSPath -ErrorAction SilentlyContinue
    # Type contains SERVICE_DRIVER (0x1 or 0x2), Start is 0(Boot) or 1(System)
    if (($props.Type -band 0x3) -ne 0 -and $props.Start -le 1) {
        $imgPath = $props.ImagePath
        if ($imgPath) {
            $imgPath = $imgPath.Trim('"')
            $imgPath = $imgPath -replace '/', '\'
            # Strip leading drive letter (e.g. C:\Windows\... → Windows\...)
            if ($imgPath -match '[A-Za-z]:\\(.+)') { $imgPath = $Matches[1] }
            # Strip trailing arguments (e.g. .sys /s → .sys)
            if ($imgPath -match '^([^\s]+)') { $imgPath = $Matches[1] }
            # Resolve %SystemRoot% or \SystemRoot\ prefix
            if ($imgPath -match '(?i).*%?SystemRoot%?\\(.+)') { $imgPath = $Matches[1] }
            # Prepend Windows\ for bare System32 paths
            if ($imgPath -match '(?i)^System32') { $imgPath = "Windows\$imgPath" }
            $imgPath = "<BootLetter>:\$imgPath"
        }
        $exists = if ($imgPath) { Test-Path $imgPath -ErrorAction SilentlyContinue } else { $false }
        $bootDrivers += [PSCustomObject]@{
            Name         = $svc.PSChildName
            Start        = $props.Start
            ErrorControl = $props.ErrorControl
            ImagePath    = $imgPath
            Exists       = $exists
            DependOnService = $props.DependOnService
        }
    }
}
# Output drivers with missing files
$bootDrivers | Where-Object { -not $_.Exists -and $_.ImagePath } | Format-List Name, Start, ErrorControl, ImagePath
# Output dependency check
$bootDrivers | Where-Object { $_.DependOnService } | ForEach-Object {
    $depMissing = @()
    foreach ($dep in $_.DependOnService) {
        $depSvc = Get-ItemProperty "<CcsPath>\Services\$dep" -ErrorAction SilentlyContinue
        if (-not $depSvc) { $depMissing += $dep }
    }
    if ($depMissing.Count -gt 0) {
        [PSCustomObject]@{ Name = $_.Name; MissingDeps = ($depMissing -join ',') }
    }
} | Format-List
```

**Analysis**:

1. Boot-Start/System-Start driver's ImagePath file does not exist:
   - ErrorControl = 3 (Critical) → **Root cause**: Boot-critical driver file missing (ErrorControl=Critical), **Severity**: Critical
   - ErrorControl = 2 (Severe) → **Severity**: Warning
   - ErrorControl = 1 (Normal) → **Severity**: Warning
2. Dependency service does not exist (service name in DependOnService not found under Services):
   - → **Root cause**: Boot driver dependency chain broken, **Severity**: Warning
3. StartOverride check: If driver is overridden to disabled in HardwareConfig → **Severity**: Warning

### Step 5: Disk Class Filter Driver Check

**Data Collection**:

> Collection target: Check upper/lower filter drivers for DiskDrive and Volume device classes

```powershell

$classGuids = @{
    'DiskDrive' = '{4d36e967-e325-11ce-bfc1-08002be10318}'
    'Volume'    = '{71a27cdd-812a-11d0-bec7-08002be2092f}'
}

$filterResults = foreach ($class in $classGuids.Keys) {
    $guidStr = $classGuids[$class]
    $path = "<CcsPath>\Control\Class\$guidStr"
    $props = Get-ItemProperty $path -ErrorAction SilentlyContinue
    [PSCustomObject]@{
        Class       = $class
        UpperFilters = $props.UpperFilters
        LowerFilters = $props.LowerFilters
    }
}
$filterResults | Format-Table -AutoSize
```

**Analysis**:

Standard value reference:
- **DiskDrive**: Pre-Win8 UpperFilters=[PartMgr]; Win8+ UpperFilters=[PartMgr], LowerFilters=[EhStorClass]
- **Volume**: Win10+ UpperFilters=[volsnap]

Determination logic:
1. Non-standard value appears in filter driver list → Check whether the corresponding service's ImagePath exists
2. Non-standard filter driver's service file does not exist → **Root cause**: Disk class filter driver residual (service file missing), **Severity**: Critical
3. Non-standard filter driver's service file exists but may be security software → **Severity**: Warning

### Step 6: NVMe Storage Controller Check

**Data Collection**:

> Collection target: Check registry and binary status of stornvme service (Windows native NVMe driver)

```powershell

$nvmeSvc = Get-ItemProperty "<CcsPath>\Services\stornvme" -ErrorAction SilentlyContinue
if ($nvmeSvc) {
    $imgPath = $nvmeSvc.ImagePath
    # Resolve ImagePath to offline absolute path
    if ($imgPath) {
        $imgPath = $imgPath.Trim('"')
        $imgPath = $imgPath -replace '/', '\'
        # Strip leading drive letter (e.g. C:\Windows\... → Windows\...)
        if ($imgPath -match '[A-Za-z]:\\(.+)') { $imgPath = $Matches[1] }
        # Strip trailing arguments (e.g. .sys /s → .sys)
        if ($imgPath -match '^([^\s]+)') { $imgPath = $Matches[1] }
        # Resolve %SystemRoot% or \SystemRoot\ prefix
        if ($imgPath -match '(?i).*%?SystemRoot%?\\(.+)') { $imgPath = $Matches[1] }
        # Prepend Windows\ for bare System32 paths
        if ($imgPath -match '(?i)^System32') { $imgPath = "Windows\$imgPath" }
        $imgPath = "<BootLetter>:\$imgPath"
    }
    $exists = if ($imgPath) { Test-Path $imgPath -ErrorAction SilentlyContinue } else { $false }
    [PSCustomObject]@{
        Name      = 'stornvme'
        Start     = $nvmeSvc.Start
        ImagePath = $imgPath
        Exists    = $exists
    }
} else {
    [PSCustomObject]@{ Name = 'stornvme'; Start = 'N/A'; ImagePath = 'N/A'; Exists = $false }
}
```

**Analysis**:

1. stornvme service does not exist (registry key missing) → **Root cause**: NVMe storage controller driver not registered, **Severity**: Critical
2. stornvme service Start == 4 (disabled) → **Root cause**: NVMe storage controller driver disabled, **Severity**: Critical
3. stornvme binary file does not exist → **Root cause**: NVMe storage controller driver file missing, **Severity**: Critical
4. If any of the above is hit and the disk BusType is NVMe, the system will STOP 0x7B

### Step 7: Boot Device Registry Instance Check

**Data Collection**:

> Collection target: Locate the PCI → SCSI registry instance chain corresponding to the boot device and verify its integrity

```powershell

$enumPath = "<CcsPath>\Enum"
$scsiAdapterGuid = '{4d36e97b-e325-11ce-bfc1-08002be10318}'

# Iterate PCI devices, find ClassGUID=SCSIAdapter with minimum UINumber (Boot device)
$pciPath = "${enumPath}\PCI"
$bootPci = $null
$minUI = [uint32]::MaxValue
Get-ChildItem $pciPath -ErrorAction SilentlyContinue | ForEach-Object {
    Get-ChildItem $_.PSPath -ErrorAction SilentlyContinue | ForEach-Object {
        $props = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue
        if ($props.ClassGUID -eq $scsiAdapterGuid) {
            $ui = [uint32]($props.UINumber)
            if ($ui -lt $minUI) {
                $minUI = $ui
                $bootPci = [PSCustomObject]@{
                    Path = $_.PSPath
                    ParentIdPrefix = $props.ParentIdPrefix
                    ConfigFlags = $props.ConfigFlags
                    UINumber = $ui
                }
            }
        }
    }
}

# Find matching instance in SCSI enumeration by ParentIdPrefix
$bootScsi = $null
$scsiPath = "${enumPath}\SCSI"
if ($bootPci -and $bootPci.ParentIdPrefix) {
    $prefix = $bootPci.ParentIdPrefix.ToLower()
    Get-ChildItem $scsiPath -ErrorAction SilentlyContinue | ForEach-Object {
        Get-ChildItem $_.PSPath -ErrorAction SilentlyContinue | ForEach-Object {
            if ($_.PSChildName.ToLower().Contains($prefix)) {
                $props = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue
                $bootScsi = [PSCustomObject]@{
                    Path = $_.PSPath
                    ConfigFlags = $props.ConfigFlags
                }
            }
        }
    }
}

[PSCustomObject]@{
    PCI_Found = ($null -ne $bootPci)
    PCI_Disabled = if ($bootPci) { $bootPci.ConfigFlags -eq 1 } else { $false }
    SCSI_Found = ($null -ne $bootScsi)
    SCSI_Disabled = if ($bootScsi) { $bootScsi.ConfigFlags -eq 1 } else { $false }
}
```

**Analysis**:

1. PCI instance does not exist (no PCI device with ClassGUID=SCSIAdapter found) → **Root cause**: Boot device PCI registry instance missing, **Severity**: Critical
2. SCSI instance does not exist (ParentIdPrefix cannot match SCSI enumeration) → **Root cause**: Boot device SCSI registry instance missing, **Severity**: Critical
3. PCI/SCSI instance ConfigFlags == 1 (CONFIGFLAG_DISABLED) → **Root cause**: Boot device registry instance disabled, **Severity**: Critical
4. All of the above can cause STOP 0x7B (INACCESSIBLE_BOOT_DEVICE)

## Cross-References

| Type | Trigger Condition | Target |
|------|-------------------|--------|
| Prerequisite | Registry HIVE must be loaded via the fixed prerequisite chain | — |
| Prerequisite | Reload registry after DISM driver addition | → [dism.md](references/offline/dism.md) (HIVE reload rules) |
| Conditional jump | VirtIO driver missing and needs to be added | → [dism.md](references/offline/dism.md) (Driver add) |
| Conditional jump | Xen driver residual affecting network | → [network.md](references/offline/network.md) Step 3 |
| Conditional jump | Network class filter driver residual | → [network.md](references/offline/network.md) Step 4 |


## Fix Recommendations

The fix solutions corresponding to the root causes confirmed in this file can be found in [driver.md](references/offline/fixes/driver.md).
