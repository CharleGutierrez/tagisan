# Registry HIVE Loading and Integrity Verification

## Function Description

Loads the target system's registry HIVE files offline and verifies file existence and integrity. This file is a prerequisite for all subsequent troubleshooting files that need to access the registry.

**Input**: Boot partition drive letter
**Output**: Root cause list (root_cause / severity / evidence / explanation / fix) + loaded HIVE path mapping

## Step Selection Guide

All steps in this file **must be executed in sequence**.

## Diagnostic Steps

### Step 1: HIVE File Existence Check

**Data Collection**:

> Collection target: Check whether all registry HIVE files that need to be loaded exist

```powershell
$bootLetter = '<BootLetter>'
# HIVE file list (shared by Step 1/Step 2; this troubleshooting file requires all steps to be executed in sequence)
$files = @(
    "${bootLetter}:\Windows\System32\config\SYSTEM",
    "${bootLetter}:\Windows\System32\config\SOFTWARE",
    "${bootLetter}:\Windows\System32\config\SAM",
    "${bootLetter}:\Windows\System32\config\SECURITY",
    "${bootLetter}:\Windows\System32\config\DRIVERS",
    "${bootLetter}:\Windows\System32\config\DEFAULT",
    "${bootLetter}:\Windows\System32\config\COMPONENTS",
    "${bootLetter}:\Users\Default\NTUSER.DAT",
    "${bootLetter}:\Windows\System32\SMI\Store\Machine\SCHEMA.DAT"
)
$files | ForEach-Object {
    $f = $_
    $info = Get-Item $f -Force -ErrorAction SilentlyContinue
    [PSCustomObject]@{
        Path = $f
        Exists = ($null -ne $info)
        Size = if ($info) { $info.Length } else { 0 }
        LastWrite = if ($info) { $info.LastWriteTime } else { $null }
    }
} | Format-List
```

**Analysis**:

1. Check file existence:
   - Normal: All files exist and Size > 0
   - Abnormal: Any of SYSTEM/SOFTWARE/SAM/SECURITY missing → **Root cause**: Critical registry HIVE file missing, **Severity**: Critical
   - DRIVERS file only exists on Server 2012+; missing on lower versions can be ignored
   - Size > 512MB → **Root cause**: Registry HIVE file too large, **Severity**: Warning

### Step 2: HIVE Loading

**Data Collection**:

> Collection target: Load all HIVE files into HKLM using the GUID path format

#### Mount Path Format (Key Constraint)

```
reg load "HKLM\{bf1a281b-ad7b-4476-ac95-f47682990ce7}<Absolute path to HIVE file, backslashes replaced with forward slashes>" <HIVE file path>
```

Example (boot partition drive letter is E:):
```
reg load "HKLM\{bf1a281b-ad7b-4476-ac95-f47682990ce7}E:/Windows/System32/config/SYSTEM" E:\Windows\System32\config\SYSTEM
```

The mount path format is fixed as `GUID + absolute path to HIVE file (backslashes replaced with forward slashes)`, where the GUID is the fixed value `{bf1a281b-ad7b-4476-ac95-f47682990ce7}`, and neither part may be modified. This format is also used internally by the DISM module; path inconsistency will cause DISM operations to fail (see dism.md for details).

Unload command:
```
reg unload "HKLM\{bf1a281b-ad7b-4476-ac95-f47682990ce7}E:/Windows/System32/config/SYSTEM"
```

#### Pre-check

Before executing the load, first check whether the above GUID path already exists:
- HIVE already loaded to the expected GUID path → skip loading, directly use the loaded path
- HIVE already loaded but path does not match expectations (e.g., loaded to a different path by another tool or manual operation) → **report error and terminate the entire diagnostic flow**, explain the path conflict reason

#### Load Script

```powershell
# Reuse $bootLetter and $files defined in Step 1 (this troubleshooting file requires Step 1→Step 2 to be executed in sequence)
$guid = '{bf1a281b-ad7b-4476-ac95-f47682990ce7}'
# Note: foreach statement cannot directly follow a pipeline; must assign first then output (see WORKFLOW-GUIDE.md PowerShell Collection Script Rules)
$loaded = foreach ($f in $files) {
    if (Test-Path $f) {
        $loadPath = $guid + ($f -replace '\\','/')
        $result = reg load "HKLM\$loadPath" $f 2>&1
        [PSCustomObject]@{ File = (Split-Path $f -Leaf); LoadPath = $loadPath; Result = ($result -join ' ').Trim() }
    }
}
$loaded | Format-List
```

