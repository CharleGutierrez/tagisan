---
name: skills-microservices-developer
description: >-
  Use when the user wants to generate production-ready microservice implementation
  code from approved architecture and design artifacts — retrieves architecture
  specifications, OpenAPI, SQL schemas, and design documents from MinIO, generates
  and validates GitHub Spec Kit artifacts (spec.md, plan.md, tasks.md), invokes
  GitHub Spec Kit implementation, and enforces enterprise coding standards and
  governance throughout. Retrieves enterprise coding standards from MinIO and enforces
  them throughout code generation.
metadata:
  disable-model-invocation: false
---

# Microservices Developer

Implementation Orchestrator that translates approved architecture and design artifacts into
production-ready microservice implementations via GitHub Spec Kit.

This skill retrieves approved artifacts from MinIO, assembles enterprise implementation context,
invokes GitHub Spec Kit, evaluates generated outputs, and enforces enterprise governance.

**Prerequisite chain**: `skills-app-discovery-enhanced` → `skills-microservices-architect` → `skills-microservices-designer` → **this skill**

## When to Use

- Generating microservice implementations from approved architecture and design artifacts
- Applying enterprise coding standards dynamically from MinIO to generated code
- Orchestrating GitHub Spec Kit artifact generation and implementation
- Preparing and validating inputs before Spec Kit invocation
- Performing governance and traceability validation on Spec Kit outputs

## When NOT to Use

- Creating architecture specifications (use `skills-microservices-architect` first)
- Creating design artifacts (use `skills-microservices-designer` first)
- Running application discovery (use `skills-app-discovery-enhanced`)

## Implementation Precedence (conflict resolution)

1. Architecture Specification — authoritative for **WHAT** to build (business capability, bounded context, aggregate roots, rules)
2. Design Document — authoritative for **HOW** to design (API design, data model, integrations, NFRs)
3. `spec.md` — generated from Architecture Specification; authoritative for Spec Kit
4. `plan.md` — generated from Design Document; authoritative for implementation approach
5. `tasks.md` — authoritative for **WHEN** (task order)
6. Enterprise standards from MinIO `coding-standards` bucket — authoritative for implementation style
7. `openapi.yaml` — supporting artifact (must align with architecture specification)
8. `schema.sql` — supporting artifact (must align with design document)

---

## Step 0 — Confirm Inputs

Use `ask_followup_question` to confirm:
- Service name (e.g., `inventory-service`)
- Technology Profile (determines stack, framework, build system, deployment platform)
- Deployment target (`kubernetes` | `openshift` | `docker`)

---

## Step 0.5 — Load Enterprise Standards from MinIO

Use `mcp__minio__list_buckets` to find the `coding-standards` bucket (fixed name).

If found:
1. Use `mcp__minio__list_objects` on `coding-standards` (recursive: true) to enumerate all documents
2. Use `mcp__minio__download_object` for each document → `temp/standards/{filename}`
3. Use `read_file` to load the downloaded documents

Standards to discover and load:
- Coding standards for the chosen language (naming conventions, patterns)
- Package/module structure and layering conventions (API → Application → Domain → Infrastructure)
- Logging standards (structured logging, log levels, correlation IDs, sensitive data masking)
- Error handling standards (exception hierarchies, error codes)
- Security standards (auth, authorization, encryption, input validation)
- Testing standards (coverage targets ≥ 80%, test patterns, mock usage)
- Observability standards (metrics naming, tracing span naming, health check format)
- Deployment standards (resource limits, scaling policies, secret management)

**Fallback**: If `coding-standards` bucket is not found, log `⚠️ Using default standards` and continue. Apply defaults from `stack-dependencies.md` and `code-patterns.md`.

---

## Step 2 — Prepare the Implementation Workspace

> **GitHub Spec Kit is the sole owner of project and source code generation.**
> This step performs orchestration preparation only. No application source folders,
> build files, deployment manifests, tests, Dockerfiles, Kubernetes manifests, or
> project scaffolding are created by this skill.

