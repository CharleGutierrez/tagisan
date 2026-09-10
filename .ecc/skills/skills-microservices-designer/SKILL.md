---
name: skills-microservices-designer
description: >-
  Use when the user wants to generate detailed microservice design artifacts —
  OpenAPI specifications, SQL ownership schemas, design documents, and design
  review guidelines — from architectural artifacts (architecture specifications,
  microservices catalog, bounded context map, data ownership matrix) produced by
  the Microservices Architect. Requires a microservices catalog CSV and optionally
  Neo4j graph evidence and MinIO discovery documents.
metadata:
  disable-model-invocation: false
---

# Microservices Designer

Translate per-service architectural intent into detailed, technology-independent design
artifacts. This skill **consumes** architecture specifications produced by the Architect
— it does not create them and does not depend on Spec Kit artifacts.

**Prerequisite chain**: `skills-app-discovery-enhanced` → `skills-microservices-architect` → **this skill**

## When to Use

- Generating OpenAPI 3.0 specifications from service architecture specifications
- Generating SQL ownership and migration scripts per service
- Generating comprehensive design documents per service
- Generating design review guidelines per service
- Enriching all artifacts with Neo4j graph evidence (business rules, method signatures, cross-context deps)

## When NOT to Use

- Creating architecture specifications (use `skills-microservices-architect` first)
- Designing the target microservice architecture from scratch
- Generating implementation plans or implementation tasks (that is a separate concern)

## Inputs

The authoritative input for every service is its **Architecture Specification**. The full
set of inputs consumed by this skill is:

| Artifact | Path | Source |
|----------|------|--------|
| Microservices Catalog | `output/microservices-catalog.csv` | Architect |
| Architecture Specification | `output/architecture/{service-name}-architecture.md` | Architect |
| Graph Evidence Summary | `output/graph-evidence-summary.md` | Architect |
| Bounded Context Map | `output/bounded-context-map.md` | Architect |
| Data Ownership Matrix | `output/data-ownership-matrix.md` | Architect |
| Migration Roadmap | `output/migration-roadmap.md` | Architect |
| Discovery Documents | `temp/discovery/` (or MinIO) | App Discovery |
| Neo4j Graph | live queries | Neo4j |

## Mandatory NFRs (applied to every generated artifact)

All generated services must include:
- **JWT Bearer authentication** on every endpoint
- **Timeout = 10 s** — `x-timeout-ms: 10000` in OpenAPI
- **Retry = 3 attempts** — exponential backoff, initial delay 1 s, max delay 5 s
- **Correlation IDs** — `X-Correlation-ID` header (UUID v4) on every request and response
- **Structured JSON logging** — timestamp, level, service, correlationId, message
- **Health checks** — `GET /health/live` and `GET /health/ready`
- **Audit trail** — `{schema}_audit_log` table; capture INSERT / UPDATE / DELETE
- **Outbox table** — `{schema}_outbox` for Saga / event-driven consistency
- **Evidence traceability** — every endpoint and table annotated with source

## Output Structure

```
output/
  swagger/{service}-openapi.yaml
  sql/{service}-schema.sql
  design/{service}-design.md
  governance/{service}-design-review-guidelines.md
```

---

## Step 1 — Initialise and Read Architect Outputs

1. Use `ask_followup_question` to confirm:
   - Path to microservices catalog CSV (default: `output/microservices-catalog.csv`)
   - Whether Neo4j is available for graph evidence enrichment
   - Whether MinIO is available for discovery documents

2. Use `read_file` to load the CSV. Parse each row — the canonical columns are:
   `Service Name`, `Service Type`, `Bounded Context`, `Business Capability`,
   `Aggregate Roots`, `Database Tables`, `Business Rules`, `External Systems`,
   `API Responsibilities`, `Upstream Services`, `Downstream Services`.

3. For each service in the catalog, use `read_file` to load its Architecture Specification
   from `output/architecture/{service-name}-architecture.md`. If the file is missing, log a
   warning and fall back to the CSV row for that service.

4. Also load the following shared artifacts with `read_file`:
   - `output/graph-evidence-summary.md`
   - `output/bounded-context-map.md`
   - `output/data-ownership-matrix.md`
   - `output/migration-roadmap.md`