**Analysis**:

1. Check load result:
   - Normal: All existing files loaded successfully
   - Abnormal: Load failure → **Root cause**: Registry HIVE file corrupted (cannot be mounted), **Severity**: Critical
   - NTUSER.DAT load failure can continue after backing up the original file (does not block subsequent diagnostics)
2. **Cases where HIVE files may not exist** (not considered abnormal, just skip):
   - `DRIVERS`: Only exists on Windows 8 / Server 2012 and above; does not exist in Windows 7 / Server 2008 R2 images
   - `BBI`: Only exists on Windows 10 1607+
   - `COMPONENTS`: May not be generated yet after Sysprep or during OOBE phase
   - During the load phase, skip missing files with `Test-Path`; do not interrupt the diagnostic flow because of this

**[CTX] Session Memory Backfill** (not displayed to the user): After completing this step, the model MUST remember the literal form of the following placeholders (all prefixed with the loaded GUID path):

| Placeholder | Value Template |
|-------------|----------------|
| `<SysPath>` | `HKLM:\{bf1a281b-ad7b-4476-ac95-f47682990ce7}<BootLetter>:/Windows/System32/config/SYSTEM` |
| `<SoftPath>` | `HKLM:\{bf1a281b-ad7b-4476-ac95-f47682990ce7}<BootLetter>:/Windows/System32/config/SOFTWARE` |
| `<BcdPath>` | UEFI: `<SystemLetter>:\EFI\Microsoft\Boot\BCD`; BIOS: `<SystemLetter>:\Boot\BCD` |

When generating subsequent scripts, you MUST first replace `<BootLetter>` / `<SystemLetter>` literals from session memory into the templates above, then execute.

### Step 3: HIVE Integrity Verification

**Data Collection**:

> Collection target: Verify each HIVE's data integrity by reading key subkeys

```powershell
$guid = '{bf1a281b-ad7b-4476-ac95-f47682990ce7}'
$bootLetter = '<BootLetter>'
$configBase = "HKLM:\${guid}${bootLetter}:/Windows/System32/config"
$sysPath = "${configBase}/SYSTEM"
$softPath = "${configBase}/SOFTWARE"
$samPath = "${configBase}/SAM"
$secPath = "${configBase}/SECURITY"
$drvPath = "${configBase}/DRIVERS"
$results = @()

# SYSTEM: Select → ControlSet → Enum → Environment
$select = Get-ItemProperty "${sysPath}\Select" -ErrorAction SilentlyContinue
$csName = "ControlSet00$($select.Current)"
$results += [PSCustomObject]@{ Hive='SYSTEM'; Test='Select\Current'; Result=if($select.Current){"OK ($csName)"}else{'MISSING'} }
$results += [PSCustomObject]@{ Hive='SYSTEM'; Test="${csName}\Enum"; Result=if(Test-Path "${sysPath}\${csName}\Enum"){'OK'}else{'MISSING'} }
$envPath = Get-ItemProperty "${sysPath}\${csName}\Control\Session Manager\Environment" -Name Path -ErrorAction SilentlyContinue
$results += [PSCustomObject]@{ Hive='SYSTEM'; Test='Environment\Path'; Result=if($envPath.Path){'OK'}else{'MISSING'} }

# SOFTWARE: CurrentVersion + CBS store
$cv = Get-ItemProperty "${softPath}\Microsoft\Windows NT\CurrentVersion" -ErrorAction SilentlyContinue
$results += [PSCustomObject]@{ Hive='SOFTWARE'; Test='CurrentVersion'; Result=if($cv.ProductName){"OK ($($cv.ProductName))"}else{'MISSING'} }
$cbsExists = Test-Path "${softPath}\Microsoft\Windows\CurrentVersion\Component Based Servicing"
$results += [PSCustomObject]@{ Hive='SOFTWARE'; Test='CBS Store'; Result=if($cbsExists){'OK'}else{'MISSING'} }

# SAM: Domains\Account\Users (may require SYSTEM privilege)
try {
    $samUsers = Get-ChildItem "${samPath}\SAM\Domains\Account\Users" -ErrorAction Stop
    $results += [PSCustomObject]@{ Hive='SAM'; Test='Domains\Account\Users'; Result="OK ($($samUsers.Count) entries)" }
} catch [System.Security.SecurityException],[System.UnauthorizedAccessException] {
    $results += [PSCustomObject]@{ Hive='SAM'; Test='Domains\Account\Users'; Result='ACCESS_DENIED (skip)' }
} catch {
    $results += [PSCustomObject]@{ Hive='SAM'; Test='Domains\Account\Users'; Result='MISSING' }
}

# SECURITY: Policy\Accounts - check expected SIDs (may require SYSTEM privilege)
try {
    $accounts = (Get-ChildItem "${secPath}\Policy\Accounts" -ErrorAction Stop).PSChildName
    $expectedSids = @('S-1-1-0','S-1-5-19','S-1-5-20','S-1-5-6')
    $missingSids = $expectedSids | Where-Object { $_ -notin $accounts }
    $results += [PSCustomObject]@{ Hive='SECURITY'; Test='Policy\Accounts'; Result=if($missingSids.Count -eq 0){'OK'}else{"MISSING SIDs: $($missingSids -join ', ')"} }
} catch [System.Security.SecurityException],[System.UnauthorizedAccessException] {
    $results += [PSCustomObject]@{ Hive='SECURITY'; Test='Policy\Accounts'; Result='ACCESS_DENIED (skip)' }
} catch {
    $results += [PSCustomObject]@{ Hive='SECURITY'; Test='Policy\Accounts'; Result='MISSING' }
}

# DRIVERS: DriverDatabase (Server 2012+ only)
if (Test-Path $drvPath) {
    $drvDb = Get-ChildItem "${drvPath}\DriverDatabase" -ErrorAction SilentlyContinue
    $results += [PSCustomObject]@{ Hive='DRIVERS'; Test='DriverDatabase'; Result=if($drvDb.Count -gt 0){"OK ($($drvDb.Count) subkeys)"}else{'MISSING'} }
}

$results | Format-Table Hive, Test, Result -AutoSize

# [CTX] Session memory backfill: output is for model parsing only, not directly displayed to the user
Write-Host "[CTX] CsName=$csName"
Write-Host "[CTX] CcsPath=$($sysPath)\$csName"
```

