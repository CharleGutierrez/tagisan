# RAM Permissions

GuestOS-level diagnostics (both online and offline, direct execution channel) run PowerShell inside the instance and require **no RAM permissions**. RAM permissions are required **only for the remote execution channel**, where the Alibaba Cloud CLI calls ECS Cloud Assistant APIs from the machine running the agent.

## required_permissions

`ecs:RunCommand` — Send PowerShell diagnostic/fix scripts to the target instance via Cloud Assistant (remote execution channel; the only write-capable action in this list)

`ecs:DescribeInvocationResults` — Poll command execution status and retrieve the Base64-encoded output

`ecs:DescribeInvocations` — List past command invocations for the instance (audit and history; optional)

`ecs:DescribeInstances` — Verify target instance existence, Running state, and OSType before entering the remote channel

`ecs:DescribeRegions` — Enumerate region IDs when the region of the target instance is unknown

`ecs:DescribeInstanceMonitorData` — Fetch instance CPU and network BPS/PPS monitoring data as auxiliary cross-validation evidence in online diagnosis (remote execution channel; optional)

`ecs:DescribeDiskMonitorData` — Fetch per-disk IOPS/BPS/latency monitoring data as auxiliary cross-validation evidence in online diagnosis (remote execution channel; optional)

`ecs:DescribeInstanceHistoryEvents` — Query system event history (maintenance / migration / host error events) for fault-time correlation in online diagnosis (optional)

`ecs:GetInstanceScreenshot` — Retrieve the console screenshot (Base64 JPEG) for screen-state cross-validation in online diagnosis (optional)

## Notes

- The action identifiers above (`ecs:RunCommand`, `ecs:DescribeInstances`, ...) are **RAM policy actions** and use the PascalCase API names. This is a different namespace from the aliyun CLI plugin-mode subcommands and flags (`aliyun ecs run-command`, `--biz-region-id`): in a RAM policy JSON the `Action` element only matches the PascalCase API names, so these identifiers must NOT be converted to kebab-case — a policy declaring `ecs:run-command` matches no action and grants nothing.
- Permissions are declared per-action with no wildcards; grant exactly the actions above rather than a broad system policy.
- All actions are read-only except `ecs:RunCommand`, which delivers scripts that execute as SYSTEM on the target instance. Fix scripts additionally require explicit user confirmation before being sent (SKILL.md Principle 6).
