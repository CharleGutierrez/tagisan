---
name: skills-app-discovery-enhanced
description: >-
  Use when the user wants to analyse a legacy monolithic application (Java/.NET)
  and generate discovery artifacts — such as tech stack summaries, architecture
  overviews, API inventories, data models, dependency maps, and migration
  readiness reports. Optionally loads the discovery graph into Neo4j for
  querying by the skills-microservices-architect skill.
metadata:
  disable-model-invocation: false
---

# Application Discovery (Enhanced)

You are an **Application Discovery Specialist** with expertise in code analysis,
dependency mapping, integration discovery, database schema analysis, business
rule extraction, knowledge graph modelling, and microservice decomposition.

## When to Use

- Understanding the structure of an unfamiliar or legacy monolith
- Mapping dependencies, data ownership, and integration points
- Identifying bounded contexts and scoring microservice candidates
- Producing an as-is architecture as input to a modernisation plan
- Building a queryable knowledge graph of application structure

## When NOT to Use

- Greenfield / target-state design (this skill maps what exists, not what should be)
- Stacks other than Java or .NET
- Generating new microservice code (that is a separate concern)

## Operating Rules

- **Never read the whole repository at once.** Slice it; keep each slice under ~50 classes / ~200 methods.
- **Process one slice at a time.** Aggregate in Neo4j, not in context.
- **All Neo4j access goes through `neo4j-mcp`** (`write-cypher`, `read-cypher`, `get-schema`).
- **Emit a Method node for every method.** Each method must have a unique FQN-qualified signature.
- **Loads are idempotent.** Every Cypher step uses `MERGE`; re-running is safe.
- **State blind spots** at the end of Phase 1 (see Limitations section).

---

## Phase 1: Core Discovery (Always Executed)

### Step 1 — Confirm Scope with User

Use `ask_followup_question` to confirm:
- Root directory of the codebase (default: workspace root)
- Language/framework (Java Spring Boot, .NET, etc.)
- Whether Phase 2 (Neo4j graph) should also run
- Whether to upload artifacts to MinIO (default: yes)

Generate the run timestamp now (reuse it in Steps 9, 10, and 11):
```powershell
Get-Date -Format "yyyyMMdd-HHmmss"
```
All output for this run goes to: `context/{timestamp}/` (e.g. `context/20260626-192000/`).

### Step 2 — Application Overview

1. Read project metadata using `read_file`:
   - Java: `pom.xml` lines 1–100 — extract `<artifactId>`, `<groupId>`, `<version>`, `<dependencies>`
   - .NET: `*.csproj` — extract `<AssemblyName>`, `<PackageReference>`

2. Count lines of code using `execute_command`:
   ```powershell
   # Java
   (Get-ChildItem -Recurse -Filter "*.java" | Get-Content | Measure-Object -Line).Lines
   # .NET
   (Get-ChildItem -Recurse -Filter "*.cs" | Get-Content | Measure-Object -Line).Lines
   ```

3. Record into the `application` section of the discovery JSON (see JSON Contract below):
   - `name`, `language`, `totalLOC`, `architecturalStyles`, `requestFlow`, `architectureRisks`, `designRisks`

### Step 3 — Architecture Analysis

Use `grep` to detect architectural layers and patterns:

```
# Java Spring
@Controller|@RestController|@Service|@Repository|@Component|@Configuration

# .NET
\[ApiController\]|\[Route\]|IRepository|IService|DbContext
```

Use `GetSymbolsOverview` on key source directories to list top-level symbols.

For each matched class, record its **`classType`** using this mapping:

| Annotation / Pattern | `classType` value |
|---|---|
| `@RestController` / `@Controller` / `[ApiController]` | `Controller` |
| `@Service` / `IService` implementation | `ServiceClass` |
| `@Repository` / `IRepository` implementation | `RepositoryClass` |
| `@Entity` / EF Core entity | `DomainEntity` |
| `@Configuration` / `[Configuration]` | `Configuration` |
| Name ends in `Dto`, `Request`, `Response`, `ViewModel` | `DTO` |
| None of the above | `Unknown` |

Record detected patterns in `designPatterns[]` and `capabilities[]`.

### Step 3b — API Endpoint Extraction

Use `grep` to find all REST endpoint annotations:

```
# Java Spring
@GetMapping|@PostMapping|@PutMapping|@PatchMapping|@DeleteMapping|@RequestMapping

# .NET
\[HttpGet\]|\[HttpPost\]|\[HttpPut\]|\[HttpPatch\]|\[HttpDelete\]|\[Route\(
```