**Analysis**:

1. SYSTEM integrity:
   - Select key does not exist or Current value is empty → SYSTEM HIVE corrupted
   - ControlSet00X\Enum does not exist → device enumeration data lost
   - Environment\Path does not exist → environment variable data lost
2. SOFTWARE integrity:
   - CurrentVersion does not exist or no ProductName → SOFTWARE HIVE corrupted
   - CBS Store does not exist → Component Based Servicing data lost, may affect Windows Update diagnostics
3. SAM integrity:
   - Users subkey cannot be enumerated → SAM HIVE corrupted
   - ACCESS_DENIED → insufficient permissions, not considered corruption (normal when not running with SYSTEM privileges)
4. SECURITY integrity:
   - Policy\Accounts missing expected SIDs (S-1-1-0/S-1-5-19/S-1-5-20/S-1-5-6) → SECURITY HIVE corrupted
   - ACCESS_DENIED → insufficient permissions, not considered corruption
5. DRIVERS integrity (only exists on Server 2012+):
   - DriverDatabase has no subkeys → DRIVERS HIVE corrupted
6. Any critical subkey result is MISSING → **Root cause**: Registry HIVE data corrupted (some subkeys inaccessible), **Severity**: Critical

**[CTX] Session Memory Backfill** (not displayed to the user): Extract the literal values of the following placeholders from the `[CTX]` output lines at the end of the script:

| Placeholder | Semantics |
|-------------|-----------|
| `<CsName>` | Active ControlSet name (e.g., `ControlSet001`) |
| `<CcsPath>` | Full registry path of the active ControlSet (e.g., `HKLM:\{bf1a281b-...}E:/Windows/System32/config/SYSTEM\ControlSet001`) |

In subsequent scripts, `<CsName>` / `<CcsPath>` MUST be replaced with the above literals before execution. Note: After each DISM call, the HIVE will be unloaded; you MUST re-execute the Step 2 load script to remount the HIVE; after remounting, the mount path remains the fixed GUID path and the ControlSet number does not change, so the `<CcsPath>` / `<SoftPath>` / `<SysPath>` literals remain valid and do not need to be refreshed by re-running this step.

## Cross-References

