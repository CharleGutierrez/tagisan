# RDP Session Abnormal Disconnection Diagnosis

## Overview

This file is a dedicated diagnostic for the "RDP session abnormal disconnection" symptom, targeting scenarios where **a session is successfully established, then disconnected after some period of use** (e.g., "internal error after running for a while, then auto-disconnects," "session interrupted for no apparent reason").

Boundaries with other RDP troubleshooting files:

- Connection **establishment failure** (cannot connect / rejected) → `rdp-service.md` / `rdp-auth.md`
- **Immediate disconnect after connection** (kicked out right after login) → `rdp-service.md` (third-party WinStation conflict) / `rdp-licensing.md` (GracePeriod disconnect)
- Session **disconnected during use** → this file

**Input**: User problem description (required), user-reported disconnect time point (strongly recommended, used to narrow the event collection window)
**Output**: Root cause list (root_cause / severity / evidence / explanation / fix)

## Trigger Conditions

- User reports "remote desktop disconnects after some use," "session reports internal error mid-way then disconnects," "connection auto-disconnects"
- Session can be normally established; disconnection occurs during use (distinct from connection establishment failure and immediate disconnect)

## Diagnostic Steps

### Step 1: Session Timeline Reconstruction

**Data Collection**: LocalSessionManager/Operational **info-level** full events (disconnect analysis depends on info-level events 21/22/24/25/40/41/42; collecting only warning level will miss all key events; when the user provides a disconnect time, MUST use afterDate/beforeDate in pair to narrow the window) + `GuestOS:RDSession` (session connect/disconnect timestamps)

- PowerShell script: [rdp-session-disconnect.ps1](references/online/scripts/rdp-session-disconnect.ps1) §Step 1

**Analysis Approach**:

1. Use LocalSessionManager/Operational info-level events to reconstruct the session timeline: Event 21 (login) → 22 (Shell start) → 25 (reconnect) → 40 (disconnect, includes reason code) → 24 (disconnect confirmation)
2. Cross-align Event 40 disconnect time with DisconnectTime returned by RDSession to confirm the target session that was disconnected and the session lifetime
   - Normal: Session survives for a long time with no Event 40 disconnect record → Not a server-side recorded disconnect; pivot to client-side troubleshooting
   - Abnormal: Event 40 clearly records disconnect time and reason code → Proceed to Step 2 for decoding
3. Session arbitration events 41/42 present in the event chain with no error code → User actively disconnected or logged off normally, **Determination**: Not a fault

### Step 2: Disconnect Reason Code Decoding and Determination

**Data Collection**: Reuse the Event 40 content collected in Step 1; no additional collection needed

- PowerShell script: [rdp-session-disconnect.ps1](references/online/scripts/rdp-session-disconnect.ps1) §Step 2

**Analysis Approach**:

1. Decode Event 40 reason code: The value is in HRESULT form (e.g., 2147942521); the low 16 bits are the Win32 error code (2147942521 = 0x80070079, low 16 bits 0x0079 = 121). You can use `net helpmsg <low-16-bit-decimal>` to look up the error description
2. Typical determinations:
   - Reason code 2147942521 (0x80070079, ERROR_SEM_TIMEOUT semaphore timeout) → The server did not receive the client transport acknowledgment within the timeout window; this is a TCP connection interruption due to unresponsive link; after ruling out server-side configuration with Step 3/4, **Determination**: Network transport layer link issue (client network / public network link / intermediate devices), not a GuestOS root cause, **Severity**: Warning
   - Reason code points to a server-side component (e.g., RPC/memory error) → Cross-check with System log and rdp-service.md service status, **Severity**: Warning
   - Reason code does not correspond to any known scenario → Proceed to Step 5: recommend enabling RDP debug logs (RDPCoreTS/LocalSessionManager/RemoteConnectionManager three Debug channels), reproduce, then collect debug channel events
3. No Event 40 but session disconnects frequently → Check Step 3 timeout configuration and Step 4 link events

### Step 3: Server-Side Active Disconnect Configuration Check

**Data Collection**: `GuestOS:WinStationRegistry` (session timeout and keep-alive configuration: MaxIdleTime / MaxConnectionTime / MaxDisconnectionTime / KeepAliveTimeout) + Group Policy keep-alive connection configuration

- PowerShell script: [rdp-session-disconnect.ps1](references/online/scripts/rdp-session-disconnect.ps1) §Step 3

**Analysis Approach**:

1. Check WinStation timeout configuration:
   - Normal: MaxIdleTime / MaxConnectionTime / MaxDisconnectionTime all 0 (no timeout limit) → Server will not actively disconnect sessions due to timeout
   - Abnormal: Non-zero and value too small (e.g., < 300000 ms = 5 minutes) → **Root cause**: Timeout configuration too aggressive; server actively disconnects the session, **Severity**: Warning; Fix direction is to adjust the overly small timeout back to 0 (no limit) or ≥ 1 hour (simultaneously check Group Policy overrides to avoid policy writeback)
2. Check keep-alive configuration (KeepAliveTimeout / Group Policy "Configure keep-alive connection interval"):
   - Keep-alive not enabled and reason code points to link interruption → **Optimization suggestion** (not a root cause): Enable Group Policy "Computer Configuration → Administrative Templates → Windows Components → Remote Desktop Services → Remote Desktop Session Host → Connections → Configure keep-alive connection interval," set to 1 minute to enhance link jitter tolerance
3. Check whether Group Policy path has policies overriding local timeout configuration (RegistryPolicy collection items)

### Step 4: Link Layer Cross-Validation

**Data Collection**: System log network sources (netkvm / TCPIP / NDIS) warning-level events + `GuestOS:NetAdapter` (network adapter status and driver version) + System log TermDD errors