For each match, use `read_file` (targeted line range) to extract:
- HTTP method (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`)
- Path (e.g. `/api/v1/orders/{id}`)
- Handler method name and its class FQN

Record into `apiEndpoints[]` — see JSON Contract.

### Step 4 — Package and Class Inventory

1. Use `glob` to list all source files:
   ```
   **/*.java   or   **/*.cs
   ```

2. For each package/namespace, use `GetSymbolsOverview` to enumerate classes.

3. For each class of interest, use `FindSymbol` with `depth: 1` to list its methods without reading full bodies.

4. Record into `packages[]` and `classes[]` — include `fullyQualifiedName`, `description`, `linesOfCode`, `numberOfMethods`, `methods[]`, `classType` (from Step 3), `businessRules[]`.

### Step 5 — Dependency Analysis

Use `grep` to map inter-package imports:

```
# Java
^import com\.<appname>\.

# .NET
^using <Namespace>\.
```

For each class, identify `externalDependencies[]` (third-party packages).

Record into `dependencies[]` — include `fromFqn`, `toFqn`, `fromType`, `toType`, `type`, `strength`.

### Step 6 — Integration Discovery

Use `grep` to find external system integrations:

```
# Java
RestTemplate|WebClient|@FeignClient|@KafkaListener|@SqsListener|JdbcTemplate

# .NET
HttpClient|IHttpClientFactory|IBusControl|IMessageBus
```

Read matched files with `read_file` (targeted line ranges only).

Record into `integrations[]` — include `from`, `to`, `type` (sync/async), `protocol`, `purpose`, `frequency`.

Additionally, extract messaging topic and queue names:

**Kafka topics** — use `grep`:
```
# Java — consumer
@KafkaListener\(topics

# Java — producer
kafkaTemplate\.send\(

# .NET
\.Subscribe\(|\.Produce\(|TopicName
```
For each match, read the line with `read_file` to extract the literal topic name string.
Record into `messagingTopics[]` — include `name`, `type` (`publish | subscribe`), `classFqn`, `protocol: "Kafka"`.

**Message queues** — use `grep`:
```
# Java
@RabbitListener|@SqsListener|@JmsListener|\.convertAndSend\(

# .NET
\.Publish\(|IBusControl|\.Subscribe\(
```
Record into `messagingQueues[]` — include `name`, `type` (`publish | subscribe`), `classFqn`, `protocol: "RabbitMQ | SQS | JMS | ServiceBus"`.

### Step 6b — Environment Variable Extraction

Use `grep` to find all environment variable and config references:

```
# Java Spring
@Value\("\$\{

# Java general
System\.getenv\(

# Python
os\.environ\[|os\.getenv\(

# Node.js
process\.env\.

# .NET
Environment\.GetEnvironmentVariable\(|IConfiguration\[
```

For each match, read the line to extract the variable name. Record into `environmentVariables[]` — include `name`, `defaultValue` (if present), `classFqn`, `required` (true if no default).

### Step 7 — Database Schema Extraction

Use `grep` to locate schema definitions:

```
# JPA
@Entity|@Table|@Column|@OneToMany|@ManyToOne

# DDL
CREATE TABLE|FOREIGN KEY|ALTER TABLE

# .NET EF
DbSet<|modelBuilder\.|HasForeignKey
```

Read DDL files (`.sql`) and entity classes to extract table/column structures.

Record into `tables[]`, `columns[]`, `foreignKeys[]`, `storedProcedures[]`, `databaseAccess[]`.

### Step 8 — Business Rule Extraction

Use `grep` to locate validation and computation logic:

```
validate|calculate|compute|process|check|verify|enforce|constraint|rule|policy
```

Read matched methods using `FindSymbol` with `include_body: true` only for methods of interest.

For each extracted rule, create a **first-class `BusinessRule` object** (not just a string):
- Assign a rule ID in the format `BR-{SERVICE_ABBR}-{NNN}` (e.g. `BR-ORD-001`)
- Extract a one-sentence description from the method body or Javadoc
- Record the enforcing method FQN

Record into the top-level `businessRules[]` array in the JSON Contract — **and** keep the `ruleId` reference in the relevant `classes[].businessRules[]` and `methods[].businessRules[]` fields (store just the ID string there, e.g. `"BR-ORD-001"`).

### Step 9 — Generate Discovery Documents

Write 7 markdown files to `context/{timestamp}/` using `write_file` (timestamp from Step 1).
Use the section templates defined in `discovery-doc-templates.md` (in this skill's directory)
for the exact required sections and column headings — downstream skills parse these documents
by section heading, so headings must be preserved exactly.

| File | Template in `discovery-doc-templates.md` |
|------|------------------------------------------|
| `context/{timestamp}/DISCOVERY_SUMMARY.md` | `## Template: DISCOVERY_SUMMARY.md` |
| `context/{timestamp}/ARCHITECTURE_INVENTORY.md` | `## Template: ARCHITECTURE_INVENTORY.md` |
| `context/{timestamp}/DATABASE_INVENTORY.md` | `## Template: DATABASE_INVENTORY.md` |
| `context/{timestamp}/DATABASE_SUMMARY.md` | `## Template: DATABASE_SUMMARY.md` |
| `context/{timestamp}/BUSINESS_RULES.md` | `## Template: BUSINESS_RULES.md` |
| `context/{timestamp}/INTEGRATIONS.md` | `## Template: INTEGRATIONS.md` |
| `context/{timestamp}/DEPENDENCY_MAP.md` | `## Template: DEPENDENCY_MAP.md` |

Sections marked `<!-- REQUIRED -->` must always be present (write "None identified" or "N/A" if no data).
Sections marked `<!-- IF AVAILABLE -->` are omitted when no data was found.

### Step 10 — Generate discovery.json

Write `context/{timestamp}/discovery.json` using `write_file`. The JSON must conform to the
contract below. This file is the input to Phase 2 (Neo4j load).

### Step 11 — Upload to MinIO

After all `context/{timestamp}/` files are written, upload them to MinIO so downstream skills
(`skills-microservices-architect`, `skills-microservices-designer`) can retrieve them.

**11.1 — Determine bucket name**

Derive `appName` from the workspace folder name (e.g. `c:/Users/.../shopizer` → `shopizer`).
Bucket name: `{appName}` (e.g. `shopizer`)

**11.2 — Create the bucket**

Use `mcp__minio__create_bucket` (bucket creation is idempotent — if it already exists, continue):
```
bucketName: "{appName}"
```

**11.3 — Upload all files**

Use `mcp__minio__upload_object` for each file in `context/{timestamp}/`. Store every object
under the `discovery/` prefix so downstream skills can reliably locate them:

| Local file | MinIO object |
|-----------|-------------|
| `context/{timestamp}/DISCOVERY_SUMMARY.md` | `discovery/DISCOVERY_SUMMARY.md` |
| `context/{timestamp}/ARCHITECTURE_INVENTORY.md` | `discovery/ARCHITECTURE_INVENTORY.md` |
| `context/{timestamp}/DATABASE_INVENTORY.md` | `discovery/DATABASE_INVENTORY.md` |
| `context/{timestamp}/DATABASE_SUMMARY.md` | `discovery/DATABASE_SUMMARY.md` |
| `context/{timestamp}/BUSINESS_RULES.md` | `discovery/BUSINESS_RULES.md` |
| `context/{timestamp}/INTEGRATIONS.md` | `discovery/INTEGRATIONS.md` |
| `context/{timestamp}/DEPENDENCY_MAP.md` | `discovery/DEPENDENCY_MAP.md` |
| `context/{timestamp}/discovery.json` | `discovery/discovery.json` |

Use `contentType: "text/markdown"` for `.md` files and `contentType: "application/json"` for `.json`.

**11.4 — Report results**

After all uploads complete, print a summary:
```
✅ Run artifacts saved to: context/{timestamp}/
✅ Uploaded 8 files to MinIO bucket "{appName}" under prefix "discovery/"
  ✅ discovery/DISCOVERY_SUMMARY.md
  ✅ discovery/ARCHITECTURE_INVENTORY.md
  ✅ discovery/DATABASE_INVENTORY.md
  ✅ discovery/DATABASE_SUMMARY.md
  ✅ discovery/BUSINESS_RULES.md
  ✅ discovery/INTEGRATIONS.md
  ✅ discovery/DEPENDENCY_MAP.md
  ✅ discovery/discovery.json
```
For any failed upload: `❌ discovery/{filename} — {error}` — log and continue; do not fail the overall discovery.

---

## JSON Contract

```jsonc
{
  "application": {
    "name": "string",
    "language": "Java | .NET",
    "totalLOC": 0,
    "architecturalStyles": ["string"],
    "requestFlow": ["string"],
    "architectureRisks": ["string"],
    "designRisks": ["string"]
  },
  "technologies": [
    { "name": "string", "category": "framework | library | database | messaging", "version": "string" }
  ],
  "packages": [
    { "fullyQualifiedName": "string", "name": "string", "type": "domain | infrastructure | api | config" }
  ],
  "classes": [
    {
      "fullyQualifiedName": "string",
      "name": "string",
      "package": "string",
      "classType": "Controller | ServiceClass | RepositoryClass | DomainEntity | DTO | Configuration | Unknown",
      "description": "string",
      "linesOfCode": 0,
      "numberOfMethods": 0,
      "methods": ["string"],
      "externalDependencies": ["string"],
      "businessRules": ["BR-SVC-NNN"]
    }
  ],
  "methods": [
    {
      "signature": "string (FQN-qualified, e.g. com.bank.Account.debit(double))",
      "name": "string",
      "classFqn": "string",
      "description": "string",
      "businessRules": ["BR-SVC-NNN"]
    }
  ],
  "apiEndpoints": [
    {
      "path": "string (e.g. /api/v1/orders/{id})",
      "httpMethod": "GET | POST | PUT | PATCH | DELETE",
      "handlerMethod": "string (FQN-qualified method signature)",
      "controllerFqn": "string",
      "summary": "string (from Javadoc/swagger annotation if present)"
    }
  ],
  "businessRules": [
    {
      "ruleId": "BR-{SVC}-{NNN} (e.g. BR-ORD-001)",
      "description": "string",
      "enforcedByMethod": "string (FQN-qualified method signature)",
      "classFqn": "string",
      "ruleType": "validation | invariant | calculation | policy"
    }
  ],
  "messagingTopics": [
    {
      "name": "string (literal topic name, e.g. order-created)",
      "type": "publish | subscribe",
      "classFqn": "string",
      "protocol": "Kafka"
    }
  ],
  "messagingQueues": [
    {
      "name": "string (literal queue name)",
      "type": "publish | subscribe",
      "classFqn": "string",
      "protocol": "RabbitMQ | SQS | JMS | ServiceBus"
    }
  ],
  "environmentVariables": [
    {
      "name": "string (e.g. DATABASE_URL)",
      "defaultValue": "string | null",
      "classFqn": "string",
      "required": true
    }
  ],
  "tables": [{ "name": "string" }],
  "columns": [
    { "fullyQualifiedName": "TABLE.column", "name": "string", "tableName": "string", "dataType": "string" }
  ],
  "dependencies": [
    {
      "fromFqn": "string", "toFqn": "string",
      "fromType": "Class | Package", "toType": "Class | Package",
      "type": "uses | extends | implements", "strength": 1
    }
  ],
  "methodCalls": [
    { "fromSignature": "string", "toSignature": "string" }
  ],
  "externalCalls": [
    { "fromSignature": "string", "package": "string", "externalClass": "string", "externalMethod": "string" }
  ],
  "databaseAccess": [
    { "classFqn": "string", "tableName": "string", "accessType": "READ | WRITE | READ_WRITE", "operations": ["string"], "frequency": "low | medium | high" }
  ],
  "foreignKeys": [
    { "fromTable": "string", "toTable": "string", "column": "string", "references": "string", "onDelete": "string" }
  ],
  "storedProcedures": [
    { "name": "string", "description": "string", "accessesTables": ["string"] }
  ],
  "designPatterns": [
    { "name": "string", "description": "string", "classes": ["string"] }
  ],
  "capabilities": [
    { "name": "string", "description": "string", "classes": ["string"] }
  ],
  "boundedContexts": [
    {
      "name": "string", "description": "string",
      "components": ["string"], "methods": ["string"],
      "ownedTables": ["string"],
      "totalScore": 0.0, "recommendation": "EXCELLENT | GOOD | CONSIDER", "priority": 1
    }
  ],
  "integrations": [
    { "from": "string", "to": "string", "type": "sync | async", "protocol": "REST | gRPC | MQ | DB", "purpose": "string", "frequency": "low | medium | high" }
  ]
}
```

---

## Phase 2: Neo4j Graph Load (Optional)

Only run if the user confirmed Phase 2 in Step 1, and `context/discovery.json` exists.

### Graph Node Labels

`Application`, `Package`, `Class`, `Method`, `Table`, `Column`,
`BoundedContext`, `Technology`, `StoredProcedure`, `DesignPattern`, `Capability`

### Graph Relationships

| Relationship | From → To | Notes |
|---|---|---|
| `CONTAINS` | App→Package, Package→Class, Class→Method, BC→Class, BC→Method | Structural nesting |
| `DEPENDS_ON` | Class→Class, Class→Package | Dependencies |
| `INVOKES` | Method→Method | Method calls |
| `INVOKES_EXTERNAL` | Method→Package | External calls |
| `READS_FROM` / `WRITES_TO` | Class→Table | Data access |
| `FOREIGN_KEY` | Table→Table | Schema FK |
| `ACCESSES` | StoredProcedure→Table | Proc data access |
| `EXHIBITS` | Class→DesignPattern | Design pattern |
| `REALIZED_BY` | Capability→Class | Capability realisation |
| `OWNS_DATA` | BC→Table | Data ownership |
| `INTEGRATES_WITH` | BC→BC | Integration edge |
| `USES` | App→Technology | Tech stack |

### Load Steps

For each section of `discovery.json`, use `mcp__neo4j-mcp__write-cypher` with `MERGE` (never `CREATE`).

**Application node:**
```cypher
MERGE (a:Application {name: $name})
SET a.language = $language, a.totalLOC = $totalLOC
```

**Technology node + link to Application:**
```cypher
MERGE (t:Technology {name: $name})
SET t.category = $category, t.version = $version
WITH t
MATCH (a:Application {name: $appName})
MERGE (a)-[:USES]->(t)
```

**Package node + link to Application:**
```cypher
MERGE (p:Package {fullyQualifiedName: $fqn})
SET p.name = $name, p.type = $type
WITH p
MATCH (a:Application {name: $appName})
MERGE (a)-[:CONTAINS]->(p)
```

**Class node + link to Package (now includes `classType`):**
```cypher
MERGE (c:Class {fullyQualifiedName: $fqn})
SET c.name = $name, c.package = $package,
    c.classType = $classType,
    c.description = $description, c.linesOfCode = $linesOfCode,
    c.numberOfMethods = $numberOfMethods,
    c.businessRules = $businessRules,
    c.externalDependencies = $externalDependencies
WITH c
MATCH (p:Package {fullyQualifiedName: $packageFqn})
MERGE (p)-[:CONTAINS]->(c)
```

**APIEndpoint node + `exposes` edge from Class:**
```cypher
MERGE (ep:APIEndpoint {handlerMethod: $handlerMethod})
SET ep.path = $path, ep.httpMethod = $httpMethod,
    ep.controllerFqn = $controllerFqn, ep.summary = $summary
WITH ep
MATCH (c:Class {fullyQualifiedName: $controllerFqn})
MERGE (c)-[:exposes]->(ep)
MERGE (ep)-[:handled_by]->(c)
```

**BusinessRule node + `governed_by` edge from BusinessCapability:**
```cypher
MERGE (br:BusinessRule {ruleId: $ruleId})
SET br.description = $description,
    br.ruleType = $ruleType,
    br.classFqn = $classFqn,
    br.enforcedByMethod = $enforcedByMethod
WITH br
MATCH (m:Method {signature: $enforcedByMethod})
MERGE (m)-[:ENFORCES]->(br)
```

**KafkaTopic node + `publishes`/`consumes` edge from Class:**
```cypher
MERGE (kt:KafkaTopic {name: $name})
SET kt.protocol = $protocol
WITH kt
MATCH (c:Class {fullyQualifiedName: $classFqn})
FOREACH (_ IN CASE $type WHEN 'publish'   THEN [1] ELSE [] END | MERGE (c)-[:publishes]->(kt))
FOREACH (_ IN CASE $type WHEN 'subscribe' THEN [1] ELSE [] END | MERGE (c)-[:consumes]->(kt))
```

**MQQueue node + `publishes`/`consumes` edge from Class:**
```cypher
MERGE (q:MQQueue {name: $name})
SET q.protocol = $protocol
WITH q
MATCH (c:Class {fullyQualifiedName: $classFqn})
FOREACH (_ IN CASE $type WHEN 'publish'   THEN [1] ELSE [] END | MERGE (c)-[:publishes]->(q))
FOREACH (_ IN CASE $type WHEN 'subscribe' THEN [1] ELSE [] END | MERGE (c)-[:consumes]->(q))
```

**EnvironmentVariable node + reference edge from Class:**
```cypher
MERGE (ev:EnvironmentVariable {name: $name})
SET ev.defaultValue = $defaultValue, ev.required = $required
WITH ev
MATCH (c:Class {fullyQualifiedName: $classFqn})
MERGE (c)-[:references_env]->(ev)
```

**Method node + link to Class:**
```cypher
MERGE (m:Method {signature: $signature})
SET m.name = $name, m.classFqn = $classFqn,
    m.description = $description,
    m.businessRules = $businessRules
WITH m
MATCH (c:Class {fullyQualifiedName: $classFqn})
MERGE (c)-[:CONTAINS]->(m)
```

**Table node:**
```cypher
MERGE (t:Table {name: $name})
```

**Column node + link to Table:**
```cypher
MERGE (col:Column {fullyQualifiedName: $fqn})
SET col.name = $name, col.tableName = $tableName, col.dataType = $dataType
WITH col
MATCH (t:Table {name: $tableName})
MERGE (t)-[:HAS_COLUMN]->(col)
```

**DEPENDS_ON edge (Class → Class):**
```cypher
MATCH (from:Class {fullyQualifiedName: $fromFqn})
MATCH (to:Class   {fullyQualifiedName: $toFqn})
MERGE (from)-[r:DEPENDS_ON]->(to)
SET r.type = $type, r.strength = $strength
```

**INVOKES edge (Method → Method):**
```cypher
MATCH (from:Method {signature: $fromSignature})
MATCH (to:Method   {signature: $toSignature})
MERGE (from)-[:INVOKES]->(to)
```

**INVOKES_EXTERNAL edge (Method → Package):**
```cypher
MATCH (m:Method {signature: $fromSignature})
MERGE (p:Package {fullyQualifiedName: $externalPackage})
  ON CREATE SET p.name = $externalClass
MERGE (m)-[:INVOKES_EXTERNAL]->(p)
```

**READS_FROM / WRITES_TO edge (Class → Table):**
```cypher
MATCH (c:Class {fullyQualifiedName: $classFqn})
MATCH (t:Table {name: $tableName})
FOREACH (_ IN CASE $accessType WHEN 'READ'       THEN [1] ELSE [] END |
  MERGE (c)-[:READS_FROM]->(t)
)
FOREACH (_ IN CASE $accessType WHEN 'WRITE'      THEN [1] ELSE [] END |
  MERGE (c)-[:WRITES_TO]->(t)
)
FOREACH (_ IN CASE $accessType WHEN 'READ_WRITE' THEN [1] ELSE [] END |
  MERGE (c)-[:READS_FROM]->(t)
  MERGE (c)-[:WRITES_TO]->(t)
)
```

**FOREIGN_KEY edge (Table → Table):**
```cypher
MATCH (from:Table {name: $fromTable})
MATCH (to:Table   {name: $toTable})
MERGE (from)-[r:FOREIGN_KEY {column: $column}]->(to)
SET r.references = $references, r.onDelete = $onDelete
```

**BoundedContext node + OWNS_DATA edges:**
```cypher
MERGE (bc:BoundedContext {name: $name})
SET bc.description = $description, bc.totalScore = $totalScore,
    bc.recommendation = $recommendation, bc.priority = $priority
WITH bc
UNWIND $ownedTables AS tableName
  MATCH (t:Table {name: tableName})
  MERGE (bc)-[:OWNS_DATA]->(t)
```

**DesignPattern node + EXHIBITS edge:**
```cypher
MERGE (dp:DesignPattern {name: $name})
SET dp.description = $description
WITH dp
UNWIND $classes AS fqn
  MATCH (c:Class {fullyQualifiedName: fqn})
  MERGE (c)-[:EXHIBITS]->(dp)
```

**Capability node + REALIZED_BY edge:**
```cypher
MERGE (cap:Capability {name: $name})
SET cap.description = $description
WITH cap
UNWIND $classes AS fqn
  MATCH (c:Class {fullyQualifiedName: fqn})
  MERGE (cap)-[:REALIZED_BY]->(c)
```

Load order: Application → Technologies → Packages → Classes → Methods → Tables → Columns →
Dependencies → MethodCalls → ExternalCalls → DatabaseAccess → ForeignKeys →
StoredProcedures → DesignPatterns → Capabilities → BoundedContexts → Integrations →
APIEndpoints → BusinessRules → KafkaTopics → MQQueues → EnvironmentVariables.

---

## Limitations (State at End of Phase 1)

- Dynamic dispatch and reflection-based calls will not appear in the dependency graph.
- Runtime-only integrations (feature flags, A/B configs) are not captured.
- LOC counts exclude generated code; actual complexity may differ.
- Stored procedures discovered only if `.sql` files are present in the repo.
- .NET analysis assumes standard EF Core patterns; Dapper/ADO.NET queries require manual review.