5. Log: `📋 Found N microservices to generate`

---

## Step 2 — Retrieve Discovery Documents from MinIO

Use `mcp__minio__list_buckets` to find the latest discovery bucket matching `{appName}-discovery-{YYYYMMDD-HHMMSS}` (appName = workspace folder name). Select the latest by timestamp.

Use `mcp__minio__list_objects` on the selected bucket to detect the object prefix (find where `DISCOVERY_SUMMARY.md` lives — it may be at root or under a `discovery/` prefix).

Download all 7 documents to `temp/discovery/` using `mcp__minio__download_object`:
`DISCOVERY_SUMMARY.md`, `ARCHITECTURE_INVENTORY.md`, `BUSINESS_RULES.md`,
`INTEGRATIONS.md`, `DEPENDENCY_MAP.md`, `DATABASE_INVENTORY.md`, `DATABASE_SUMMARY.md`

**Fallback**: If MinIO is unavailable, read from `context/` directory instead. Log a warning and continue.

Use `read_file` to load the downloaded documents into context.

---

## Step 3 — Enrich with Neo4j Graph Evidence (per service)

For each service, run four Cypher queries via `mcp__neo4j-mcp__read-cypher`.
If Neo4j is unreachable, log a warning, set `graphEvidence = {}`, and continue.

**Query A — Business rules on classes owned by this bounded context**
```cypher
MATCH (bc:BoundedContext {name: $contextName})-[:OWNS_DATA]->(t:Table)
      <-[:WRITES_TO]-(c:Class)
WHERE c.businessRules IS NOT NULL
RETURN c.name AS className, c.description AS classDescription,
       c.businessRules AS businessRules
ORDER BY c.name
```
*Used for*: OpenAPI 400/422 error scenarios; SQL `CHECK` constraint comments; design document business rules section.

**Query B — Method signatures and descriptions**
```cypher
MATCH (bc:BoundedContext {name: $contextName})-[:OWNS_DATA]->(t:Table)
      <-[:WRITES_TO]-(c:Class)-[:CONTAINS]->(m:Method)
WHERE m.signature IS NOT NULL AND m.description IS NOT NULL
RETURN m.name AS methodName, m.signature AS signature,
       m.description AS description, m.businessRule AS businessRule,
       c.name AS className
ORDER BY c.name, m.name
```
*Used for*: OpenAPI `operationId`, `summary`, request/response body shapes.

**Query C — Bounded context recommendation score**
```cypher
MATCH (bc:BoundedContext {name: $contextName})
RETURN bc.name AS name, bc.description AS description,
       bc.recommendation AS recommendation
```
*Used for*: Design doc migration phase ordering; `CONSIDER` adds an Open Question.

**Query D — Cross-context DEPENDS_ON edges**
```cypher
MATCH (c:Class)-[:DEPENDS_ON]->(c2:Class)
WHERE c.package IS NOT NULL AND c2.package IS NOT NULL
  AND (c.package CONTAINS toLower($contextName) OR c2.package CONTAINS toLower($contextName))
RETURN c.name AS fromClass, c.package AS fromPackage,
       c2.name AS toClass, c2.package AS toPackage
ORDER BY c.name
```
*Used for*: Design doc upstream/downstream sections; Saga pattern recommendation.

Log per service: `✅ {name}: N business rules, N methods, recommendation=X, N cross-deps`

---

## Step 4 — Generate Design Artifacts

For each service, generate three files using `write_file`. The Architecture Specification
is the **primary input** for every design decision. Graph evidence and discovery documents
enrich and validate; they do not replace the specification.

Reference the output templates in this skill's directory:
- `openapi-template.yaml` — structure for OpenAPI 3.0.3 specification
- `sql-template.sql` — structure for SQL schema, audit trail, and outbox
- `design-template.md` — structure for the 14-section design document

### 4.1 — OpenAPI Specification → `output/swagger/{service}-openapi.yaml`

