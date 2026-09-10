# Windows Offline Diagnostic Execution Flow

This file carries the complete execution flow of Windows ECS **offline diagnostics**, defining all execution details after entering offline diagnostics.

**User Scenario**: The faulty system disk has been mounted as a data disk to the current instance, and root cause analysis and fix of the offline system disk are required.

**Invocation Conditions** (must be met simultaneously):
1. The target faulty system disk has been mounted as a data disk to the current instance (if the user has not explicitly provided drive letter, serial number, disk number, etc., the target disk must be inferred)
2. The current instance has a PowerShell command execution channel

**Prohibited**: Performing any diagnostic or modification operations on the currently running system. All operations are strictly on the offline mounted target disk.

This file is self-contained: the problem domain routing table, fallback mechanism, cross-step caching mechanism, and PowerShell collection script rules are all defined inline in this file.

---

## Table of Contents

- General Execution Constraints
  - User Presentation Rules
  - Collection Channel Rules
  - Collection Fallback Chain
  - DISM Mandatory Rules
- Diagnostic Execution Flow
  - Prerequisite Check
  - Problem Understanding
  - Path Planning
  - Environment Context Initialization
  - Step-by-Step Execution
  - Causal Chain Analysis
  - Evidence Review
  - Fix Plan
  - Diagnostic Cleanup
- Offline Problem Domain Routing
  - Refined Routing Table
- Fallback Mechanism
- Data Collection and Cross-Step Caching
  - Global Context Constant Fields
  - Cross-Step Data Caching
- Fix Phase Execution Convention
- Output Format Templates
  - Evidence Review Template
  - Causal Chain Example
  - Diagnostic Conclusion Output Template
  - Check Item Summary Template
- PowerShell Collection Script Rules
  - 0. Pre-Execution Self-Check Checklist
  - 1. Cmdlet Selection
  - 2. Output Style
  - 3. Variable Naming: Avoid Built-in Identifiers
  - 4. cmd Tool Invocation and Encoding Handling
  - 5. REG_MULTI_SZ Multi-Value Parsing
  - 6. Variable Name Followed by Colon in String Interpolation
  - 7. foreach Statement Cannot Directly Connect to Pipeline
  - 8. cmd Tool Error Handling Using $LASTEXITCODE
  - 9. Version String Parsing
  - 10. UTF-8 BOM Compatibility
  - 11. ConvertTo-Json Depth Limit
  - 12. User Command Display Gate (MUST Verify Before Display)
  - Offline Environment Specific Rules
  - High-Frequency Pitfalls

## General Execution Constraints

### User Presentation Rules

The following phrasing templates correspond one-to-one with internal diagnostic phases/actions; when presenting progress, troubleshooting paths, current step, and other information to the user, MUST use the natural language phrasing from this table, and MUST NOT present the corresponding internal markers verbatim.

**Diagnostic Phase Recommended Phrasing Templates** (user-facing, unified style):

When loading each domain's troubleshooting file, express as "Checking + the domain's diagnostic function" (described in natural language by the domain's diagnostic function, without mentioning file names), e.g., driver.md → "Checking storage / virtualization driver status", update.md → "Checking installed patches and update status". Non-trivial phases/actions refer to the table below:

| Internal Phase / Action | User-Facing Phrasing (Example) |
|------|------|
| Loading boot-triage.md | "Performing preliminary interpretation of user-provided screenshots" |
| Querying instance status | "Confirming the current running status of the target instance" |
| Querying offline disk mount status | "Checking offline disk mount status" |
| Obtaining screenshot | "Obtaining instance screen snapshot to determine current display" |
| Verifying command execution channel | "Verifying PowerShell command execution channel" |
| Skipping a step (fallback) | "No issues found in the above checks, continuing with additional checks" |
| Direct root cause | "Found a problem that directly caused this fault" |
| Contributing root cause | "Found a contributing / indirectly impacting problem" |
| Critical severity | "Severe issue, will prevent boot" |
| Warning severity | "Issue requiring attention, may affect stability" |
| Info severity | "Informational item for reference" |

**Note**: The table above is for reference only; wording may be adjusted as appropriate; the principle is to express "what is being done", without presenting internal identifiers like "Calling X.md / Step Y".

### Collection Channel Rules

The collection channel for offline diagnostics is PowerShell command execution; before executing any collection, MUST follow the PowerShell Collection Script Rules section below in this file and write collection scripts according to its rules:

- **Direct execution channel** (PowerShell command execution on local instance) → PowerShell Collection Script Rules section below
- **Remote execution channel** (PowerShell command execution via `aliyun ecs run-command` on remote instance where the offline disk is mounted) → Same PowerShell Collection Script Rules, plus [REMOTE-EXECUTION.md](references/REMOTE-EXECUTION.md) for remote command delivery patterns and API semantics

**Pre-Execution Self-Check (MUST)**: After writing a script and **before** executing it in the offline environment, MUST scan the script text item by item against the Pre-Execution Self-Check Checklist in the PowerShell Collection Script Rules section below; if any signal matches, fix it before executing. The value of this step is to catch rule violations before execution: each round trip in the offline environment is more costly, and script-layer errors (Chinese garbled text, output truncation, exit code not checked) can easily be mistaken for real faults in the offline system itself, thereby skewing root cause determination.

- **Remote execution channel additional checks**: (1) script does not reference local files on the agent machine, (2) script does not require interactive input, (3) script output stays small — Cloud Assistant truncates oversized output (see [REMOTE-EXECUTION.md](references/REMOTE-EXECUTION.md) §Output Size Management)

### Collection Fallback Chain

**Fallback chain by execution channel (MUST degrade level by level in order)**:

- **Local mode**: Command line (PowerShell direct execution) → display command for user manual execution
- **Remote mode**: Remote command line (`aliyun ecs run-command`) → display command for user manual execution

When the command execution channel is unavailable causing collection / fix scripts to be unable to execute directly in the target environment, MUST output the complete pending command verbatim to the user, ask the user to manually execute it in the target environment and paste back the raw output; upon receiving the user's pasted result, continue the original process with "Analysis & Determination → Normal/Abnormal Conclusion"; the diagnostic sequence, problem domain matching, causal chain analysis, fix plan, and other processes remain unchanged.

**Fallback Determination Strategy**:

- **Channel-layer failure** (Cloud Assistant unreachable, network error, authorization failure, instance not found, remote session unreachable, connection timeout, exit code empty or no output at all) → Enter fallback: degrade to next level. The first determination takes effect immediately with no retries
- **Script-layer failure** (cmdlet not found, permission denied, path not found, syntax error with non-zero exit code and clear error output) → This is a script issue, not a channel issue; fix the script and retry on the same channel
- **Timeout failure** (command execution exceeded timeout) → Increase timeout and retry once; if still times out, break script into smaller chunks or fallback to next level

**Collection Script Merging Principle**: After entering the fallback process, MUST as much as possible combine multiple related collection items in the current diagnostic phase into a single executable script output for the user, reducing the number of manual executions. When merging, maintain clear separator markers between each collection item's output (e.g., comment line `# --- <item-name> ---`), to facilitate segment-by-segment parsing after pasting back. Only when there are strong dependencies between collection items (subsequent commands need to be dynamically determined based on prior results) should they be output separately.

**User Command Display Gate (Mandatory)**: Before displaying commands for user manual execution, MUST verify item by item against §12 "User Command Display Gate" in the PowerShell Collection Script Rules section below (pure ASCII, placeholder compliance, no local path dependencies, copy-paste executable as a whole); if any item does not pass, fix before displaying. Pure ASCII is a hard requirement: the user's terminal encoding environment is uncontrollable (GBK/UTF-8), and non-ASCII characters are prone to garbled text after copy-paste, which may cause PowerShell parsing failure.