- PowerShell script: [rdp-session-disconnect.ps1](references/online/scripts/rdp-session-disconnect.ps1) §Step 4

**Analysis Approach**:

1. Check network adapter and link events:
   - Normal: No warning or above events from netkvm / TCPIP / NDIS, all network adapters Up, no TermDD errors → Supports the "link transport interruption" conclusion (interruption occurs on the external link of the instance)
   - Abnormal: Network adapter link jitter/driver error events appear → **Root cause**: In-instance network adapter or driver abnormality causing session disconnect; jump to [device-driver.md](references/online/device-driver.md), **Severity**: Warning
2. Conclusion synthesis: Step 2 reason code points to link + Step 3 no timeout configuration + Step 4 no in-instance link abnormality → **Root cause**: Network transport layer link interruption (client network / public network link / intermediate devices), not a GuestOS issue; recommend the user capture ping/tracert from the client side for comparative verification when reproducing

### Step 5: Protocol Stack Debug Log Deep Dive (Optional)

**Trigger Condition**: Execute only when the Step 2 reason code does not correspond to any known scenario and protocol stack-level details are needed; otherwise skip this step and proceed directly to the cross-reference chain successor.

**Optional Prerequisite (recommend user to execute)**: Conventional Operational/Admin channels only record key events; RDP component Debug/Analytic channels are disabled by default. MUST first recommend the user to execute the following commands inside the instance to enable debug logs (three groups: RDPCoreTS protocol stack, LocalSessionManager session management, RemoteConnectionManager connection management), reproduce the fault once, then collect:

```powershell
# Enable RDP debug logs (RDPCoreTS Debug + Analytic, LocalSessionManager Debug, RemoteConnectionManager Debug)
wevtutil sl Microsoft-Windows-TerminalServices-RDPCoreTS/Debug /e:true /q:true
wevtutil sl Microsoft-Windows-TerminalServices-RDPCoreTS/Analytic /e:true /q:true
wevtutil sl Microsoft-Windows-TerminalServices-LocalSessionManager/Debug /e:true /q:true
wevtutil sl Microsoft-Windows-TerminalServices-RemoteConnectionManager/Debug /e:true /q:true
```

After troubleshooting is complete, recommend the user disable debug logs:

```powershell
wevtutil sl Microsoft-Windows-TerminalServices-RDPCoreTS/Debug /e:false /q:true
wevtutil sl Microsoft-Windows-TerminalServices-RDPCoreTS/Analytic /e:false /q:true
wevtutil sl Microsoft-Windows-TerminalServices-LocalSessionManager/Debug /e:false /q:true
wevtutil sl Microsoft-Windows-TerminalServices-RemoteConnectionManager/Debug /e:false /q:true
```

- When asking the user whether to execute this item, MUST include the complete enable commands and execution instructions (administrator PowerShell) in the same turn for the user to execute immediately after choosing; do not let the user choose first and then provide commands separately
- User declines or does not respond → Skip this step; note in the conclusion "debug logs not enabled, protocol stack details not covered"
- Note: TermDD kernel driver errors are recorded in the System log (source TermDD), not in the above debug channels; already covered by Step 4

**Data Collection**: After the user has enabled debug logs and reproduced the fault, collect Error/Warning-level events from three debug channels: TerminalServices-RDPCoreTS/Debug (protocol stack debug events), TerminalServices-LocalSessionManager/Debug (session management debug events), TerminalServices-RemoteConnectionManager/Debug (connection management debug events); narrow the time window based on user reproduction time

- PowerShell script: [rdp-session-disconnect.ps1](references/online/scripts/rdp-session-disconnect.ps1) §Step 5

**Analysis Approach**:

1. RDPCoreTS/Debug: Protocol negotiation, channel initialization, or encryption layer errors appear → Points to RDP protocol stack internal issue, **Severity**: Warning
2. LocalSessionManager/Debug: Internal errors in session creation/destruction/switching → Points to session management layer issue, **Severity**: Warning
3. RemoteConnectionManager/Debug: Internal errors in connection acceptance/routing/authorization → Points to connection management layer issue, **Severity**: Warning
4. No Error-level events in all three channels → No abnormal protocol stack records; synthesize with existing findings and proceed to the cross-reference chain successor

## Cross-References

| Type | Trigger Condition | Jump Target |
|------|-------------------|-------------|
| Conditional jump | Session cannot be established (not disconnected during use) | → [rdp-service.md](references/online/rdp-service.md) (service/listener) and [rdp-auth.md](references/online/rdp-auth.md) (authentication) |
| Conditional jump | Immediate disconnect after connection | → [rdp-service.md](references/online/rdp-service.md) (third-party WinStation conflict) / [rdp-licensing.md](references/online/rdp-licensing.md) (GracePeriod disconnect) |
| Conditional jump | Step 4 finds network adapter/driver link events | → [device-driver.md](references/online/device-driver.md) |
| Conditional jump | Reason code cannot be decoded and protocol stack details needed | → This file [Step 5: Protocol Stack Debug Log Deep Dive (Optional)](#step-5-protocol-stack-debug-log-deep-dive-optional) |
| Chain successor | Determined as link layer issue | → Network domain troubleshooting (follow [WORKFLOW-GUIDE.md](references/online/WORKFLOW-GUIDE.md) `GuestOS.InsideNetworkAccessFailed` unified sequence) |
| Chain successor | All Steps in this file (including Step 5) executed without confirming root cause | → [system-health-check.md](references/online/system-health-check.md) global baseline health check |


## Fix Recommendations

Root causes confirmed in this file are addressed by the conditional jump targets in the Cross-References table above (device-driver, system-health-check, network domain, etc.).