Derive from the Architecture Specification (`API Responsibilities`, `Event Contracts`) enriched with graph evidence:
- `operationId` and `summary` — from `graphEvidence.methods[*].signature` and `.description`; fall back to API Responsibilities in architecture specification
- 400/422 error scenarios — from `graphEvidence.classRules[*].businessRules`; fall back to Business Rules in architecture specification
- Add `x-circuit-breaker` extension on services with `recommendation = CONSIDER`
- Apply all Mandatory NFRs (JWT security, `x-timeout-ms`, `x-retry-policy`, `X-Correlation-ID`, health endpoints)
- Mark each endpoint as `Status: Existing` (if found in source code) or `Status: Proposed`
- Include `x-evidence-sources` traceability block

### 4.2 — SQL Schema → `output/sql/{service}-schema.sql`

Derive from the Architecture Specification (`Owned Database Tables`, `Data Ownership Rules`, `Transaction Boundaries`) enriched with graph evidence:
- Owned tables — from Architecture Specification `Owned Database Tables` section; validated against Neo4j `OWNS_DATA` relationships
- `CHECK` constraint comments — from `graphEvidence.classRules[*].businessRules`
- Always include: mandatory audit columns (`created_at`, `created_by`, `updated_at`, `updated_by`), `{schema}_audit_log` table, `{schema}_outbox` table, update-timestamp trigger, audit trigger
- No cross-microservice foreign keys
- Document read-only views for tables owned by other services

### 4.3 — Design Document → `output/design/{service}-design.md`

Derive from all evidence sources with the Architecture Specification as the authoritative baseline. Sections (see `design-template.md`):
1. Service Overview — 2. Business Context (flows + rules) — 3. Architecture (aggregates, domain model, data ownership) — 4. API Design — 5. Data Design — 6. Integration Design (§6.1/§6.2 upstream/downstream from `graphEvidence.crossDeps`) — 7. Event-Driven Architecture (Saga, Outbox) — 8. NFRs — 9. Deployment — 10. Testing Strategy — 11. Migration Strategy (phase order from `graphEvidence.boundedContext.recommendation`) — 12. Open Questions (add one if `recommendation = CONSIDER`) — 13. Risks — 14. Appendix

---

## Step 5 — Generate Design Review Guidelines

For each microservice, write `output/governance/{service-name}-design-review-guidelines.md`
using `write_file`. This document defines the **acceptance criteria** for validating that
any future implementation plan correctly reflects the approved design. Every rule must
state the validation rule, pass condition, failure condition, and design rationale.
This document must be completely technology independent.