### DISM Mandatory Rules

Any DISM cmdlet (`Get-WindowsPackage` / `Get-WindowsDriver`, etc.) invocation MUST comply with the two fixed rules in [dism.md](references/offline/dism.md) "DISM Mandatory Rules" (HIVE mount MUST use GUID `{bf1a281b-ad7b-4476-ac95-f47682990ce7}` fixed path; after cmdlet returns, MUST immediately remount HIVE per [registry.md](references/offline/registry.md) Step 2), without skipping steps or taking shortcuts.

To reduce the number of load/unload operations, MUST batch all DISM operations together (in [network.md](references/offline/network.md) → [update.md](references/offline/update.md) order); the same `Get-WindowsPackage` / `Get-WindowsDriver` result is called only once per session, stored in cross-step cache for subsequent step reuse (cache key definitions see the Data Collection and Cross-Step Caching section below).

---

## Diagnostic Execution Flow

### Prerequisite Check

Before entering the main diagnostic flow, MUST first complete the following prerequisite gates; if any step fails, MUST terminate offline diagnostics and explain the reason to the user.

#### 1. Offline Disk Visibility Confirmation

Execute PowerShell `Get-Disk` to confirm the target faulty system disk has been mounted as a data disk to the current instance and is visible:

- **Target disk visible** → Continue normal flow; [environment.md](references/offline/environment.md) Step 3 disk not-present determination serves as fallback confirmation
- **Target disk not visible** (target disk not found in `Get-Disk` output) → **Root cause: faulty system disk not mounted to current instance (severity=Critical)**. Guide user to verify system disk mount status in the console (whether it was accidentally detached, whether it was mounted to another instance, whether disk replacement / migration was performed recently); if the console confirms an anomaly, remount and retry. The fix direction is platform-side remounting of the system disk, which does not fall under GuestOS-level fix scope, **exit offline diagnostics** and output the above conclusion

#### 2. Command Execution Capability Verification

Execute a PowerShell `Get-Date` probe command to verify the command execution channel is available:

- Channel normal (command returns successfully) → Enter "Problem Understanding"
- Channel abnormal (command fails on first execution) → Immediately handle per the "Collection Fallback Chain" section in this file; retries prohibited

#### 3. Connection to Subsequent Flow

- [environment.md](references/offline/environment.md) in "Path Planning → Fixed Prerequisite Chain" takes over PowerShell channel, Storage module, target disk localization, and other environment verification in the offline environment

### Problem Understanding

1. **Extract core symptoms and key context**: BSOD code / black screen position / boot loop behavior / recent change operations / user-provided or context screenshots; all subsequent analysis MUST revolve around the original problem description
2. **Target disk identification**:
   - If the user has explicitly specified the target disk (drive letter, disk number, serial number, device path, etc. in any form), directly use that disk as the diagnostic target and proceed to subsequent flow; do not ask further or infer on your own
   - If the user has not explicitly specified, infer the target disk during the [environment.md](references/offline/environment.md) phase based on on-site information
3. **When information is insufficient, MUST first ask the user** (fault symptoms, recent operations, error messages, etc.), and only proceed to path planning after collecting sufficient information
4. **Evidence pre-analysis**: Load [boot-triage.md](references/offline/boot-triage.md), and perform preliminary delimitation of the problem through user-provided screenshots (if any; live screenshots are prohibited). If problem clues are found, use them as initial input for subsequent path planning, **MUST NOT exit prematurely, MUST continue executing the complete offline diagnostic flow (Path Planning → Step-by-Step Execution → Causal Chain Analysis → Fix Plan)**

### Path Planning

1. **Fixed prerequisite chain** (all diagnostics must execute strictly in order, cannot be omitted): [environment.md](references/offline/environment.md) → [disk-partition.md](references/offline/disk-partition.md) → [registry.md](references/offline/registry.md)
2. **Problem classification** (the sole classification point in the entire flow; secondary classification outside this step is prohibited): Use the Offline Problem Domain Routing section below, combine the problem's core symptoms, boot-triage evidence pre-analysis conclusions, and ambiguity resolution conclusions to determine the offline troubleshooting sequence per the refined routing table; classification results are only passed as internal context and not disclosed to the user; fuzzy matching → troubleshooting path is labeled "speculative troubleshooting". If subsequent evidence indicates classification needs adjustment, MUST return to this step for reclassification
3. **Dynamic planning** (only when classification does not match any offline scenario, or the classification sequence completes without finding a root cause):
   - Combine user problem description and boot-triage evidence pre-analysis conclusions to select relevant problem domain files from the routing table and fallback mechanism sections below
   - Compose a troubleshooting sequence, and inform the user: "This is a dynamically planned troubleshooting path, not a preset scenario"; fallback rules see the Fallback Mechanism section below
4. Output: **Fixed prerequisite chain + classification sequence (or dynamic planning sequence)**

### Environment Context Initialization

After the fixed prerequisite chain ([environment.md](references/offline/environment.md) → [disk-partition.md](references/offline/disk-partition.md) → [registry.md](references/offline/registry.md)) is completed, and before entering the problem domain file for problem domain matching, MUST **merge and save as session memory** the key constants and loaded offline registry paths obtained in this phase, for reuse by all subsequent diagnostic steps.

**Purpose**: Enable the large language model to **collect only once** throughout the diagnostic session and remember these public information items, with subsequent steps directly reusing them, **avoiding repeated declaration of drive letters / HIVE paths / boot mode, and avoiding repeated collection of the same data** (registry keys, DISM package lists, BCD enumerations, etc.).

**Execution Constraints**:

1. **Scalar constants** — "Extract → Write to session memory → Replace placeholders before execution":
   - `<BootLetter>` / `<SystemLetter>` / `<BootMode>` / `<DiskNumber>` / `<TargetDiskNumber>` (both are equivalent, both refer to the target disk number) / `<CcsPath>` / `<SoftPath>` / `<SysPath>` / `<BcdPath>` / `<CsName>` appearing in subsequent troubleshooting document scripts all refer to literals produced in this phase
   - After executing each step of the prerequisite chain, MUST extract literals from the output and write them to session memory per the fields listed in the "[CTX] Session Memory Backfill" section at the end of that step
   - Before generating and executing subsequent scripts, MUST replace all placeholders in the script with corresponding literals one by one; MUST NOT carry unreplaced placeholders into the command execution channel

2. **Large objects** — "First collection to disk → subsequent read from cache → cleanup at end":
   - **Cache directory**: `$cacheDir = Join-Path $env:SystemRoot 'Temp\diag-cache'` (in a normal Windows environment resolves to `C:\Windows\Temp\diag-cache`, in offline environment resolves to `X:\Windows\Temp\diag-cache`). All scripts that create / read / delete this cache MUST use this expression, MUST NOT hardcode drive letters
   - When first calling `Get-WindowsDriver` / `Get-WindowsPackage` / `bcdedit /enum all`, MUST process per [dism.md](references/offline/dism.md) "Standard Disk Cache Pattern"
   - Before subsequent steps repeat the above commands, MUST first `Test-Path $cacheFile`: if hit, directly read cache; if not hit, re-collect and write to disk
   - During diagnostic cleanup phase, MUST execute the cleanup script per [dism.md](references/offline/dism.md) "Standard Disk Cache Pattern"