| Type | Trigger Condition | Target |
|------|-------------------|--------|
| Depended upon | When any troubleshooting file needs to access the registry | This file must be executed first as a prerequisite |
| Chain successor | HIVE loading complete, continue boot chain diagnostics | → [bcd-boot.md](references/offline/bcd-boot.md) |
| Chain successor | HIVE loading complete, continue driver diagnostics | → [driver.md](references/offline/driver.md) |
| Conditional jump | SYSTEM HIVE corrupted and cannot be fixed individually | → Inform user that backup restore needs to be considered |
| Conditional jump | RegBack backup is valid | → Provide backup restore solution |

## Fix Recommendations

### Root Cause: Registry HIVE File Corrupted

**Fix Operation**:

```powershell
# Try restoring from RegBack
$bootLetter = '<BootLetter>'
$configDir = "${bootLetter}:\Windows\System32\config"
$backupDir = "${configDir}\RegBack"
if (Test-Path $backupDir) {
    # List backup files (may be empty or 0-byte on Win10 1803+ client editions)
    Get-ChildItem $backupDir | Format-Table Name, Length, LastWriteTime -AutoSize
    # Backup current corrupted files then copy from RegBack
    # Copy-Item "${backupDir}\SYSTEM" "${configDir}\SYSTEM" -Force
} else {
    Write-Host "RegBack directory not found"
}
```

**Verification**:

After replacement, re-execute reg load to verify whether it can be loaded successfully

**Risk Notes**: RegBack backups may not be the latest state; after restoration, recent configuration changes may be lost. Win10 1803+ client editions no longer automatically back up by default; the RegBack directory may be empty or files may be 0 bytes; Windows Server editions typically retain RegBack backups

## HIVE Unloading

After diagnostics are complete, **all loaded HIVEs must be unloaded** (including HIVEs not directly accessed during diagnostics). Incomplete unloading will prevent the target disk from being safely detached.

### Unload Script

```powershell
$guid = '{bf1a281b-ad7b-4476-ac95-f47682990ce7}'
$bootLetter = '<BootLetter>'
$files = @(
    "${bootLetter}:\Windows\System32\config\SYSTEM",
    "${bootLetter}:\Windows\System32\config\SOFTWARE",
    "${bootLetter}:\Windows\System32\config\SAM",
    "${bootLetter}:\Windows\System32\config\SECURITY",
    "${bootLetter}:\Windows\System32\config\DRIVERS",
    "${bootLetter}:\Windows\System32\config\DEFAULT",
    "${bootLetter}:\Windows\System32\config\COMPONENTS",
    "${bootLetter}:\Users\Default\NTUSER.DAT",
    "${bootLetter}:\Windows\System32\SMI\Store\Machine\SCHEMA.DAT"
)
$results = @()
foreach ($f in $files) {
    $loadPath = $guid + ($f -replace '\\','/')
    $output = reg unload "HKLM\$loadPath" 2>&1
    $results += [PSCustomObject]@{
        File   = Split-Path $f -Leaf
        Result = if ($LASTEXITCODE -eq 0) { 'OK' } else { $output }
    }
}
$results | Format-Table -AutoSize
```

### Unload Verification (Must Execute)

After unloading, you **must verify** that no residual keys with the GUID prefix remain under HKLM:

```powershell
$guid = '{bf1a281b-ad7b-4476-ac95-f47682990ce7}'
$remaining = reg query HKLM 2>&1 | Select-String $guid
if ($remaining) {
    Write-Host "WARNING: The following HIVEs are still not unloaded:" -ForegroundColor Red
    $remaining | ForEach-Object { Write-Host "  $_" -ForegroundColor Yellow }
    # Retry unloading for each residual item
    foreach ($line in $remaining) {
        $key = ($line -replace '^\s*HKEY_LOCAL_MACHINE\\','').Trim()
        if ($key) { reg unload "HKLM\$key" 2>&1 }
    }
} else {
    Write-Host "OK: All HIVEs have been successfully unloaded" -ForegroundColor Green
}
```

**Notes**:
- Common causes of unload failure: a process or handle is still accessing the HIVE (e.g., regedit editor not closed)
- When encountering unload failures, first close all processes that may hold handles, then retry
- **You must** verify pass before performing disk detach operations

**Analysis Determination**: The unload script output results need to be distinguished into three states:
- `OK` (exit code 0) = unload successful
- Output contains `ERROR_FILE_NOT_FOUND` / `not found` / `not loaded` etc. = the HIVE was never loaded, no need to unload, considered normal
- Other non-zero exit codes = actual unload failure, need to retry or investigate handle occupation