```markdown
# {Service Name} — Design Review Guidelines

## Purpose

This document defines the design acceptance criteria derived from the
`{service-name}-architecture.md` specification and the `{service-name}-design.md` design
document. Any implementation plan produced for this service must pass all validation
rules below before development may proceed.

---

## API Design

### Rule API-01: All Required Business APIs Exist
- **Validation rule**: Count the API operations defined in the architecture specification's `API Responsibilities` section.
- **Pass condition**: Every business operation listed in the architecture specification has a corresponding endpoint in the design.
- **Failure condition**: Any required business operation is absent from the API design.
- **Design rationale**: Missing APIs leave business capabilities unreachable by consumers.

### Rule API-02: Endpoint Responsibilities Are Correct
- **Validation rule**: For each endpoint, verify its responsibility matches the architecture specification.
- **Pass condition**: Each endpoint fulfils exactly the business responsibility assigned to it in the architecture specification.
- **Failure condition**: Any endpoint is missing, mislabelled, or assigned a responsibility outside its bounded context.
- **Design rationale**: Incorrect endpoint responsibilities indicate bounded context violations.

### Rule API-03: Request and Response Models Are Complete
- **Validation rule**: Every endpoint has a fully defined request model and at least one success response model.
- **Pass condition**: No endpoint has an empty or partially defined request or response schema.
- **Failure condition**: Any endpoint is missing request body definition, response schema, or both.
- **Design rationale**: Incomplete API contracts prevent consumers from building reliable integrations.

### Rule API-04: Authentication Requirements Are Defined
- **Validation rule**: Every non-health endpoint declares an authentication requirement.
- **Pass condition**: All endpoints (excluding `/health/live` and `/health/ready`) declare authentication.
- **Failure condition**: Any endpoint is missing an authentication declaration.
- **Design rationale**: Unauthenticated endpoints are a security vulnerability.

### Rule API-05: Error Responses Are Defined
- **Validation rule**: Every endpoint declares at least one error response scenario.
- **Pass condition**: All endpoints include error response definitions covering at minimum client error and server error scenarios.
- **Failure condition**: Any endpoint has no error response defined.
- **Design rationale**: Missing error contracts prevent consumers from implementing correct error handling.

---

## Data Design

### Rule DATA-01: Data Ownership Preserved
- **Validation rule**: Compare tables in the data design against `Owned Database Tables` in the architecture specification.
- **Pass condition**: The data design owns exactly the tables listed in the architecture specification — no more, no less.
- **Failure condition**: Any owned table is missing, or any table not listed in the architecture specification is claimed.
- **Design rationale**: Data ownership is the foundation of service independence.

### Rule DATA-02: Audit Tables Included
- **Validation rule**: Verify that a `{schema}_audit_log` table is present in the data design.
- **Pass condition**: An audit log table exists and captures INSERT, UPDATE, and DELETE operations.
- **Failure condition**: The audit log table is absent or incomplete.
- **Design rationale**: Audit trails are a mandatory non-functional requirement for all services.

### Rule DATA-03: Outbox Pattern Included
- **Validation rule**: Verify that a `{schema}_outbox` table is present in the data design.
- **Pass condition**: An outbox table exists to support event-driven consistency.
- **Failure condition**: The outbox table is absent.
- **Design rationale**: The outbox pattern is mandatory for reliable event publishing in distributed systems.

### Rule DATA-04: No Cross-Service Foreign Keys
- **Validation rule**: Inspect all foreign key constraints in the data design.
- **Pass condition**: No foreign key references a table owned by a different service.
- **Failure condition**: Any foreign key points to a table outside this service's ownership boundary.
- **Design rationale**: Cross-service foreign keys create hard database coupling and prevent independent deployment.

### Rule DATA-05: Transaction Boundaries Preserved
- **Validation rule**: Verify that no data operation spans tables owned by multiple bounded contexts in a single transaction.
- **Pass condition**: All transactions are scoped to tables owned by this service.
- **Failure condition**: Any transaction modifies tables belonging to another service's ownership boundary.
- **Design rationale**: Cross-boundary transactions prevent independent deployability and introduce hidden coupling.

---

## Integration Design

### Rule INT-01: Upstream Services Preserved
- **Validation rule**: Compare the integration design's upstream dependencies against the architecture specification's `Upstream Services` section.
- **Pass condition**: All upstream dependencies listed in the architecture specification are present in the integration design.
- **Failure condition**: Any upstream dependency is absent or replaced without justification.
- **Design rationale**: Unresolved upstream dependencies leave the service unable to fulfil its business capability.

### Rule INT-02: Downstream Services Preserved
- **Validation rule**: Compare the integration design's downstream consumers against the architecture specification's `Downstream Services` section.
- **Pass condition**: All downstream service relationships listed in the architecture specification are accounted for.
- **Failure condition**: Any downstream relationship is absent or incorrectly modelled.
- **Design rationale**: Broken downstream contracts disrupt dependent services.

### Rule INT-03: External Integrations Preserved
- **Validation rule**: Verify that all external integrations listed in the architecture specification's `External Integrations` section are modelled in the integration design.
- **Pass condition**: Every external integration is present with its interaction pattern described.
- **Failure condition**: Any external integration is absent.
- **Design rationale**: External integrations are part of the service's bounded context and must be explicitly designed.

### Rule INT-04: Event Publishers Identified
- **Validation rule**: Compare published events in the design against `Published Events` in the architecture specification.
- **Pass condition**: Every event listed as published in the architecture specification has a corresponding publisher in the design.
- **Failure condition**: Any published event is absent from the integration design.
- **Design rationale**: Missing event publishers break the event contract with downstream consumers.

### Rule INT-05: Event Consumers Identified
- **Validation rule**: Compare consumed events in the design against `Consumed Events` in the architecture specification.
- **Pass condition**: Every event listed as consumed in the architecture specification has a corresponding consumer handler in the design.
- **Failure condition**: Any consumed event has no handler defined in the design.
- **Design rationale**: Unhandled consumed events mean domain state changes go unprocessed.

---

## Non-functional Requirements

### Rule NFR-01: Security Requirements
- **Validation rule**: Verify that every non-health endpoint declares authentication and authorisation requirements.
- **Pass condition**: Authentication (token-based) and authorisation scopes are defined for all endpoints.
- **Failure condition**: Any endpoint is missing authentication or authorisation definition.
- **Design rationale**: Security is a mandatory non-functional requirement for all services.

### Rule NFR-02: Observability
- **Validation rule**: Verify that the design includes tracing, metrics collection, and log aggregation requirements.
- **Pass condition**: All three observability concerns are addressed in the design.
- **Failure condition**: Any observability concern is absent.
- **Design rationale**: Unobservable services cannot be diagnosed or operated in production.

### Rule NFR-03: Logging
- **Validation rule**: Verify that the design specifies structured logging with standard fields.
- **Pass condition**: Logging includes at minimum: timestamp, level, service name, correlation ID, and message.
- **Failure condition**: Any required log field is absent, or logging format is unspecified.
- **Design rationale**: Inconsistent logging prevents effective incident investigation.

### Rule NFR-04: Correlation IDs
- **Validation rule**: Verify that every API endpoint accepts and propagates a correlation ID.
- **Pass condition**: `X-Correlation-ID` header is defined on all request and response models.
- **Failure condition**: Any endpoint is missing the correlation ID header definition.
- **Design rationale**: Correlation IDs are required for distributed request tracing.

### Rule NFR-05: Health Endpoints
- **Validation rule**: Verify that the API design includes liveness and readiness health endpoints.
- **Pass condition**: Both `GET /health/live` and `GET /health/ready` are defined.
- **Failure condition**: Either health endpoint is absent.
- **Design rationale**: Health endpoints are mandatory for deployment and orchestration.

### Rule NFR-06: Retry Strategy
- **Validation rule**: Verify that the design defines a retry policy for outbound calls.
- **Pass condition**: Retry policy with attempt count, backoff strategy, and delay bounds is defined.
- **Failure condition**: Retry policy is absent or partially defined.
- **Design rationale**: Missing retry policies lead to brittle integrations under transient failures.

### Rule NFR-07: Timeout Requirements
- **Validation rule**: Verify that the design defines timeout values for all outbound calls and inbound endpoints.
- **Pass condition**: Timeout values are specified for all service interactions.
- **Failure condition**: Any timeout is absent or undefined.
- **Design rationale**: Undefined timeouts cause cascading failures under degraded conditions.

### Rule NFR-08: Audit Requirements
- **Validation rule**: Verify that the design includes an audit log for all state-changing operations.
- **Pass condition**: An audit log captures INSERT, UPDATE, and DELETE events for all owned tables.
- **Failure condition**: Any state-changing operation lacks audit coverage.
- **Design rationale**: Audit trails are required for compliance and operational investigation.

---

## Design Constraints

### Rule DC-01: No Shared Database Ownership
- **Validation rule**: Verify that no table in the data design is shared with or owned by another service.
- **Pass condition**: Every table in the data design is exclusively owned by this service.
- **Failure condition**: Any table is shared with or claimed by another service.
- **Design rationale**: Shared database ownership is the root cause of distributed monoliths.

### Rule DC-02: No Cross-Context Transactions
- **Validation rule**: Verify that no operation in the design spans multiple bounded contexts in a single transaction.
- **Pass condition**: All transactions are scoped entirely within this service's data ownership boundary.
- **Failure condition**: Any operation atomically modifies data across bounded context boundaries.
- **Design rationale**: Cross-context transactions prevent independent deployability.

### Rule DC-03: No Missing NFRs
- **Validation rule**: Verify that all non-functional requirements listed in the architecture specification are addressed in the design.
- **Pass condition**: Every NFR from the architecture specification has a corresponding design decision.
- **Failure condition**: Any NFR is absent or unaddressed.
- **Design rationale**: Unaddressed NFRs result in operationally deficient services.

### Rule DC-04: No Missing APIs
- **Validation rule**: Verify that all API responsibilities from the architecture specification are present in the design.
- **Pass condition**: Every API responsibility is represented as at least one designed operation.
- **Failure condition**: Any API responsibility has no corresponding operation in the design.
- **Design rationale**: Missing APIs create gaps in the service's business capability coverage.

### Rule DC-05: No Missing Events
- **Validation rule**: Verify that all published and consumed events from the architecture specification are present in the design.
- **Pass condition**: Every event contract is represented in the integration design.
- **Failure condition**: Any event is absent from the design.
- **Design rationale**: Missing events break the asynchronous integration contracts of the service.

### Rule DC-06: No Missing Integration Points
- **Validation rule**: Verify that every integration point (upstream, downstream, external) from the architecture specification is present in the design.
- **Pass condition**: All integration points are explicitly modelled with their interaction patterns.
- **Failure condition**: Any integration point is absent or implicit.
- **Design rationale**: Implicit integrations become undocumented hidden coupling.

### Rule DC-07: No Inconsistent Ownership Boundaries
- **Validation rule**: Verify that the data ownership boundaries in the design are consistent with the architecture specification and data ownership matrix.
- **Pass condition**: Data ownership in the design matches the architecture specification and `output/data-ownership-matrix.md` exactly.
- **Failure condition**: Any ownership boundary in the design differs from the architecture specification without documented justification.
- **Design rationale**: Inconsistent ownership boundaries indicate an unresolved architectural conflict.
```