**Usage Conventions**:
- Subsequent diagnostic collection blocks assume by default that the above two mechanisms are available: placeholders taken from session memory, large objects taken from disk cache; data not covered is collected per the actual steps in the problem domain file.
- DISM cmdlets unload the loaded registry HIVE during execution; after each DISM execution, MUST reload the HIVE (see [registry.md](references/offline/registry.md) Step 2); after reloading, the mount path remains the fixed GUID path, the ControlSet number does not change, and the literals for `<CcsPath>` / `<SoftPath>` / `<SysPath>` in session memory remain valid without needing to re-run Step 3 to refresh; disk cache does not depend on HIVE and also does not need re-collection
- This mechanism only serves diagnostic collection blocks. **Fix blocks MUST remain self-contained** (independently complete HIVE load → read active ControlSet → HIVE unload full flow, using placeholders the user can directly provide), so that users can independently paste and run them in the offline environment without relying on session memory and disk cache

**Specific field list, cache key definitions, backfill locations**: See the Data Collection and Cross-Step Caching section below.

**Output Hiding Constraint**: Session memory (placeholder → literal mapping) and disk cache state are both **model internal state**, MUST NOT be presented to the user in forms such as "session memory backfilled", "[CTX]", "placeholder list" in the final reply. User-facing output retains per-step Check Item Summaries, diagnostic conclusions, evidence, causal chain, and fix recommendations; `<placeholders>` appearing in scripts MUST be replaced with literals before execution, and the user always sees actual values.

### Step-by-Step Execution

Execute diagnostics by loading problem domain files one by one according to the troubleshooting sequence.

**Single Problem Domain File Execution Rules**:
1. **Select relevant step subset**: Filter based on the "Step Selection Guide" in the problem domain file, combined with core symptoms from the problem understanding phase + evidence pre-analysis results; if the problem domain file does not provide guidance or matching is uncertain, include conservatively; omitted items are covered by the "Subset Fallback Rule"
2. **Strict step-by-step execution**: Each step must complete the full "Data Collection → Analysis & Determination → Normal/Abnormal Conclusion" flow before proceeding to the next step, and MUST present a brief Check Item Summary to the user before proceeding to the next step; batch collection of multiple steps' data followed by centralized analysis is prohibited
   - Each abnormal determination MUST have corresponding collection data as evidence
   - Each step MUST provide a clear normal/abnormal binary conclusion
   - Discovered anomalies MUST be linked to specific root causes
   - Collection blocks assume by default that session memory and disk cache are available; `<placeholders>` in scripts MUST be replaced with literals from session memory before execution; placeholders MUST NOT be carried into the command execution channel
3. **Subset Fallback Rule**: If the selected steps find no issues, continue executing all remaining steps in the problem domain file
4. **Cross-reference handling**:
   - When encountering a cross-reference → prioritize processing the jump target, then return to the main sequence after completion
   - When the same problem domain file is referenced multiple times, execute it only once; directly reuse existing results for subsequent references

> Collection commands MUST use the commands given in the "Diagnostic Steps" of the problem domain file; only placeholder replacement and parameter adaptation to the current environment are allowed; generating collection scripts based solely on step descriptions is prohibited.

**Sequence Control Logic**:
- **Early termination**: The discovered root cause can fully explain all user symptoms → terminate subsequent problem domain files
- **Continue execution**: Only partially explains symptoms → continue executing subsequent problem domain files
- **Direction correction**: During execution, if the problem direction is found to be inconsistent with initial planning → return to path planning phase and re-plan based on new clues

### Causal Chain Analysis