Perform the following pre-flight checks before invoking GitHub Spec Kit:

1. **Repository initialisation** — Verify that the repository is initialised for GitHub Spec Kit (`.specify/` directory or equivalent Spec Kit marker exists). If not present, report the missing initialisation and stop execution.

2. **Branch verification** — Verify that the correct repository and branch are checked out for the target service. Report the active branch and confirm it is appropriate for generation.

3. **Existence check** — Determine whether the target service already exists in the repository:
   - If the service does not exist → proceed as a **new service generation**.
   - If the service already exists → determine whether this is an **update** to an existing service, confirm with the user before continuing.

4. **Output directory check** — Verify that the required output directories for Spec Kit generated artifacts exist (e.g., `output/` tree expected by this skill's publishing steps). Create only these temporary working folders if they are absent:
   ```
   temp/standards/      ← enterprise standards downloads (Step 0.5)
   temp/artifacts/      ← retrieved MinIO artifact downloads (Step 3.1)
   ```
   Do **not** create service source directories, build directories, or any application scaffolding.

5. **Prerequisite artifact availability** — Validate that all required architecture and design artifacts are resolvable (MinIO or local fallback) before Spec Kit is invoked. This is a pre-check only; full artifact retrieval happens in Phase 3.
   - Architecture Specification resolvable?
   - OpenAPI Specification resolvable?
   - SQL Schema resolvable?
   - Design Document resolvable?
   - Enterprise Coding Standards resolvable?

   If any prerequisite artifact is unresolvable at this stage, report every missing artifact and stop execution. Do not proceed to Phase 3.

**Ownership boundary — this skill does NOT create:**
- Application source folders or package hierarchies
- Build configuration files (`pom.xml`, `build.gradle`, `package.json`, `requirements.txt`, etc.)
- Dependency manifests
- Dockerfile or container build files
- `docker-compose.yml` or any container orchestration files
- Kubernetes or OpenShift manifests
- Any test scaffolding or test source directories
- Any deployment or infrastructure files
- Any project skeleton, module, or directory beyond the `temp/` working folders listed above

GitHub Spec Kit generates all of the above when `speckit-implement` is invoked in Phase 4.

---

## Phase 3 — Load, Generate and Validate GitHub Spec Kit Artifacts

The Developer is the **only** skill responsible for interacting with GitHub Spec Kit.

The Developer does **not** generate architecture, design, or implementation artifacts.

It consumes approved architecture and design artifacts, assembles the implementation context, invokes GitHub Spec Kit, and evaluates the generated outputs.

---

### Step 3.1 — Retrieve Approved Inputs

The MinIO MCP Server is the authoritative source for all approved artifacts.

Always retrieve artifacts from MinIO first.

Only if an artifact is not found in MinIO should the Developer attempt to load the corresponding file from the local workspace.

The lookup order is always:

```text
MinIO MCP Server
        ↓
Local output/ directory
        ↓
Fail
```

Never prefer the local copy over MinIO.

#### Architecture Inputs

Retrieve from MinIO (prefix `architect/`) then fall back to local:

```
output/architecture/{service-name}-architecture.md
output/governance/{service-name}-spec-review-guidelines.md
```

#### Design Inputs

Retrieve from MinIO (prefix `designer/`) then fall back to local:

```
output/swagger/{service-name}-openapi.yaml
output/sql/{service-name}-schema.sql
output/design/{service-name}-design.md
output/governance/{service-name}-design-review-guidelines.md
```

#### Enterprise Inputs

Retrieve from the `coding-standards` bucket (already loaded in Step 0.5):

- Coding Standards
- Security Standards
- Logging Standards
- Testing Standards
- Observability Standards
- Deployment Standards
- Architecture Standards
- Technology Profiles

If an artifact is found in MinIO: download it and use the downloaded copy.

If not found: attempt to load it from the local workspace.

If still not found: stop execution, report every missing artifact, and do not invoke GitHub Spec Kit.

---

### Step 3.2 — Validate Retrieved Inputs

This is the single authoritative validation gate before invoking GitHub Spec Kit.

Validate all retrieved inputs only after Step 3.1 has completed.

Before invoking GitHub Spec Kit verify that every required artifact exists:

- [ ] Architecture Specification exists
- [ ] OpenAPI Specification exists
- [ ] SQL Schema exists
- [ ] Design Document exists
- [ ] Architecture Spec Review Guidelines exist
- [ ] Design Review Guidelines exist
- [ ] Technology Profile exists
- [ ] Enterprise Coding Standards loaded successfully

Verify consistency:

- Architecture Specification aligns with OpenAPI
- Architecture Specification aligns with SQL Schema
- Design Document aligns with Architecture Specification
- OpenAPI aligns with SQL Schema
- Review Guidelines exist for the service

If any verification fails: report every validation error, stop execution, and do not invoke GitHub Spec Kit.

---

## Implementation Context

Before invoking GitHub Spec Kit, the Developer Agent assembles an implementation context consisting of:

- User implementation request
- Architecture Specification
- Design Document
- OpenAPI Specification
- SQL Schema
- Technology Profile
- Coding Standards
- Design Standards
- Organization Constitution
- Any additional approved implementation constraints

This implementation context becomes the input to GitHub Spec Kit.

**Developer Agent responsibilities:**

- Retrieve approved artifacts
- Validate artifact completeness
- Assemble Implementation Context
- Invoke GitHub Spec-Kit commands
- Evaluate generated outputs
- Improve the Implementation Context if required
- Invoke GitHub Spec-Kit again when necessary
- Publish validated outputs

**GitHub Spec-Kit responsibilities:**

- `spec.md`
- `plan.md`
- `tasks.md`
- implementation
- project structure
- source code
- tests
- documentation

The Developer Agent must never directly generate, edit, or regenerate these artifacts.

---

### Step 3.3 — Assemble Implementation Context

Construct the complete implementation context using:

- User implementation request
- Architecture Specification
- OpenAPI Specification
- SQL Schema
- Design Document
- Enterprise Coding Standards
- Design Standards
- Organization Constitution
- Technology Profile
- Additional approved implementation constraints

The Developer must never invent:

- Business Requirements
- Business Rules
- Service Responsibilities
- Architecture
- Design Decisions

These are authoritative inputs and must remain traceable to the generated artifacts.

---

### Step 3.4 — Invoke GitHub Spec-Kit speckit-specify

Invoke:

```
speckit-specify
```

using the assembled implementation context.

---

### Step 3.5 — Evaluate spec.md

Evaluate the generated `spec.md` against the Architecture Specification and Architecture Spec Review Guidelines.

Verify that:

- [ ] Business capability preserved
- [ ] Bounded context preserved
- [ ] Aggregate roots preserved
- [ ] Business rules preserved
- [ ] Data ownership preserved
- [ ] APIs preserved
- [ ] Events preserved
- [ ] External integrations preserved
- [ ] NFRs preserved
- [ ] Architectural constraints preserved

If validation fails: report every deviation, improve the Implementation Context, and invoke `speckit-specify` again.

Do not edit `spec.md` directly. Do not continue until validation succeeds.

---

### Step 3.6 — Invoke GitHub Spec-Kit speckit-plan

Invoke:

```
speckit-plan
```

using the selected Technology Profile.

---

### Step 3.7 — Evaluate plan.md

Evaluate `plan.md` against the Design Document, OpenAPI Specification, SQL Schema, Design Review Guidelines, and Technology Profile.

Verify that:

- [ ] APIs correctly represented
- [ ] SQL model preserved
- [ ] Integration design preserved
- [ ] NFRs included
- [ ] Security strategy included
- [ ] Observability strategy included
- [ ] Deployment strategy included
- [ ] Testing strategy included
- [ ] Selected Technology Profile correctly applied

If validation fails: report every deviation, improve the Implementation Context, and invoke `speckit-plan` again.

Do not edit `plan.md` directly. Do not continue until validation succeeds.

---

### Step 3.8 — Invoke GitHub Spec-Kit speckit-tasks

Invoke:

```
speckit-tasks
```

---

### Step 3.9 — Evaluate tasks.md

Verify:

- [ ] Every Functional Requirement has implementation tasks
- [ ] Every Business Rule has implementation tasks
- [ ] Every API has implementation tasks
- [ ] Every Integration has implementation tasks
- [ ] Every NFR has implementation tasks
- [ ] Security tasks exist
- [ ] Observability tasks exist
- [ ] Testing tasks exist
- [ ] Deployment tasks exist
- [ ] Tasks follow logical dependency order
- [ ] No duplicate tasks
- [ ] No orphan tasks

If validation fails: report every deviation, improve the Implementation Context, and invoke `speckit-tasks` again.

Do not edit `tasks.md` directly. Do not continue until validation succeeds.

---

## Phase 4 — Invoke GitHub Spec-Kit speckit-implement

After successful evaluation of all Spec Kit artifacts, invoke:

```
speckit-implement
```

using the selected Technology Profile.

The Technology Profile determines the implementation stack. It may specify:

- Programming Language
- Framework
- Build System
- Dependency Management
- Persistence Technology
- ORM / Data Access Framework
- Messaging Technology
- API Framework
- Security Framework
- Testing Framework
- Observability Framework
- Deployment Platform

The implementation may target any supported technology including (but not limited to):

| Runtime | Framework |
|---------|-----------|
| Java | Spring Boot |
| Java | Quarkus |
| Java | Micronaut |
| C# | ASP.NET Core |
| Go | Gin |
| Go | Fiber |
| Python | FastAPI |
| Python | Flask |
| Node.js | Express |
| Node.js | NestJS |

The Developer must never contain technology-specific generation logic.

GitHub Spec Kit is responsible for generating all implementation artifacts appropriate for the selected Technology Profile.

The Developer must **not** manually generate:

- Project structure
- Source code
- Domain layer
- Application layer
- Persistence layer
- API layer
- Infrastructure layer
- Security implementation
- Messaging implementation
- Caching implementation
- Observability implementation
- Unit tests
- Integration tests
- Contract tests
- Build configuration
- Dockerfiles
- Kubernetes manifests
- Documentation

After GitHub Spec Kit completes implementation, the Developer must:

1. Validate the generated implementation against Enterprise Coding Standards.
2. Validate traceability to the user implementation request, Architecture Specification, Design Document, OpenAPI Specification, SQL Schema, Technology Profile, Coding Standards, Constitution, and Spec Kit artifacts.
3. Validate compliance with enterprise governance.
4. Validate completeness of generated artifacts.
5. Continue with the compliance validation and publishing phases below.

---

## Phase 5 — Documentation (follows tasks.md order)

GitHub Spec Kit generates the primary service documentation as part of `speckit-implement`.

After Spec Kit completes, verify that `output/code/{service}/README.md` exists and contains:
- Service overview (traceable to architecture specification)
- Setup and local development instructions (traceable to plan.md)
- API endpoint summary (traceable to openapi.yaml)
- Configuration reference (all env vars / ConfigMap keys)
- Deployment guide (traceable to plan.md)
- Troubleshooting guide

If the README is absent or incomplete after Spec Kit execution, report the gap and request regeneration via Spec Kit. Do **not** write the README manually.

---

## Phase 6 — Validate Implementation

Before completing, run a final compliance check:

**Spec Kit coverage:**
- [ ] All tasks from tasks.md implemented
- [ ] All functional requirements from spec.md implemented
- [ ] Architecture conforms to plan.md
- [ ] All business rules from spec.md enforced

**Enterprise standards compliance:**
- [ ] Naming conventions, layering, dependency rules followed
- [ ] Logging, security, testing, observability, deployment standards met
- [ ] No hardcoded credentials

**Artifact completeness:**
- [ ] All OpenAPI endpoints implemented
- [ ] All SQL tables have entity / repository classes
- [ ] Unit tests ≥ 80% coverage
- [ ] Dockerfile, Kubernetes manifests generated
- [ ] README.md complete

**Traceability:**
- [ ] All generated artifacts traceable to the user implementation request
- [ ] All generated artifacts traceable to Architecture Specification
- [ ] All generated artifacts traceable to Design Document
- [ ] All generated artifacts traceable to OpenAPI Specification
- [ ] All generated artifacts traceable to SQL Schema
- [ ] All generated artifacts traceable to Technology Profile
- [ ] All generated artifacts traceable to Coding Standards
- [ ] All generated artifacts traceable to Constitution
- [ ] All generated artifacts traceable to Spec Kit artifacts

If any check fails, improve the Implementation Context, invoke `speckit-implement` again, and revalidate until compliant.

---

## Phase 7 — Publish README to MinIO

After all artifacts are validated:

1. Determine `appName` from workspace folder name.
2. Upload `output/code/{service}/README.md` to the `{appName}` application bucket using `mcp__minio__upload_object`:
   - `bucketName` = `{appName}`
   - `objectName` = `output/{service-name}/README.md`
   - `contentType` = `text/markdown`

3. Verify with `mcp__minio__get_object_info` — confirm the returned size matches the local file.

4. Report: `✅ README uploaded to {appName}/output/{service-name}/README.md`

**Failure**: If upload fails, log `⚠️ MinIO upload failed — file available locally at output/code/{service}/README.md` and continue. Do not fail overall generation.

---

## Guiding Principle

The Developer Agent is an orchestration agent. It prepares enterprise implementation context, invokes GitHub Spec-Kit, evaluates the generated outputs, and determines whether another iteration is required. GitHub Spec-Kit is the authoritative owner of `spec.md`, `plan.md`, `tasks.md`, implementation artifacts, and project generation.

Its responsibilities are to:

- Retrieve approved artifacts from MinIO.
- Fall back to the local workspace only when necessary.
- Load Enterprise Coding Standards.
- Load the selected Technology Profile.
- Prepare the implementation workspace (pre-flight checks only).
- Assemble the Implementation Context.
- Invoke GitHub Spec-Kit commands.
- Evaluate generated Spec-Kit artifacts against approved architecture and design.
- Improve the Implementation Context when validation requires another iteration.
- Validate enterprise compliance of Spec Kit outputs.
- Publish validated artifacts.

---

**GitHub Spec-Kit owns generation of:**

- `spec.md`
- `plan.md`
- `tasks.md`
- implementation artifacts
- Project structure
- Source code
- Build configuration
- Dependency management
- API implementation
- Persistence layer
- Infrastructure layer
- Tests
- Deployment manifests
- Documentation

**The Developer skill only:**

- Prepares inputs
- Assembles implementation context
- Invokes Spec-Kit
- Evaluates outputs
- Performs governance checks

---

## Rules

1. **Architecture Specification is authoritative** — defines WHAT to build; spec.md must faithfully reflect it
2. **Design Document is authoritative** — defines HOW to design; plan.md must faithfully reflect it
3. **MinIO is the authoritative source** — always retrieve approved artifacts from MinIO before the local workspace
4. **Task order matters** — implement in the sequence defined by tasks.md
5. **Every artifact must be traceable** — @specification, @requirement, @task, @governance in all doc comments
6. **Enterprise standards override defaults** — loaded from MinIO `coding-standards` bucket
7. **No hardcoded credentials** — secrets always via environment variables or Kubernetes Secrets
8. **No cross-microservice foreign keys** — services own their data
9. **Regenerate until compliant** — if validation fails, improve the Implementation Context, invoke GitHub Spec-Kit again, and revalidate
10. **Never invent requirements** — business rules, architecture, and design decisions come from approved artifacts only