---

## Step 6 — Validate Outputs

For each service, confirm using `glob` or `read_file`:
- `output/swagger/{service}-openapi.yaml` exists and contains `openapi: 3.0.3`
- `output/sql/{service}-schema.sql` exists and contains the audit log table
- `output/design/{service}-design.md` exists
- `output/governance/{service}-design-review-guidelines.md` exists

Quality checklist — every artifact must pass:
- [ ] All mandatory NFRs included
- [ ] Every endpoint has an evidence source annotation
- [ ] Every owned table has a clear ownership comment
- [ ] No cross-microservice foreign keys
- [ ] APIs marked Existing or Proposed
- [ ] Open questions documented where ownership is unclear
- [ ] Design review guidelines generated for every service

---

## Step 7 — Upload Design Documents to MinIO

1. Determine `appName` from workspace folder name
2. Use `mcp__minio__list_buckets` — if `{appName}` bucket does not exist, create it with `mcp__minio__create_bucket`
3. Upload each design document using `mcp__minio__upload_object` with prefix `designer/`:

| Local file | MinIO object |
|-----------|-------------|
| `output/swagger/{service}-openapi.yaml` | `designer/swagger/{service}-openapi.yaml` |
| `output/sql/{service}-schema.sql` | `designer/sql/{service}-schema.sql` |
| `output/design/{service}-design.md` | `designer/design/{service}-design.md` |
| `output/governance/{service}-design-review-guidelines.md` | `designer/governance/{service}-design-review-guidelines.md` |

4. Report: `✅ Uploaded N design documents to MinIO bucket "{appName}" under prefix "designer/"`. For each file: `✅ {objectName}` or `❌ {objectName} — {error}`.

---

## Guiding Principle

The Designer translates architecture into technology-independent design.

It does not generate implementation plans or implementation tasks.

Its outputs become the authoritative design inputs that the Developer skill will use to
generate implementation artifacts and ultimately implement the service.

---

## Rules

1. **Architecture Specification is the primary design source** — OpenAPI, SQL, and design docs must be derived from `{service-name}-architecture.md`
2. **CSV provides service inventory** — use it to enumerate services; defer to the architecture specification for all design decisions
3. **Neo4j graph evidence enriches artifacts** — it is not a hard dependency; skip gracefully if unavailable
4. **Discovery documents provide business context only** — not direct design input
5. **No generic services** — all services must be specific and well-defined
6. **No cross-microservice foreign keys** — services own their data
7. **Capture unclear ownership as Open Questions** — document ambiguities
8. **Include evidence traceability** in all generated artifacts
9. **Mark APIs as Existing or Proposed** — distinguish current from future state
10. **Technology agnostic outputs** — design artifacts must not reference programming languages, frameworks, ORMs, or deployment technologies
