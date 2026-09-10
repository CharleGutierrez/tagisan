# Networking and security

## Contents

- Network zones and capacity
- SandboxGateway
- TrafficPolicy
- Enhanced L7 egress
- Credential injection

## Network zones and capacity

Separate trusted control-plane components, non-managed control components, and untrusted Sandbox compute. Use dedicated sandbox vSwitches and enterprise security groups.

- Enterprise security groups support at most 65,536 ENIs; plan expansion around 80% utilization (about 52,000).
- Create at least one sandbox vSwitch per availability zone and add each new vSwitch to NAT SNAT if public egress is required.
- New vSwitch/security-group annotations affect only recreated sandboxes.
- Update target-side security groups, TrafficPolicy, and NetworkPolicy for every added CIDR.

Baseline sandbox egress: deny metadata `100.100.100.200/32`; allow only required DNS, API Server, controller, identity, gateway, OSS/NAS, and explicit public endpoints; deny RFC1918 networks before optional public allow. Baseline ingress: allow only required controller, gateway, metrics, and health ports.

## SandboxGateway

Manager `>= v0.5.2` can separate control traffic (`api.<domain>` to manager:8080) from data traffic (`*.<domain>` to gateway:7788). Validate gateway-to-Sandbox connectivity before setting manager `dataplaneService=sandbox-gateway`, then verify Ingress/ALB targets and gateway logs. Prepare rollback to `sandbox-manager`.

Routing prefers headers `e2b-sandbox-id` and optional `e2b-sandbox-port` (default 49983), then falls back to host `{port}-{namespace}--{name}.{domain}`. The leftmost DNS label must not exceed 63 characters.

Default documented gateway deployment is three replicas at 4 vCPU/8 GiB. One replica has theoretical 1.5 Gbit/s bidirectional capacity, about 90 MB/s application throughput; benchmark the real workload and scale replica count/resources.

Data-plane authentication:

- Static token (`>= v0.6.6`): header `X-Access-Token`, sourced from `envdAccessToken`; invalid requests return 401.
- JWT (`>= v0.6.8`, Agent Identity `>= 0.4.1-rc.1`): label `security.agents.kruise.io/enable-jwt-auth=true`, header `E2B-Traffic-Access-Token`, sourced from `trafficAccessToken`; invalid requests return 403 and gateway misconfiguration returns 503.

Tokens are secrets. Do not log private SDK fields or CR annotations containing runtime credentials.

## TrafficPolicy

`TrafficPolicy` is namespace scoped; `GlobalTrafficPolicy` is cluster scoped. They select Pods by labels and evaluate priority 1-1000 (smaller first), then rule order; first match wins and unmatched traffic is denied once a policy selects the Pod.

- Base feature requires Poseidon `>= v0.7.0`, IPv4, and cluster `>= 1.24`.
- L4 `ports` requires `>= v0.7.3`; workload selector requires `>= v0.7.6`; reject action requires `>= v0.8.0`.
- Ingress supports CIDR, Service, workload, and ports. Egress additionally supports exact FQDN, but no wildcard FQDN.
- Service/workload peers resolve dynamically and have propagation delay; use CIDR for strict immediate control.
- Enable on newly created ACS Pods with annotations `network.alibabacloud.com/enable-network-policy-agent=true` and `network.alibabacloud.com/network-policy-mode=traffic-policy`.

Always place specific allow/deny rules before catch-all rules. Permit DNS over UDP and TCP 53. Use GlobalTrafficPolicy for baseline metadata/private-network denial and TrafficPolicy for application exceptions.

## Enhanced L7 egress

Set `network-policy-mode=enhanced-traffic-policy` to inject `traffic-proxy`. This combines TrafficPolicy L4 controls with `SecurityProfile`/`GlobalSecurityProfile` L7 controls. Current enhanced L4 enforcement is TCP-only.

The egress gateway and traffic extension evaluate matching profiles by priority, creation time, name, and namespace. Global scope does not imply higher priority. Rules can match domains (wildcards supported), ports, paths (`Prefix`, `Exact`, RE2 `Regex`), methods, headers, and query parameters.

Actions include terminal `block` and `bypass`, and non-terminal `audit` and `tokenTransformation`. MCP tool policy evaluates JSON-RPC `tools/call` only; tool names are exact. Supported documented MCP protocol versions are `2025-06-18` and `2025-11-25`. Missing/unsupported versions default to deny unless explicitly configured to pass through.

For HTTPS L7 controls, configure the documented TLS termination path and restrict included hosts. Validate both an allowed and a blocked request, plus traffic-extension and gateway logs.

## Credential injection

Use `SecurityProfile.actions.tokenTransformation` so the Sandbox holds only placeholder credentials and the egress path injects real credentials.

- `ApiKey` replaces a target header using a Go template such as `Bearer {{ .Token }}`.
- `AliyunSTS` obtains temporary credentials and re-signs supported Alibaba Cloud requests.
- Credential source should be an authorized `CredentialProvider`; API key mode can also reference a Secret.
- Default `failStrategy=Block`; do not change to `Allow` without explicitly accepting credential-bypass behavior.

Supported Alibaba Cloud signing: V3 `ACS3-HMAC-SHA256`, V1 RPC/ROA HMAC-SHA1, and OSS V4. OSS V1 and SLS private signing are unsupported. STS injection requires HTTPS traffic control.

Bind the Sandbox to `AgentIdentity` using `security.agents.kruise.io/agent-name`, authorize the CredentialProvider through `AgentRole` and `AgentRoleBinding`, match a narrowly scoped destination in SecurityProfile, and test with a placeholder. Never use an echo service with a real secret in production validation logs.
