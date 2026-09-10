# Windows Firewall Diagnostic Fix

## Fix Recommendations

### Root cause: Firewall service or dependent service not running

Covers: MpsSvc (firewall service) not running/disabled, BFE (Base Filtering Engine) not running, netprofm (Network List Service) not running.

**Fix operation**:

```powershell
# Start services in dependency order: BFE → MpsSvc → netprofm
$services = @(
    @{ Name = 'BFE';      StartupType = 'Automatic' },
    @{ Name = 'MpsSvc';   StartupType = 'Automatic' },
    @{ Name = 'netprofm'; StartupType = 'Manual'    }
)
foreach ($svc in $services) {
    $current = Get-Service -Name $svc.Name -ErrorAction SilentlyContinue
    if ($current -and $current.Status -ne 'Running') {
        Set-Service -Name $svc.Name -StartupType $svc.StartupType
        Start-Service -Name $svc.Name
        Write-Host "Started: $($svc.Name)"
    }
}
```

**Verification**:

```powershell
Get-Service -Name BFE, MpsSvc, netprofm | Select-Object Name, DisplayName, Status, StartType | Format-Table -AutoSize
Get-NetConnectionProfile | Select-Object Name, InterfaceAlias, NetworkCategory | Format-Table -AutoSize
```

Expected result: All three services Status = Running, network profile type correctly identified

**Risk notes**:

- **Session impact**: BFE restart will briefly interrupt all network filtering; after starting MpsSvc, all configured firewall rules are immediately enabled, which may block currently allowed traffic.
- **Persistence scope**: StartupType changes persisted across reboot.
- **Rollback command**: `Stop-Service MpsSvc; Set-Service -Name MpsSvc -StartupType Disabled`
- **Note**: Before starting, ensure critical ports (e.g., 3389) have corresponding allow rules.


---

### Root cause: No effective inbound allow rule

Covers: No matching inbound allow rule, or rule exists but Profile does not cover the current active network type.

**Fix operation**:

```powershell
# Add inbound allow rule for target port (replace port number and protocol)
$port = 3389
$protocol = 'TCP'
New-NetFirewallRule -DisplayName "Allow $protocol Port $port" -Direction Inbound -Protocol $protocol -LocalPort $port -Action Allow -Profile Any

# If an existing rule has mismatched Profile, modify it directly (replace rule name)
# Set-NetFirewallRule -DisplayName "<RuleName>" -Profile Any
```

**Verification**:

```powershell
Get-NetFirewallRule -DisplayName "Allow $protocol Port $port" | Select-Object DisplayName, Enabled, Action, Profile
```

Expected result: Rule exists, Enabled = True, Action = Allow, Profile covers the current active profile

**Risk notes**:

- **Session impact**: New rule takes effect immediately, does not interrupt existing connections.
- **Persistence scope**: Firewall rules persisted across reboot.
- **Rollback command**: `Remove-NetFirewallRule -DisplayName 'Allow <protocol> Port <port>'`
- **Note**: Only open business-essential ports; Profile = Any opens under all network types, recommend extending only to the actually needed profile.

---

### Root cause: Explicit Block rule overrides Allow rule

**Fix operation**:

```powershell
# First identify conflicting Block rules (replace port number)
$targetPort = '3389'
Get-NetFirewallRule -Direction Inbound -Action Block -Enabled True | Where-Object {
    $port = ($_ | Get-NetFirewallPortFilter).LocalPort
    @($targetPort, 'Any') -contains $port
} | Select-Object DisplayName, Action, Profile

# Disable conflicting Block rule (replace rule name)
# Disable-NetFirewallRule -DisplayName "<RuleName>"
```

**Verification**:

```powershell
Get-NetFirewallRule -Direction Inbound -Action Block -Enabled True | Where-Object {
    ($_ | Get-NetFirewallPortFilter).LocalPort -eq $targetPort
} | Measure-Object
```

Expected result: Count = 0, no conflicting Block rules

**Risk notes**:

- **Session impact**: Disabling rule takes effect immediately, previously blocked traffic will be allowed through.
- **Persistence scope**: Rule state changes persisted across reboot.
- **Rollback command**: `Enable-NetFirewallRule -DisplayName '<RuleName>'`
- **Note**: Before disabling, confirm the rule's purpose; Block rules may be required by security policy.

---

### Root cause: Outbound block rule scope too broad

**Fix operation**:

```powershell
# First identify overly broad outbound block rules
Get-NetFirewallRule -Direction Outbound -Action Block -Enabled True | ForEach-Object {
    $portFilter = $_ | Get-NetFirewallPortFilter
    [PSCustomObject]@{
        DisplayName = $_.DisplayName
        Profile     = $_.Profile
        Protocol    = $portFilter.Protocol
        RemotePort  = $portFilter.RemotePort
    }
} | Format-Table -AutoSize

# Disable overly broad rules (replace rule name)
# Disable-NetFirewallRule -DisplayName "<RuleName>"
```

**Verification**:

```powershell
Get-NetFirewallRule -Direction Outbound -Action Block -Enabled True | Measure-Object
```

Expected result: No unreasonable broad block rules

**Risk notes**:

- **Session impact**: Disabling rule takes effect immediately, previously blocked outbound traffic will be allowed through.
- **Persistence scope**: Rule state changes persisted across reboot.
- **Rollback command**: `Enable-NetFirewallRule -DisplayName '<RuleName>'`
- **Note**: Some outbound block rules may be required by security policy; recommend narrowing the rule scope rather than directly disabling.

---

### Root cause: Firewall profile default blocks outbound connections

**Fix operation**:

```powershell
# Change default outbound action to Allow
Set-NetFirewallProfile -Profile Domain,Private,Public -DefaultOutboundAction Allow
```

**Verification**:

```powershell
Get-NetFirewallProfile -All | Select-Object Name, DefaultOutboundAction
```

Expected result: All profiles DefaultOutboundAction = Allow

**Risk notes**:

- **Session impact**: Takes effect immediately, previously blocked outbound connections will be allowed through.
- **Persistence scope**: Profile settings persisted across reboot.
- **Rollback command**: `Set-NetFirewallProfile -Profile Domain,Private,Public -DefaultOutboundAction Block`
- **Note**: Restoring default outbound allow reduces outbound traffic control; if required by security policy, should instead add specific outbound allow rules.

---

### Root cause: Firewall profiles disabled (security posture)

Covers: All firewall profiles (Domain/Private/Public) found Disabled — typically a deliberate earlier change or a leftover hardening rollback — leaving the instance with no host-layer packet filtering. Diagnostic signal: `Get-NetFirewallProfile` shows `Enabled False` for every profile.

**Fix operation**:

```powershell
# BEFORE enabling: confirm critical ports (e.g. 3389 RDP) have allow rules,
# otherwise enabling the firewall may cut the current remote session
Get-NetFirewallRule -DisplayName "*Remote Desktop*" |
    Select-Object DisplayName, Enabled, Direction, Action, Profile | Format-Table -AutoSize

# Enable all profiles (default inbound Block / outbound Allow then applies)
Set-NetFirewallProfile -Profile Domain,Private,Public -Enabled True
```

**Verification**:

```powershell
Get-NetFirewallProfile | Select-Object Name, Enabled, DefaultInboundAction, DefaultOutboundAction | Format-Table -AutoSize
```

Expected result: All profiles Enabled = True

**Risk notes**:

- **Session impact**: HIGH — with profiles enabled, default inbound action Block takes effect immediately; if no allow rule covers the current session port (RDP 3389), the session drops. Verify the allow rule first (above) or add one per "No effective inbound allow rule" before enabling.
- **Persistence scope**: Profile state persisted across reboot.
- **Rollback command**: `Set-NetFirewallProfile -Profile Domain,Private,Public -Enabled False`
- **Note**: Enabling with defaults blocks unsolicited inbound traffic; business ports need explicit allow rules. The security group continues to control edge filtering regardless.

---

### Root cause: WFP third-party filter rules causing traffic drops

Covers: WFP filter rules injected by third-party software (security software, VPN, Npcap, etc.) causing packet drops, or orphaned filters left after software uninstall still intercepting traffic.

**Fix operation**:

```powershell
# Option A: Uninstall the offending software identified via providerKey
# Control Panel → Programs and Features → uninstall the software
# Common culprits: Npcap (Wireshark), third-party antivirus, VPN clients
# Uninstall triggers the software's WFP provider to unregister and clean up its filters
# Reboot after uninstall — non-persistent orphan filters are cleared on reboot
Write-Host 'Reboot required: shutdown /r /t 0'

# Option B: After reboot, if the specific filter still exists (persistent), delete it by runtime ID.
# Use when: filterId known from Step 7, filters are persistent (survived reboot), or reboot is undesirable.
# Runtime filterId may change after reboot — re-query from Step 7 before executing.
$code = @'
using System;
using System.Runtime.InteropServices;

public static class Wfp
{
    [DllImport("fwpuclnt.dll")]
    public static extern uint FwpmEngineOpen0(
        string serverName,
        uint authnService,
        IntPtr authIdentity,
        IntPtr session,
        out IntPtr engineHandle);

    [DllImport("fwpuclnt.dll")]
    public static extern uint FwpmEngineClose0(IntPtr engineHandle);

    [DllImport("fwpuclnt.dll")]
    public static extern uint FwpmFilterDeleteById0(IntPtr engineHandle, ulong id);

    public const uint RPC_C_AUTHN_WINNT = 10;

    public static uint DeleteFilterById(ulong id)
    {
        IntPtr engine;
        uint r = FwpmEngineOpen0(null, RPC_C_AUTHN_WINNT, IntPtr.Zero, IntPtr.Zero, out engine);
        if (r != 0) { return r; }
        try {
            return FwpmFilterDeleteById0(engine, id);
        } finally {
            FwpmEngineClose0(engine);
        }
    }
}
'@
Add-Type -TypeDefinition $code -ErrorAction Stop

$id = [uint64]<filterId_from_Step7>
Write-Output "Deleting WFP filter runtime id=$id via FwpmFilterDeleteById0 ..."
$ret = [Wfp]::DeleteFilterById($id)
if ($ret -eq 0) {
    Write-Output "SUCCESS: filter $id deleted (return=0)"
} else {
    Write-Output ("FAILED: return code = 0x{0:X8} ({1})" -f $ret, $ret)
}
```

**Verification**:

```powershell
# Re-export WFP events after fix and check for drops
netsh wfp show netevents file="$env:TEMP\netevents_after.xml"
# Verify target traffic is no longer dropped
Test-Connection -ComputerName <TargetIP> -Count 2
```

Expected result: No matching drop events, target traffic passes normally

**Risk notes**:

- **Session impact**: `netsh advfirewall reset` restores firewall to default profile; active RDP sessions may be interrupted if default rules differ from current rules. `FwpmFilterDeleteById0` removes a specific WFP filter; if the filter ID is wrong, unrelated traffic filtering may be affected.
- **Persistence scope**: Firewall reset persists across reboot; WFP filter deletion is permanent.
- **Rollback command**: No direct rollback for firewall reset; re-apply custom firewall rules manually. For WFP filter deletion, re-add the filter if the filter definition is known.
- **Note**: `FwpmFilterDeleteById0` is an advanced operation; use only when filterId is confirmed via WFP diagnostics. Always export current firewall policy (`netsh advfirewall export`) before reset.
