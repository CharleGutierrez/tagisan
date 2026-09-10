# Disk Partition Identification and Attribute Verification Fix

## Fix Recommendations

### Root cause: Boot partition hidden

**Fix operation**:

```powershell
$diskNum = <TargetDiskNumber>
$partNum = <BootPartitionNumber>
$script = @"
select disk $diskNum
select partition $partNum
attribute volume clear hidden
"@
$script | diskpart.exe
if ($LASTEXITCODE -ne 0) { Exit $LASTEXITCODE }
```

**Verification**: `Get-Partition -DiskNumber $diskNum -PartitionNumber $partNum | Select-Object IsHidden` → IsHidden=False

**Risk notes**:
- Session impact: Clears the hidden attribute on the boot partition of the offline disk using diskpart
- Persistence scope: Survives reboot — the partition attribute change persists on the target system
- Rollback: Re-set the hidden flag using `diskpart` with `attribute volume set hidden` on the same partition

---

### Root cause: System partition has no Active flag set

**Fix operation**:

```powershell
$diskNum = <TargetDiskNumber>
$partNum = <SystemPartitionNumber>
$script = @"
select disk $diskNum
select partition $partNum
active
"@
$script | diskpart.exe
if ($LASTEXITCODE -ne 0) { Exit $LASTEXITCODE }
```

**Verification**: `Get-Partition -DriveLetter <SystemLetter> | Select-Object IsActive` → IsActive=True

**Risk notes**:
- Session impact: Sets the Active flag on the system partition of the offline disk using diskpart (BIOS boot mode only)
- Persistence scope: Survives reboot — the partition attribute change persists on the target system
- Rollback: Clear the Active flag using `diskpart` with `inactive`. Setting the wrong partition as Active may cause boot failure

---

### Root cause: Insufficient permissions on boot partition root directory

**Fix operation**:

```powershell
function AddAllowAccess {
    param (
        [System.Security.AccessControl.FileSystemSecurity]$acl,
        [System.Security.Principal.NTAccount]$identity,
        [System.Security.AccessControl.FileSystemRights]$rights,
        [switch]$RemoveOnly
    )
    # Remove Deny rules
    $rule = New-Object System.Security.AccessControl.FileSystemAccessRule($identity, $rights, "Deny")
    $acl.RemoveAccessRuleAll($rule)
    if (-not $RemoveOnly) {
        $rule = New-Object System.Security.AccessControl.FileSystemAccessRule($identity, $rights, "Allow")
        $acl.AddAccessRule($rule)
    }
}

$bootLetter = '<BootLetter>'
$path = "${bootLetter}:\"
try {
    $acl = Get-Acl $path
    AddAllowAccess $acl "NT AUTHORITY\SERVICE" "Read" -RemoveOnly
    AddAllowAccess $acl "BUILTIN\Users" "ReadAndExecute"
    AddAllowAccess $acl "NT AUTHORITY\SYSTEM" "FullControl"
    Set-Acl -Path $path -AclObject $acl
} catch {
    Write-Error "Failed to set acl: $($_.Exception.Message)"
    Exit 1
}
```

**Verification**:

```powershell
(Get-Acl "<BootLetter>:\").Access | Where-Object { $_.IdentityReference -match 'Users|SYSTEM' } | Format-List IdentityReference, FileSystemRights, AccessControlType, IsInherited, InheritanceFlags, PropagationFlags
```

Expected result: Users has ReadAndExecute, SYSTEM has FullControl

**Risk notes**:
- Session impact: Modifies ACLs on the boot partition root directory of the offline disk; grants SYSTEM FullControl and Users ReadAndExecute, non-recursive
- Persistence scope: Survives reboot — ACL changes persist on the target system
- Rollback: Restore original ACLs using `icacls "${bootLetter}:\" /reset`
