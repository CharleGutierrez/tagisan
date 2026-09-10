# Discovery Document Templates
# Used by: skills-app-discovery-enhanced/SKILL.md — Step 9
#
# Each section marked <!-- REQUIRED --> must be present.
# Sections marked <!-- IF AVAILABLE --> are included only when data exists.
# Downstream skills (Architect, Designer) parse these documents by section heading —
# heading names must be preserved exactly.

---

## Template: DISCOVERY_SUMMARY.md

```markdown
# Application Discovery Summary
- **Application Name**: {name}
- **Language / Framework**: {language} / {framework}
- **Total Lines of Code**: {totalLOC}
- **Analysis Date**: {date}

## Architecture Style  <!-- REQUIRED -->
{architecturalStyles — e.g. "Layered (Controller → Service → Repository)"}

## Request Flow  <!-- REQUIRED -->
{requestFlow — ordered list from entry point to data store}

## Technology Stack  <!-- REQUIRED -->
| Technology | Category | Version |
|-----------|----------|---------|
| {name} | {category} | {version} |

## Key Architecture Risks  <!-- REQUIRED -->
{architectureRisks — bullet list; "None identified" if empty}

## Key Design Risks  <!-- REQUIRED -->
{designRisks — bullet list; "None identified" if empty}

## Discovery Completeness
- Packages analysed: {N}
- Classes analysed: {N}
- Methods analysed: {N}
- Tables discovered: {N}
- Integrations found: {N}
```

---

## Template: ARCHITECTURE_INVENTORY.md

```markdown
# Architecture Inventory

## Architectural Layers  <!-- REQUIRED -->
| Layer | Packages | Responsibility |
|-------|----------|---------------|
| API / Controller | {packages} | Handles HTTP requests |
| Service / Application | {packages} | Business logic orchestration |
| Domain | {packages} | Core domain model and rules |
| Repository / Infrastructure | {packages} | Data access and external calls |

## Design Patterns Detected  <!-- REQUIRED -->
| Pattern | Classes | Description |
|---------|---------|-------------|
| {pattern} | {classes} | {description} |

## Business Capabilities  <!-- REQUIRED -->
| Capability | Description | Key Classes |
|-----------|-------------|------------|
| {name} | {description} | {classes} |

## Entry Points  <!-- REQUIRED -->
| Class | Type | Endpoints / Methods |
|-------|------|-------------------|
| {class} | REST Controller / MVC Controller | {endpoints} |

## Component Interaction Diagram  <!-- IF AVAILABLE -->
```
{ASCII or textual diagram showing major component interactions}
```
```

---

## Template: DATABASE_INVENTORY.md

```markdown
# Database Inventory

## Tables  <!-- REQUIRED -->
| Table | Purpose | Row Estimate |
|-------|---------|-------------|
| {table} | {purpose} | {estimate or "unknown"} |

## Columns  <!-- REQUIRED — one sub-section per table -->

### {TABLE_NAME}
| Column | Data Type | Nullable | Constraints |
|--------|-----------|----------|-------------|
| {column} | {type} | {YES/NO} | {PK/FK/UQ/CHECK} |

## Stored Procedures  <!-- IF AVAILABLE -->
| Name | Description | Accesses Tables |
|------|-------------|----------------|
| {name} | {description} | {tables} |
```

---

## Template: DATABASE_SUMMARY.md

```markdown
# Database Summary

## Foreign Key Relationships  <!-- REQUIRED -->
| From Table | Column | References Table | Column | On Delete |
|-----------|--------|-----------------|--------|-----------|
| {fromTable} | {column} | {toTable} | {references} | {onDelete} |

## Shared Table Access (Coupling Risk)  <!-- REQUIRED -->
Tables accessed by more than one package — these are coupling hotspots.

| Table | Accessing Packages | Access Types |
|-------|--------------------|--------------|
| {table} | {packages} | READ / WRITE |

## Data Access Patterns  <!-- REQUIRED -->
| Class | Table | Access Type | Operations | Frequency |
|-------|-------|-------------|------------|-----------|
| {class} | {table} | READ/WRITE/READ_WRITE | SELECT/INSERT/UPDATE/DELETE | low/medium/high |

## Data Ownership Assessment  <!-- REQUIRED -->
| Table | Likely Owner Package | Confidence | Notes |
|-------|---------------------|------------|-------|
| {table} | {package} | HIGH/MEDIUM/LOW | {notes} |
```

---

## Template: BUSINESS_RULES.md

```markdown
# Business Rules

## Summary
Total rules extracted: {N}

## Rules by Class  <!-- REQUIRED — one sub-section per class that has rules -->

### {ClassName} ({fullyQualifiedName})

| Rule ID | Description | Method | Enforcement |
|---------|-------------|--------|-------------|
| BR-{SVC}-{NNN} | {description} | {methodName} | throw / validation / guard clause |

**Example (from code)**:
```
{brief code snippet or paraphrase of the logic}
```

## Validation Rules  <!-- IF AVAILABLE -->
| Rule ID | Field | Constraint | Error Message |
|---------|-------|------------|---------------|
| VAL-{NNN} | {field} | {constraint} | {message} |

## Business Invariants  <!-- IF AVAILABLE -->
Conditions that must always hold true regardless of operation:
- {invariant description}
```

---

## Template: INTEGRATIONS.md

```markdown
# External Integrations

## Summary
Total integrations found: {N}

## Integration Inventory  <!-- REQUIRED -->
| From | To | Type | Protocol | Purpose | Frequency |
|------|----|------|----------|---------|-----------|
| {fromContext} | {toSystem} | sync/async | REST/gRPC/MQ/DB | {purpose} | low/medium/high |

## Integration Details  <!-- REQUIRED — one sub-section per integration -->

### {From} → {To}
- **Type**: {sync | async}
- **Protocol**: {REST | gRPC | Kafka | RabbitMQ | JMS | JDBC | other}
- **Authentication**: {JWT / API Key / mTLS / none / unknown}
- **Key Classes**: {classes that initiate or handle this integration}
- **Evidence**: {grep pattern that identified this integration}

## External Systems Map  <!-- IF AVAILABLE -->
| External System | Role | Access Pattern |
|----------------|------|---------------|
| {system} | {database/cache/queue/api} | {read/write/pubsub} |
```

---

## Template: DEPENDENCY_MAP.md

```markdown
# Dependency Map

## Package-Level Dependencies  <!-- REQUIRED -->
| From Package | To Package | Dependency Count | Type |
|-------------|-----------|-----------------|------|
| {from} | {to} | {count} | uses/extends/implements |

## Coupling Hotspots  <!-- REQUIRED -->
Classes or packages with the highest number of inbound dependencies (most depended-upon):

| Class / Package | Inbound Dependencies | Risk |
|----------------|---------------------|------|
| {class} | {count} | HIGH/MEDIUM/LOW |

## Highly Coupled Classes (God Classes / Anti-patterns)  <!-- REQUIRED -->
Classes with both high fan-in AND high fan-out:

| Class | Fan-In | Fan-Out | Assessment |
|-------|--------|---------|------------|
| {class} | {N} | {N} | Potential god class / refactor candidate |

## Dependency Cycles  <!-- IF AVAILABLE -->
Circular dependencies that will complicate microservice extraction:

- {PackageA} ↔ {PackageB}: {description}

## External Library Dependencies  <!-- REQUIRED -->
| Library / Framework | Version | Used By (packages) |
|--------------------|---------|-------------------|
| {library} | {version} | {packages} |
```
