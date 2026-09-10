# Platform-Side Data Cross-Validation (Online Only)

## Function Description

When online diagnosis runs over the remote execution channel, the aliyun CLI can additionally fetch platform-side data — instance monitoring metrics, disk monitoring metrics, system events, and console screenshots — as **auxiliary cross-validation evidence** alongside in-instance (GuestOS) collection. The platform sees the instance from the virtualization layer; comparing the two views either corroborates in-instance findings, bounds them (e.g., resource throttling at spec limits), or exposes discrepancies that are themselves diagnostic clues.

**Scope constraints (MUST observe)**:

1. **Online diagnosis scope**: This file is part of the online diagnostic flow and is loaded only by the online WORKFLOW-GUIDE.
2. **Remote execution channel only**: These are OpenAPI calls made from the machine running the aliyun CLI. In direct execution channel mode (agent inside the GuestOS), skip platform-side collection entirely; do not ask the user to install/configure the CLI just for cross-validation.
3. **Auxiliary, not primary**: Platform data corroborates, bounds, or timestamps in-instance findings. It NEVER replaces intra-instance collection and NEVER changes problem domain classification. When platform data and in-instance data conflict, keep both and present the discrepancy honestly in the Evidence Review.
4. **Per-domain triggering**: Only fetch the APIs matching the current problem domain group (table below); do not pull everything for every case.

## API Mapping by Problem Domain Group

| Domain Group | CLI (plugin mode) | Data Returned | Cross-Validation Value |
| --- | --- | --- | --- |
| Performance (CPU) | `aliyun ecs describe-instance-monitor-data --region <region-id> --instance-id <id> --start-time <iso8601> --end-time <iso8601>` | Sampled CPU utilization, internal/public network BPS and PPS | Whether high CPU observed in-instance is also visible platform-side, and whether the time profile matches the user-reported fault window |
| Network | `aliyun ecs describe-instance-monitor-data` (same parameters) | Network BPS/PPS samples | Whether packet loss / slowdown coincides with bandwidth or PPS saturation (spec throttling) versus a GuestOS configuration issue |
| Storage / disk performance | `aliyun ecs describe-disk-monitor-data --region <region-id> --disk-id <disk-id> --start-time <iso8601> --end-time <iso8601>` | Per-disk IOPS, BPS, latency | Whether slow disk I/O is cloud-disk throttling (approaching the disk performance-level limits) versus a GuestOS-level issue (filter driver, queue depth) |
| All domains (event correlation) | `aliyun ecs describe-instance-history-events --biz-region-id <region-id> --instance-id <id>` | System events: maintenance, live migration, host errors | Whether a platform event coincides with the fault time — turns "sudden unexplained fault" into a correlated finding |
| VNC / screen state | `aliyun ecs get-instance-screenshot --biz-region-id <region-id> --instance-id <id>` | Console screenshot (Base64-encoded JPEG) | Cross-check the actual screen state for VNC black screen / unresponsive-screen cases without asking the user for a manual screenshot |

Note: the two monitor-data subcommands take the global `--region` endpoint override (they have no region request parameter and reject `--biz-region-id`); the other three take `--biz-region-id`. Flag spellings were verified with `aliyun-cli-ecs 0.7.8` — see the CLI flag reference in [REMOTE-EXECUTION.md](references/REMOTE-EXECUTION.md) §CLI Flag Reference, and check `aliyun ecs <subcommand> --help` when a flag is rejected.

**Memory note**: instance basic monitoring does not provide memory utilization — do not chase platform-side memory data; rely on in-instance memory collection in the performance domain files.

## GetInstanceScreenshot Usage Notes

- The instance MUST be in Running state; instances created before 2018-01-01 and retired instance families do not support the API
- The response `Screenshot` field is a Base64-encoded JPEG: decode it to a file and view the image directly (e.g., `base64 -d` or PowerShell `[IO.File]::WriteAllBytes('<path>', [Convert]::FromBase64String('<b64>'))`)
- The screenshot is screen-state cross-validation evidence only
- If the call fails (unsupported instance, permission), fall back to asking the user for a console screenshot — do not block the main diagnostic sequence on it

## Execution Rules

1. Reuse the session context (`RegionId`, `InstanceId`) and the UA/session-id rules of [REMOTE-EXECUTION.md](references/REMOTE-EXECUTION.md) §Observability for every call; these are read-only OpenAPI calls and do not involve Cloud Assistant
2. Time window: align `--start-time`/`--end-time` (ISO 8601) with the user-reported fault window; when the fault time is unknown, use the last 1-3 hours. Note that monitoring data has platform-side sampling granularity — a short spike may not appear in platform samples, so absence of a platform-side spike does not refute an in-instance observation
3. Interpretation discipline: platform metrics are measured at the virtualization/cloud-disk layer with their own aggregation windows; when the two views disagree, first consider measurement layer and aggregation differences, then treat genuine discrepancies as clues (e.g., platform bandwidth pinned at the instance spec cap → spec throttling, not a GuestOS fault)
4. Output: platform findings enter the Evidence Review as evidence items labeled **platform-side**, cross-checked against in-instance items labeled **in-instance**; conclusions supported by only one view MUST keep that limitation visible

## Cross-References

- [REMOTE-EXECUTION.md](references/REMOTE-EXECUTION.md) — remote channel mechanics, UA template and session-id rules
- [ram-policies.md](references/ram-policies.md) — RAM actions required by these APIs (optional, cross-validation use)
- Online WORKFLOW-GUIDE "Step-by-Step Execution" rule 7 — when this file is loaded