1. Aggregate findings from all problem domain files
2. Build causal chain (distinguishing direct causes and indirect causes)
3. Prioritize by relevance to user's problem: **relation (Direct > Contributing > Unrelated) × severity (Critical > Warning > Info) dual-dimension sorting**
4. When no findings: attempt to expand the troubleshooting scope; if still no findings → truthfully inform the user
5. **Causal edge evidence triple (mandatory)**: Every edge in the causal chain ("A caused B") must carry all three of: ① the collection command plus the output excerpt it produced, ② the documented judgment it matched (the "Abnormal: X → Root cause: Y" pattern in the problem domain file), and ③ a time-window check using offline evidence timestamps (registry key last-write time, file timestamps, package/driver install dates, BCD entry times) — the cause-side timestamps must precede or overlap the fault's occurrence window. An edge missing any of the three MUST be downgraded to a speculative hypothesis in the Evidence Review and never presented as a conclusion
6. **Co-occurrence is not causation (red line)**: Linking an observed anomaly A to the user's symptom B merely because both were found on the mounted disk, or because they sound topically related, is prohibited. A causal claim requires all of: temporal precedence or overlap, the same object on both sides (same disk / partition / driver / package / registry key), and a causal mechanism documented in the reference files. When any of these cannot be established, report A and B as separate observations and state what additional data would connect them
7. **Differential diagnosis checklist**: Before asserting a root cause, list the alternative candidate causes for the symptom (drawn from the problem domain file's sub-scenario routing and related domains) that were examined, and the specific collected data that excluded each. A root cause asserted without its ruled-out alternatives is incomplete — return to collection for the unexamined candidates instead of asserting with confidence

### Evidence Review

Before presenting the diagnostic conclusion and fix plan, perform a consolidated review of the entire diagnostic process. Individual step summaries show per-check results, but the causal chain analysis may introduce inferences that go beyond what the data directly proves. This review catches those gaps before they reach the user — a conclusion that sounds authoritative but rests on an unsupported assumption is worse than an honest "I don't have enough data to confirm this yet".

**Relationship to Check Item Summary**: The per-step Check Item Summary is a progress checkpoint — it shows what each single step collected and concluded, helping the user track diagnostic progress in real time. The Evidence Review is a cross-step verification gate — it consolidates all judgments from the causal chain and verifies each one against collected data before any fix is proposed. They serve different purposes and are not redundant: one tracks progress during diagnosis, the other validates conclusions before action.

**Review process**:

1. **Enumerate collection actions**: List every data collection command executed during this session, with a brief summary of what was returned (registry values, DISM package states, BCD entries, driver statuses, file existence checks, etc.). This gives both you and the user a complete picture of what was actually checked, and makes it easy to spot collection gaps.

2. **Map evidence to judgments**: For each conclusion in the causal chain, cite the specific data point that supports it:
   - A registry value read from the offline HIVE, a DISM package state, a BCD entry, a driver start type, a file existence check, or a disk/partition status that directly confirms the finding → labeled **Confirmed**
   - If the conclusion is an inference (e.g., "likely caused by X because Y and Z were observed, but X itself was not directly measured") → labeled **Speculative**, with an explanation of what was observed and what additional data would confirm it
   - For each causal edge ("A caused B"), verify the **evidence triple**: the collection command + output excerpt, the matched documented judgment, and time-window consistency (cause-side timestamps — registry last-write, file times, install dates — precede or overlap the fault window). An edge missing any element keeps only its directly-observed endpoints as Confirmed findings — the causal link itself is downgraded to Speculative
   - **Co-occurrence is not causation**: two abnormalities found on the same mounted disk remain separate findings until temporal, same-object, and documented-mechanism linkage is shown; do not merge them into one story without that linkage
   - Include the **differential diagnosis**: for each asserted root cause, list the alternative candidates examined and the data that excluded each; alternatives not yet examined are collection gaps, not grounds for confidence

3. **Identify unsupported judgments**: Any judgment that cannot cite specific collected data must not be presented as a conclusion. Instead, downgrade it to a **hypothesis pending verification** and:
   - Label it as "Hypothesis — inference based on: {what was observed}"
   - List the missing data items needed to confirm or refute it
   - Provide the specific collection commands the user can run to obtain that data
   - Present these separately from evidence-backed conclusions so the user can clearly distinguish between proven findings and unverified hypotheses

4. **Present to user**: Output the evidence review using the template below before entering the fix plan. This gives the user the opportunity to spot gaps, correct misunderstandings, or provide additional context before fixes are proposed.

> The goal is transparency: the user should be able to trace every conclusion back to a specific piece of collected data, and clearly see where inference fills the gaps. When the user can see the evidence trail, they can make informed decisions about whether to proceed with a fix or collect more data first.

### Fix Plan

**Trigger condition**: Once "Evidence Review" is completed, directly enter fix plan output. Do not stop after only describing the root cause — the fix plan must follow the evidence review in the same response.

1. **Fully load the fix reference for the domain where the root cause resides**: Before entering the fix phase, for each root cause to be fixed, MUST first load its fix reference content — when the "Fix Recommendations" section of the problem domain file is a single-line pointer, MUST load the corresponding fix file in `references/offline/fixes/` that the pointer points to; when it is an inline fix section, MUST reload the corresponding problem domain file **reading completely to the end**, locate the "Fix Recommendations" section and execute the fix blocks given therein; when the root cause comes from a cross-domain parameterized reference, load the fix reference from the referenced party (fix authority). **Prohibited** from cobbling together fix commands based solely on impressions from the "Diagnostic Steps" section
2. **Prohibited from self-cobbling or rewriting fix commands**: MUST directly reuse the fix scripts given in the problem domain file, **only replacing placeholders** (e.g., `<BootLetter>` / `<SystemLetter>` / `<UEFI|BIOS>` / GUID / KB number, etc.); MUST NOT adjust command parameters based on "experience", rewrite field mappings (e.g., `{default}` device/osdevice pointing), skip steps, or merge multiple fix blocks
3. **Handling uncovered root causes**: If the "Fix Recommendations" section of the problem domain file does not have a fix matching the current root cause, MUST truthfully inform the user "No matching fix found in the documentation" and seek the user's opinion; **prohibited from generating self-created fix commands based on experience**
4. Provide executable offline fix commands for each root cause, with verification and expected results
5. Present in priority order, show complete fix content to user, wait for explicit confirmation — automatic execution prohibited. After presenting the plan (including risk notes and confirmation request), END the current turn; execution may start only after the user's explicit confirmation reply in a later turn — presenting the plan and executing the fix in the same turn is prohibited
6. When registry or file modifications are involved, MUST first back up the `\Windows\System32\config` directory and annotate operational risks
7. **Risk notes (mandatory)**: Each fix item MUST include "Risk notes", clearly explaining to the user: possible side effects, irreversible consequences, impact on other components, and execution prerequisites. Even if the fix is a low-risk operation, it must explicitly state "Low risk" and the reason. Omitting risk notes is prohibited
8. **Fix Impact Standard Fields (required)**: The risk notes section of each fix plan MUST include the following three items:
   - **Session impact**: Whether existing TCP/RDP sessions will be disconnected; offline fixes executed in the offline environment, typically annotated as "No impact on existing sessions (executed in offline environment)"
   - **Persistence scope**: "Persist across reboot" / "Written to registry" / "Written to disk file" (offline fixes default to persist across reboot)
   - **Rollback command**: A one-line copyable reversible operation; when truly irreversible, MUST explicitly annotate "Irreversible" with backup and restore method

### Diagnostic Cleanup

- After the entire diagnostic flow ends (whether or not fixes were executed), MUST execute the [registry.md](references/offline/registry.md) "HIVE Unload" script to unload all loaded registry HIVEs
- Failure to unload HIVEs will leave file handles locked, causing subsequent disk detach failures or data corruption risk
- MUST execute the cleanup script per [dism.md](references/offline/dism.md) "Standard Disk Cache Pattern" to delete disk cache written during diagnostics

---

## Offline Problem Domain Routing

Problem classification in offline mode MUST be based on this section. There is only one main domain in offline mode:

| Problem Domain | Unique Identifier | Unified Troubleshooting Sequence | Covered Sub-scenarios |
| --- | --- | --- | --- |
| Instance is "Running" but GuestOS has not booted normally | `GuestOS.SystemBootstrapFailed` | [boot-triage.md](references/offline/boot-triage.md) evidence pre-assessment → confirm offline system disk is mounted to current instance → offline fixed prerequisite chain ([environment.md](references/offline/environment.md) → [disk-partition.md](references/offline/disk-partition.md) → [registry.md](references/offline/registry.md)) → refined routing table domain selection (see below) | Boot phase anomalies (bootmgr/BCD/MBR-VBR errors), OS loading phase anomalies (BSOD, spinning, stuck at Logo), boot loop, automatic repair loop, safe mode loop |

> The domain phrasing "Instance is 'Running' but GuestOS has not booted normally" is not contradictory: console instance status reflects the virtualization-layer lifecycle — "Running" only means the platform has provisioned hardware resources and powered on the instance, i.e., it has at least *begun attempting* to load the OS; it does not guarantee that Windows booted or that services are usable. Underlying hardware or software faults, system misconfiguration, or file corruption can leave the GuestOS down while the platform status stays Running, which is exactly why the faulty disk can be detached and mounted elsewhere for this offline diagnosis. The direct evidence of where boot stopped is the console VNC output (boot phase anomaly vs OS loading phase anomaly), which [boot-triage.md](references/offline/boot-triage.md) uses for evidence pre-assessment. Conversely, an instance stuck in "Starting" for an abnormally long time is a platform-side startup anomaly (non-GuestOS primary cause) and does not enter this offline flow.

> BitLocker offline branch (`GuestOS.BitLockerLocked`, offline mounted partition is unreadable / recovery mode after instance type change): directly execute [bitlocker.md](references/offline/bitlocker.md) (VBR signature detection; upon confirming encryption, terminate diagnostics and inform user); for online branch see [WORKFLOW-GUIDE.md](references/online/WORKFLOW-GUIDE.md).

> Crash / Hang offline branch (`GuestOS.Crash` / `GuestOS.Hang`, cannot boot after crash/hang or repeated BSODs): first execute [crash-hang.md](references/offline/crash-hang.md) Step 1/2 (crash analysis / Hang core collection), then complete the offline fixed prerequisite chain, followed by Step 3 offline general analysis chain ([driver.md](references/offline/driver.md) → [device-tree.md](references/offline/device-tree.md) → [system-config.md](references/offline/system-config.md)) for cross-checking; for online branch see [WORKFLOW-GUIDE.md](references/online/WORKFLOW-GUIDE.md).

### Refined Routing Table

After matching `SystemBootstrapFailed`, refine the troubleshooting sequence by the following keywords (matching rule: exact match → fuzzy match → fallback mechanism):

| Typical Description / Keywords | Scenario-Specific Troubleshooting Sequence |
| --- | --- |
| System disk missing, offline cannot see system disk, system disk not mounted, No bootable device (firmware phase), cannot find boot disk | Confirmation **before** mounting offline disk (WORKFLOW-GUIDE "Prerequisite Check → Offline Disk Visibility Confirmation": `Get-Disk` shows no target system disk → root cause "system disk not mounted to instance", prompt user to verify mount status in console); if management plane confirms existence but not visible offline, `environment.md` Step 3 disk not-present determination serves as fallback confirmation; MUST precede BCD / partition-type checks — without a target disk, all subsequent checks are invalid |
| BSOD STOP 0x7B, INACCESSIBLE_BOOT_DEVICE | `driver.md` → `device-tree.md` (when occurring after instance type change, first verify image and target type compatibility) |
| BSOD STOP 0xC000021A | `system-config.md` |
| Stuck at Windows Logo / boot loop, spinning, repeated restarts | `bcd-boot.md` → `driver.md` |
| Black screen with no output | `bcd-boot.md` |
| BCD, 0xc000000e, 0xc000000f, 0xc0000098, winload, bootmgr | `bcd-boot.md` |
| BOOTMGR is missing / compressed, NTLDR is missing, Missing operating system, Invalid partition table | `bcd-boot.md` |
| BSOD STOP 0x7E / 0x7F | `driver.md` → `system-config.md` |
| Network unreachable after boot, IP lost, NIC disappeared | `network.md` |
| VirtIO, viostor, vioscsi, disk driver missing | `driver.md` |
| BSOD after update, cannot boot after patch, rollback failed | `bcd-boot.md` → `driver.md` → `update.md` |
| Sysprep, cannot boot after sealing, OOBE failed | `system-config.md` |
| vminit, cloud assistant, AliyunService, password reset not taking effect (offline) | `cloud-agent.md` |
| Filter driver, UpperFilters, LowerFilters, third-party driver residuals | `driver.md` → `network.md` |
| Offline mounted partition cannot be read / shows encryption, recovery key | `bitlocker.md` |
| Enters safe mode on every boot, SafeBoot residual | `safeboot-winre.md` |
| Automatic repair loop, WinRE stuck | `safeboot-winre.md` |
| BitLocker recovery mode after instance type change | `bitlocker.md` |
| Cannot boot after crash, cannot boot after Hang/frozen, repeated BSODs (no clear STOP code), NMI, dump analysis | `crash-hang.md` (first crash analysis, then follow Step 3 general chain) |
| Cause unknown, comprehensive check | `bcd-boot.md` → `driver.md` → `device-tree.md` → `network.md` → `system-config.md` → `update.md` → `cloud-agent.md` |

## Fallback Mechanism

When no refined routing match is found, or the sequence completes without identifying a root cause:

1. **Dynamic planning**: Combine boot-triage evidence pre-assessment conclusions and user description to select relevant troubleshooting documents from the "Diagnostic Capability Declaration" to compose a sequence; MUST inform the user: "This is a dynamically planned troubleshooting path, not a preset scenario"
2. **Capability boundary**: If comprehensive troubleshooting still cannot confirm a root cause, or all references are irrelevant → truthfully record the scope investigated and all findings, inform the user that current diagnostic capabilities do not cover this scenario, provide suggested troubleshooting directions and transfer to expert support

---

## Data Collection and Cross-Step Caching

Except for the three prerequisite chain files (`environment.md` / `disk-partition.md` / `registry.md`), all troubleshooting files' collection scripts assume by default that the global diagnostic context has been generated during the "Environment Context Initialization" phase, and **directly use public constants and cached data retrieved from the context** without re-declaration.

### Global Context Constant Fields

After the fixed prerequisite chain is completed, the global diagnostic context should contain the following constants (produced by each step of the prerequisite chain):

| Field | Production Source | Semantics |
|------|---------|------|
| BootLetter | disk-partition.md Step 2 | Boot partition drive letter (containing `\Windows`) |
| SystemLetter | disk-partition.md Step 3 | System partition drive letter (active / ESP partition containing bootmgr/BCD) |
| BootMode | disk-partition.md Step 1 | `UEFI` or `BIOS` |
| DiskNumber | environment.md | Target disk number |
| GUID | registry.md Step 2 | Fixed value `{bf1a281b-ad7b-4476-ac95-f47682990ce7}` |
| CcsPath | registry.md Step 3 | Full registry path of the active ControlSet |
| SoftPath | registry.md Step 2 | Full registry path of the SOFTWARE HIVE |
| SysPath | registry.md Step 2 | Full registry path of the SYSTEM HIVE |
| BcdPath | registry.md Step 2 | Absolute path of the BCD file (differs for UEFI/BIOS templates) |
| CsName | registry.md Step 3 | Active ControlSet name (e.g., `ControlSet001`) |

Fields in the table appear as `<FieldName>` placeholders in subsequent troubleshooting document scripts; replacement constraints are described in the "Environment Context Initialization - Execution Constraints - Scalar Constants" section.

### Cross-Step Data Caching

Mechanism rules (dual-track reuse, cache directory, reuse mode, HIVE reload, fix block exception) are described in the "Environment Context Initialization - Execution Constraints - Large Objects" section; disk write and cleanup script templates are in [dism.md](references/offline/dism.md) "Standard Disk Cache Pattern". This section only registers the cache keys under this mechanism:

| Cache Key | Data Source | First Collector | Reuser |
|--------|----------|------------|--------|
| `WindowsDriver` | `Get-WindowsDriver -Path "<BootLetter>:\"` | driver.md Step 2 / network.md Step 1 (first-come-first-collected) | driver.md / network.md subsequent steps |
| `WindowsPackage` | `Get-WindowsPackage -Path "<BootLetter>:\"` | update.md Step 2 | update.md Step 4 / same troubleshooting document subsequent steps |
| `BcdEnumAll` | `bcdedit /store <BcdPath> /enum all` | safeboot-winre.md Step 1 | safeboot-winre.md subsequent steps |

## Fix Phase Execution Convention

When generating fix commands based on root causes, MUST comply with the fix contract defined in the "Fix Plan" section.

---

## Output Format Templates

### Evidence Review Template

Present this before the diagnostic conclusion:

> **Evidence Review**
>
> **Data Collected**:
>
> | # | Collection Action | Key Result |
> |---|-------------------|------------|
> | 1 | {command or check description} | {brief summary of what was returned} |
> | 2 | ... | ... |
>
> **Judgment Traceability**:
>
> | Judgment | Supporting Evidence | Confidence |
> |----------|---------------------|------------|
> | {conclusion from causal chain} | {specific data point that directly confirms it} | Confirmed |
> | {inferred conclusion} | {what was observed that led to this inference} | Hypothesis — missing: {data items}; collect via: `{commands}` |
>
> **Unsupported Judgments**: {list any judgments downgraded to hypotheses, or "None — all conclusions are evidence-backed"}

### Causal Chain Example

```
[User Problem] Instance shows BSOD STOP 0x7B after boot
    |
    +-- [Direct cause] viostor service Start=4 (disabled), cannot load storage driver at boot
    |
    +-- [Indirect cause] Third-party antivirus XxxFilter residual in DiskDrive class UpperFilters
            +-- Driver binary deleted but registry not cleaned
```

### Diagnostic Conclusion Output Template

> **Diagnostic Conclusion**
>
> **User Problem**: {original problem description}
>
> Found {N} issues, sorted by fix priority:
>
> ---
>
> **Issue 1 (Direct cause, Critical, will prevent boot): {root_cause}**
>
> **Evidence**: {collected abnormal data}
>
> **Analysis**: {why this problem caused the phenomenon the user observed}
>
> **Causal Chain**: {user problem} <- {direct cause} <- {indirect cause (if any)}
>
> **Fix Plan**:
> ```powershell
> {offline fix commands}
> ```
>
> **Risk Notes**: {side effects, irreversible consequences, impact on other components, execution prerequisites; low risk must also be declared with reason}
>
> **Fix Impact**:
> - Session impact: {whether existing sessions are disconnected; offline fixes typically annotated as "No impact on existing sessions (executed in offline environment)"}
> - Persistence scope: {"Persist across reboot" / "Written to registry" / "Written to disk file"}
> - Rollback command: {one-line copyable reversible operation; if irreversible, annotate and provide backup restore method}
>
> **Verification**: Observe whether the system boots normally after restart
>
> ---

### Check Item Summary Template

After completing each diagnostic step, present a brief summary to the user before proceeding:

> **Check Item: [Step Name]**
> - **Collection**: [key data collected, abbreviated]
> - **Analysis**: [one-sentence determination basis]
> - **Result**: Normal / Abnormal [if abnormal: brief description]

---

## PowerShell Collection Script Rules

This section collects medium-to-low frequency pitfalls and style conventions that need to be avoided when writing offline diagnostic PowerShell collection scripts. MUST refer to this section before generating or adjusting collection scripts.

### 0. Pre-Execution Self-Check Checklist

After writing a script and **before** executing it in the offline environment, MUST scan the script text item by item against the table below; if any signal matches, fix per the corresponding section before executing.

This step cannot be skipped: violations of these rules almost always manifest as PowerShell execution errors, Chinese garbled text, or output truncation, not locally visible syntax errors. Waiting until the command fails to look up rules: first, it wastes a round trip (offline environment round trips are more costly); second, script-layer errors are easily misread as "abnormalities" in the offline system itself, skewing root cause determination. Scanning item by item takes only a few seconds, and the benefit far exceeds the cost.

| # | What to search for in the script | What to do when matched |
|---|---|---|
| 1 | `Get-WmiObject`, `reg query`, `bcdedit`, `diskpart`, `dism`, `attrib` | Replace with native cmdlet per §1 mapping table; only keep and pass through raw output for items marked "No native alternative" |
| 2 | `Get-WindowsDriver` | Confirm `-All` was not added casually; only add when inbox driver troubleshooting is genuinely needed, and explain the reason in comments, see §1 |
| 3 | Statements reading HIVE / BCD / bootmgr files | Confirm `-Force` is included (Hidden+System attribute files cannot be read without it), see §1 |
| 4 | Any pipeline outputting to console | Confirm `Select-Object` is used to limit key fields, see §2 |
| 5 | Pipeline produced by `Select-Object` / `ForEach-Object { [PSCustomObject]@{...} }` | Append `| Format-Table -AutoSize` or `| Format-List` at the pipeline end; prefer `Format-List` for long fields to prevent truncation, see §2 |
| 6 | `Get-ItemProperty` | Confirm metadata fields like `PSPath` are filtered out, see §2 |
| 7 | `Get-WindowsOptionalFeature`, `Get-WindowsFeature`, `Invoke-WebRequest`, `Invoke-RestMethod`, `Expand-Archive`, `Copy-Item -ToSession/-FromSession` | Add `$ProgressPreference = 'SilentlyContinue'` before them, see §2 |
| 8 | Custom variable names | Compare against built-in keywords / automatic variables list in §3; rename if collision |
| 9 | exe calls that need to read Chinese output (`bcdedit` / `reg` / `diskpart`) | Use `ProcessStartInfo` to explicitly specify OEM codepage for capture, see §4 |
| 10 | `UpperFilters`, `LowerFilters`, `ServiceGroupOrder`, `DependOnService` | Handle as `string[]` array operations; write back with explicit `-Type MultiString`, see §5 |
| 11 | `$variable:` in double-quoted strings | Rewrite as `${variable}:`, see §6 |
| 12 | `foreach (` statement's closing brace immediately followed by `|` | Use `ForEach-Object` instead, see §7 |
| 13 | exe calls like `reg`, `bcdedit`, `diskpart`, `dism` | Confirm using `$LASTEXITCODE` or stderr for error detection instead of `try/catch`, see §8 |
| 14 | `Substring(`, `CurrentVersion`, `ProductName` | Change to version number comparison or SKU branching; add null check for `CurrentVersion`, see §9 |
| 15 | `Set-Content`, `[System.IO.File]::WriteAllText` | Confirm the same file is not written using both methods, see §10 |
| 16 | `ConvertTo-Json` | Add `-Depth 4` (default 2 serializes nested objects as `"System.Object[]"`), see §11 |
| 17 | Commands prepared for user manual execution | Before display, pass through §12 display gate (pure ASCII + placeholder compliance + no local path dependencies + copy-paste executable as a whole); if any item fails, fix before display |
| 18 | `-ErrorAction SilentlyContinue`, `2>$null`, `\| Out-Null`, `2>/dev/null` | Applies to BOTH Local and Remote mode payloads. Suppression is allowed only for state-table collection where absence of output is itself the finding; otherwise keep stderr, error text, and exit codes in the returned output (`$LASTEXITCODE` locally, `ExitCode` + `Output` remotely) and analyze them as potential root cause — "cmdlet not found" → OS-version boundary, "Access is denied" → permission root cause, "RPC server is unavailable" → service dependency root cause; related exe-call rule in §8, and multi-step scripts must use the §8 try/catch section guards instead of any suppression |
| 19 | Double-quoted strings in native CLI invocations (`reg`, `bcdedit`, `dism`, `diskpart` arguments) | PowerShell 5.1 strips double quotes when passing arguments to native CLIs — offline scripts call these tools heavily (`reg load`, `bcdedit /store`, DISM cmdlets), so the trap applies the same as online; rewrite the script to contain single-quoted strings only and retry on the same channel (script-layer error, not an offline-disk fault), see High-Frequency Pitfalls |

### 1. Cmdlet Selection

- Prefer PowerShell native cmdlets over cmd tools to avoid depending on cmd text output (field names vary with system language)
- `Get-WmiObject` is deprecated on some systems; prefer `Get-CimInstance`

#### cmd → PowerShell Replacement Mapping (Offline Common)

| cmd Command | PowerShell Replacement |
|---|---|
| `reg query` | After HIVE is loaded, use `Get-ItemProperty` / `Get-Item` |
| `reg load` / `reg unload` | Must use `reg.exe`, no native alternative |
| `bcdedit` | No native alternative, pass through raw output |
| `diskpart` | Prefer `Get-Disk` / `Get-Partition`; fall back to `diskpart` for dynamic disk scenarios |
| `dism` | `Get-WindowsPackage` / `Get-WindowsDriver` |
| `attrib` | `Get-Item -Force` / `Set-ItemProperty`; reading Hidden+System files MUST add `-Force` |

When no native alternative exists → pass through raw output to LLM for analysis; do not parse text fields in scripts.

#### Get-WindowsDriver Should Not Add `-All` by Default

`Get-WindowsDriver -Path X:\` by default returns only third-party / OEM driver packages (`oem*.inf`), covering VirtIO, network, storage, and all user-installed drivers; adding `-All` additionally returns hundreds of inbox drivers, increasing output volume by 10x or more with no diagnostic benefit.

Only add `-All` when troubleshooting Windows inbox driver itself missing / corrupted / tampered (e.g., inbox `disk.sys` overwritten causing boot failure), and MUST explicitly state so in the collection code block comments.

### 2. Output Style

#### Command Output Format Simplification

Prefer using `Select-Object` to output only key fields, avoiding verbose full object output.

- Recommended: `Get-WindowsDriver -Path X:\ | Select-Object Driver, ProviderName, Version`
- Not recommended: `Get-WindowsDriver -Path X:\` (includes many irrelevant fields)

#### Format-Table / Format-List Selection

Objects returned by `Select-Object` enter the deferred formatting queue; subsequent `Write-Host` may reach the console before the table, causing output order confusion. Any pipeline producing `[PSCustomObject]` sequences (`Select-Object` / `ForEach-Object { [PSCustomObject]@{...} }` / cmdlets returning object collections) MUST append `| Format-Table -AutoSize` or `| Format-List` at the **pipeline end** to force synchronous rendering.

In non-interactive execution environments, the output buffer is typically only 80~120 columns; long fields will be truncated by `Format-Table` (with `...` in the middle); `-AutoSize` only changes column width allocation strategy and **cannot break through the buffer limit**. `Format-List` uses one field per line, unconstrained by width.

Collection scripts should choose `Format-Table -AutoSize` or `Format-List` appropriately based on actual field width and column count.

#### Get-ItemProperty Output Filtering

`Get-ItemProperty` by default returns metadata fields including `PSPath`, `PSParentPath`, `PSChildName`, `PSDrive`, `PSProvider`, which interfere with diagnostic output. MUST choose one of:

1. `| Select-Object <target properties>` (recommended, when property names are known)
   ```powershell
   Get-ItemProperty 'HKLM:\{bf1a281b-...}\ControlSet001\Services\viostor' -Name Start, ImagePath |
       Select-Object Start, ImagePath
   ```
2. `| Select-Object -Property * -ExcludeProperty PSPath,PSParentPath,PSChildName,PSDrive,PSProvider` (when all registry values are needed)

Directly outputting the full result of `Get-ItemProperty` is prohibited.

#### Suppress Progress Stream in Remote Execution

In remote execution, some cmdlets' Progress stream mixes into CLIXML metadata interfering with output parsing. When using the following cmdlets, MUST add `$ProgressPreference = 'SilentlyContinue'` before them:

| cmdlet | Trigger Scenario |
|---|---|
| `Get-WindowsOptionalFeature` / `Get-WindowsFeature` | Enumerate features / roles |
| `Invoke-WebRequest` / `Invoke-RestMethod` | Download content |
| `Expand-Archive` | Extract files |
| `Copy-Item -ToSession/-FromSession` | Cross-session copy |

### 3. Variable Naming: Avoid Built-in Identifiers

Custom variable names MUST avoid built-in keywords (`switch`, `foreach`, `function`, etc.) and automatic variables (`$_`, `$input`, `$args`, `$error`, `$host`, `$pwd`, `$foreach`, `$switch`, `$null`, `$true`, `$false`, etc.), otherwise it will cause abnormal script behavior or variable value override.

### 4. cmd Tool Invocation and Encoding Handling

When cmd tools like `bcdedit` / `reg` / `diskpart` are invoked with PowerShell's default capture mode, Chinese fields may be garbled or truncated. When stable capture of Chinese OEM output is needed, MUST use `System.Diagnostics.ProcessStartInfo` to explicitly specify output encoding:

```powershell
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = 'bcdedit.exe'
$psi.Arguments = '/store X:\Boot\BCD /enum all'
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true
$psi.UseShellExecute = $false
$psi.StandardOutputEncoding = [System.Text.Encoding]::GetEncoding(936)
$psi.StandardErrorEncoding  = [System.Text.Encoding]::GetEncoding(936)
$proc = [System.Diagnostics.Process]::Start($psi)
$stdout = $proc.StandardOutput.ReadToEnd()
$stderr = $proc.StandardError.ReadToEnd()
$proc.WaitForExit()
```

Chinese Windows OEM codepage is `936` (GBK); English systems are typically `437` or `1252`; select based on the target offline environment's actual codepage.

### 5. REG_MULTI_SZ Multi-Value Parsing

In the offline registry, fields such as `UpperFilters` / `LowerFilters` / `ServiceGroupOrder` / `DependOnService` are `REG_MULTI_SZ`; `Get-ItemProperty` returns them as `string[]` after reading, MUST be handled with array operations:

```powershell
$filters = (Get-ItemProperty '...\Class\{4d36e967-...}' -Name UpperFilters).UpperFilters
if ($filters -contains 'XxxFilter') { ... }
$newFilters = @($filters | Where-Object { $_ -ne 'XxxFilter' })
Set-ItemProperty '...' -Name UpperFilters -Value $newFilters -Type MultiString
```

Using `-eq` for overall comparison or string concatenation will produce incorrect results; when writing back, MUST explicitly use `-Type MultiString`, otherwise it degrades to `REG_SZ`.

### 6. Variable Name Followed by Colon in String Interpolation

`"$var:"` will be parsed as a drive-qualified variable reference (e.g., `$env:`, `$global:`), and the variable value will not expand. MUST use curly braces:

```powershell
# Wrong
"$name: OK"
# Correct
"${name}: OK"
```

The same applies when other special characters (`(`, `[`) immediately follow the variable name.

### 7. foreach Statement Cannot Directly Connect to Pipeline

`foreach ($x in $list) { ... }` is a **statement**, not an expression; appending `| Format-Table` / `| Where-Object` / `| Select-Object` and other pipeline operations after the closing brace will report "An empty pipe element is not allowed". MUST use `ForEach-Object` cmdlet instead:

```powershell
# Wrong
foreach ($f in $files) {
    [PSCustomObject]@{ File = $f }
} | Format-Table

# Correct
$files | ForEach-Object { [PSCustomObject]@{ File = $_ } } | Format-Table -AutoSize
```

For any scenario where loop results need to be uniformly formatted for output, always use `ForEach-Object`.

### 8. cmd Tool Error Handling Using $LASTEXITCODE

`reg.exe` / `bcdedit.exe` / `diskpart.exe` / `dism.exe` and other cmd tools do not throw PowerShell exceptions; errors are only indicated by non-zero exit codes. `try/catch` cannot catch them; MUST check `$LASTEXITCODE` or capture stderr:

```powershell
$out = reg load "HKLM\$path" $file 2>&1
if ($LASTEXITCODE -ne 0) { Write-Host "reg load failed: $out" }
```

#### Section Guard: try/catch Keeps Collection Running

exe tools need `$LASTEXITCODE` (above); PowerShell cmdlet errors need the opposite — a section-level `try/catch` guard. Collection/fix steps are independent: one failing check must never abort the rest of the script. Multi-step PowerShell written for offline flows MUST follow this shape and MUST NOT use `-ErrorAction SilentlyContinue` — the error text is diagnostic evidence, and a swallowed error is a lost signal:

```powershell
$ErrorActionPreference = 'Stop'

# --- Step 1: ... ---
try {
    # cmdlet collection / fix body
} catch {
    Write-Host ("ERROR step1 <tag>: " + $_.Exception.Message)
}

# --- Step 2: runs even if Step 1 failed ---
```

With `Stop` preference the error jumps to that section's catch, gets an `ERROR step<N> <tag>:` prefix, and the next section still runs; without a guard a terminating error kills the script and every later step is lost.

For scenarios requiring simultaneous recording of stdout/stderr, see section 4.

### 9. Version String Parsing

Strings like `CurrentVersion` / `ProductName` have inconsistent formats across versions (`"6.1"` vs `"10.0"`); **prohibited** from using `Substring(0, N)` or fixed indices, which will throw `ArgumentOutOfRangeException` on short strings. MUST choose one of:

```powershell
# Pattern A: Version number comparison
if ([version]$cv.CurrentVersion -ge [version]'6.2') { ... }
# Pattern B: Product SKU branching
if ($cv.ProductName -match 'Server 2008 R2|Windows 7') { ... }
```

When `CurrentVersion` is not read, MUST first `if ($cv.CurrentVersion) { ... }`, otherwise `$null` will cause subsequent exceptions.

### 10. UTF-8 BOM Compatibility

Windows PowerShell 5.x `Set-Content -Encoding UTF8` writes **with BOM** (`UTF8` = UTF-8 with BOM, no `utf8NoBOM` option); PowerShell 7+ `-Encoding UTF8` writes **without BOM** (`utf8` = utf8NoBOM alias), needs `-Encoding UTF8BOM` to align with PS5; PS 7+ defaults to `utf8NoBOM` when `-Encoding` is not specified.

- `ConvertTo-Json | Set-Content -Encoding UTF8` can be read back normally by `Get-Content -Raw | ConvertFrom-Json` in both PS5/PS7 (BOM-tolerant), no special handling needed
- Mixing `Set-Content` and `[System.IO.File]::WriteAllText` on the same file across scripts is prohibited; BOM will be written repeatedly
- Strictly no BOM: `[System.IO.File]::WriteAllText($path, $content, [System.Text.UTF8Encoding]::new($false))` (`$false` = do not emit BOM)

### 11. ConvertTo-Json Depth Limit

`ConvertTo-Json` defaults to `-Depth 2`; nested objects beyond the depth are serialized as `"System.Object[]"`. `Get-WindowsDriver` / `Get-WindowsPackage` returned objects contain multiple nesting levels (e.g., `CustomProperties`); JSON written to disk MUST explicitly use `-Depth 4` (consistent with dism.md "Standard Disk Cache Pattern").

### 12. User Command Display Gate (MUST Verify Before Display)

For any scenario that degrades to "display command for user manual execution in offline environment", the command MUST pass the following gate checks **before** being displayed to the user; if any item fails, fix before display — the user's offline environment cannot tolerate like the direct execution channel; one non-compliant command will directly stall the diagnostic flow:

1. **Pure ASCII**: The entire command text (script comments, separator markers like `# --- <item-name> ---`, placeholders and description text) MUST be ASCII characters; Chinese comments, Chinese prompts, and full-width symbols are prohibited. Reason: the user's offline terminal encoding environment is uncontrollable (GBK/UTF-8); non-ASCII characters are prone to garbled text after copy-paste, which may cause PowerShell parsing failure
2. **Placeholder compliance**: Values the user needs to replace (e.g., target drive letter `<BootLetter>`) MUST be explicitly marked with `<placeholder>` and explained outside the command what to replace; residual internal variables from diagnostic context or temporary values from prior collection are prohibited
3. **No local path dependencies**: The command must not reference local temporary files/paths on the diagnostic side; all input must come from the offline environment itself or be inline in the command
4. **Copy-paste executable as a whole**: When merging multiple commands, maintain clear separator markers; individual commands must not depend on the previous command's shell state (each is self-contained)

```powershell
# bad - non-ASCII comments break in some terminals
# Check offline registry HIVE file integrity
Get-ChildItem "$bootLetter\Windows\System32\config" -Force

# good - pure ASCII
# Check offline registry HIVE file integrity
Get-ChildItem "$bootLetter\Windows\System32\config" -Force
```

### Offline Environment Specific Rules

#### Offline Environment Access Restrictions

- **Prohibited from using online commands**: `Get-Service` / `Get-Process` / `Get-NetAdapter` / `Get-WmiObject` and other commands querying the running system are meaningless in offline diagnostics and may pollute conclusions
- **Registry access path**: All reg query MUST use the `HKLM\{bf1a281b-...}\...` prefix path; accessing `HKLM\SYSTEM` and other currently running system registries is prohibited
- **Offline paths and hidden files**: Registry ImagePath must be converted to offline disk absolute paths before verification; reading files with Hidden+System attributes such as BCD / bootmgr / HIVE MUST use `-Force`

#### Dynamic Disk Fallback to diskpart

Dynamic Disks cannot be queried through PowerShell's `Get-Disk` / `Get-Partition` / `Get-Volume` and other Storage module cmdlets; when Storage cmdlets return empty or error, MUST fall back to `diskpart` to complete disk management operations (list disks, partition identification, attribute modification, drive letter assignment, etc.).

#### Command Not Found Staircase Handling

When a cmd tool or PowerShell cmdlet reports "command/module not found", the environment likely does not support that tool; **retrying the original command is prohibited**; MUST handle per the following staircase:

1. **Try same-semantics alternative tool**: Select another tool that can achieve the same collection goal
2. **Skip and record**: When all alternatives are unavailable, record "This check item was skipped due to tool unavailability" and continue with subsequent steps

### High-Frequency Pitfalls

#### PowerShell and cmd Redirection Syntax Mixing

In PowerShell, discarding output MUST use `$null` instead of cmd's `nul`; using `>nul` / `2>nul` / `1>nul` and other cmd redirection syntax will trigger `RedirectionFailed` error.

#### Command Parameters with Curly Braces MUST Be Quoted

PowerShell treats bare `{...}` as a script block; when passed to native cmd/exe tools, it is not passed as a literal string, causing original command parameter parsing failure. Literals containing curly braces MUST be wrapped in **single quotes** (preferred) or **double quotes** — single quotes are preferred because PowerShell 5.1 strips embedded double quotes in native CLI arguments (see the pitfall below), e.g., `bcdedit /enum '{default}'`.

#### PowerShell 5.1 Strips Double Quotes in Native CLI Argument Passing

Windows Server images ship with PowerShell 5.1 by default; when passing arguments to native executables, embedded **double quotes are stripped** and never reach the target CLI. Offline collection and fix scripts call native tools constantly (`reg load`/`reg unload` with HIVE paths, `bcdedit /store <BcdPath>`, DISM cmdlets, `diskpart`), so a quoted path or GUID argument silently loses its quotes. Symptom: the call fails with parameter-parsing errors — "The syntax of the command is incorrect", paths split at spaces, flags reported as missing — even though the command text looks correct. This is a **script-layer error**, not a fault in the offline disk: do NOT reclassify it as target abnormality and do NOT switch channels. Fix: rewrite the script so it contains **single-quoted strings only**, then retry on the **same channel** (per the Collection Fallback Chain). When a value genuinely needs quoting for the target CLI, prefer restructure-into-single-quotes over embedded double quotes. This is the target-side twin of the operator-side rule in [REMOTE-EXECUTION.md](references/REMOTE-EXECUTION.md) §Operator-Shell Quoting for JSON Arguments — same mechanism, different side of the connection.

#### Repeated Queries of Same Object Prohibited in Same Code Block

In the same PowerShell code block, multiple calls to the same cmdlet with the same parameters (e.g., `Get-Disk -Number $n`, `Get-Partition -DriveLetter $x`, `Get-CimInstance Win32_*`) MUST be merged into a single query and stored in a variable; subsequent access through variable properties or in-memory filtering.

**Exception**: Re-querying after executing a modification operation (e.g., `Set-Disk`) to verify modification results is a reasonable verification call.

#### Parsing cmd Tool Raw Output Prohibited

`bcdedit` / `diskpart` and other cmd tools' field names / prompt messages vary with system language; using `Select-String`, regex, etc. to extract specific fields is prohibited; raw output should be passed directly to the LLM for analysis.

#### Windows Boot Terminology Confusion Points (System / Boot Partition)

Windows official boot terminology is **opposite** to literal intuition:

- **System Partition** = the small partition containing bootmgr/BCD (ESP for UEFI, System Reserved for BIOS) → "System partition drive letter" in the global context
- **Boot Partition** = the partition containing `\Windows` (where OS files reside) → "Boot partition drive letter" in the global context
- BCD field mapping (remember): `{bootmgr}` `device` → System Partition; `{default}` / OS Loader `device` and `osdevice` → Boot Partition

The everyday Chinese phrase for "system partition" often refers to the Windows partition, which is opposite to official terminology. When fixing BCD, **use the path semantics in the global context as the authority**; do not infer based on the literal Chinese; any fix command should prioritize reusing the script template from the problem domain file's "Fix Recommendations".
