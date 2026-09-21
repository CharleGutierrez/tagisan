---
name: harness-protocol-serialization-pro-max
description: Universal Protocol & Serialization Ingestion for Tagisan. Synthesizes CLI harnesses from gRPC/Protobuf (.proto) definitions, GraphQL schemas and introspection, SQL relational schemas (PostgreSQL, MySQL, SQLite) to typed CRUD CLIs, and binary wire protocols (FIX 4.2/5.0, NASDAQ ITCH/OUCH, MQTT v5, Modbus RTU/TCP). Enforces schema validation, round-trip serialization preservation, type-safe enum mapping, and robust connection pooling.
version: 1.0.0
tags:
  - protocol-ingestion
  - serialization
  - grpc-protobuf
  - graphql
  - sql-crud
  - fix-protocol
  - nasdaq-itch
  - mqtt
  - modbus-scada
  - wire-codecs
triggers:
  - protocol-serialization
  - grpc-to-cli
  - proto-to-cli
  - graphql-to-cli
  - sql-to-crud
  - fix-wire-codec
  - itch-ouch-codec
  - mqtt-packet-codec
  - modbus-scada-harness
  - schema-to-cli
compatibility: ">=0.2.0"
---

# Universal Protocol & Serialization Ingestion: Schema-Driven Polyglot Wire Codecs & CRUD Synthesis

## Purpose & Scope
Modern infrastructure operates across dozens of heterogeneous data interchange protocols: gRPC/Protobuf in microservice meshes, GraphQL in API gateways, SQL databases in operational stores, FIX and ITCH/OUCH in financial trading engines, and MQTT/Modbus in IoT and SCADA systems.

The `harness-protocol-serialization-pro-max` skill autonomously ingests formal schema definitions and wire format specifications, compiling them into battle-tested, type-safe CLI harnesses and zero-copy wire codecs.

---

## 1. Operational Invariants

```
+---------------------------------------------------------------------------------------------------+
|                            PROTOCOL & SERIALIZATION INVARIANTS                                    |
+---------------------------------------------------------------------------------------------------+
|  1. SCHEMA VALIDATION & PARSING SOUNDNESS                                                         |
|     Schemas must be strictly validated before code synthesis. Unknown fields, broken references,  |
|     or circular dependencies must trigger immediate descriptive errors with line numbers.         |
+---------------------------------------------------------------------------------------------------+
|  2. ROUND-TRIP SERIALIZATION PRESERVATION                                                         |
|     For all serialized payloads, `decode(encode(msg)) == msg`. Zero data loss, truncation, or    |
|     precision degradation across IEEE 754 floating point numbers and 64-bit integers.             |
+---------------------------------------------------------------------------------------------------+
|  3. TYPE-SAFE ENUM & VARIANT MAPPING                                                              |
|     Enums across all protocols (Protobuf, GraphQL, SQL, FIX) must be mapped to closed type-safe    |
|     variants with exhaustive match checks. Unknown variants must fall back to a typed Unknown(u32)|
|     variant rather than failing ungracefully.                                                     |
+---------------------------------------------------------------------------------------------------+
|  4. CONNECTION POOLING & SOCKET REUSE                                                             |
|     Synthesized network CLIs must maintain reusable connection pools (HTTP/2 keepalive for gRPC,  |
|     r2d2/sqlx pool for SQL, persistent TCP sessions with heartbeat for FIX/MQTT).                 |
+---------------------------------------------------------------------------------------------------+
|  5. STRICT BINARY WIRE ALIGNMENT & CHECKSUM INTEGRITY                                             |
|     Binary protocols (ITCH, Modbus, FIX) must strictly validate packet lengths, offsets, and      |
|     checksums (CRC16-IBM, CheckSum Tag 10) before attempting any field extraction.                |
+---------------------------------------------------------------------------------------------------+
```

---

## 2. Protocol Ingestion & Synthesis Engines

### 1. gRPC & Protocol Buffers (`.proto`) to CLI
- Parses `.proto` AST (proto2/proto3 syntax).
- Maps each `service` to a top-level command and each `rpc` method to a subcommand.
- Transforms complex message fields into CLI flags:
  ```protobuf
  syntax = "proto3";
  package telemetry.v1;

  service MetricsService {
    rpc PushMetric (MetricPayload) returns (MetricResponse);
  }

  message MetricPayload {
    string metric_name = 1;
    double value = 2;
    map<string, string> labels = 3;
  }
  ```
  **Synthesized CLI invocation**:
  ```bash
  tgs-metrics push-metric --metric-name "cpu_usage" --value 84.5 --labels "host=node-1,env=prod" --json
  ```

### 2. GraphQL Schema & Introspection to CLI
- Accepts `.graphql` SDL or queries `__schema` via live HTTP introspection.
- Synthesizes queries and mutations into CLI subcommands.
- Generates GraphQL document strings with JSON variable binding:
  ```graphql
  type User {
    id: ID!
    name: String!
    email: String!
    role: UserRole!
  }
  enum UserRole { ADMIN, USER, GUEST }
  type Query {
    getUser(id: ID!): User
  }
  ```
  **Synthesized CLI**:
  ```bash
  tgs-api user get --id "usr_12345" --fields "id,name,role" --json
  ```

### 3. SQL Relational Schemas to CRUD CLIs (Postgres, MySQL, SQLite)
- Extracts table definitions, primary keys, foreign keys, and column constraints:
  ```sql
  CREATE TABLE orders (
    order_id BIGSERIAL PRIMARY KEY,
    customer_id BIGINT NOT NULL,
    amount NUMERIC(12, 2) NOT NULL,
    status VARCHAR(32) DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
  );
  ```
- Generates subcommands: `create`, `get`, `list`, `update`, `delete`.
- Implements connection pooling, query parameterization (preventing SQL injection), and streaming cursor pagination.

### 4. Binary Wire Protocols Codecs
- **Financial FIX 4.2 / 5.0 (Tag-Value)**:
  - Validates checksum `10=***`: `(sum(bytes) % 256)`.
  - Parses standard tags (`35=MsgType`, `49=SenderCompID`, `56=TargetCompID`, `38=OrderQty`).
  ```rust
  pub fn compute_fix_checksum(buf: &[u8]) -> u8 {
      let mut sum: u32 = 0;
      for &b in buf {
          sum = sum.wrapping_add(b as u32);
      }
      (sum % 256) as u8
  }
  ```
- **NASDAQ TotalView ITCH 5.0 (Zero-Copy Binary)**:
  - Handles message types: `'A'` (Add Order), `'E'` (Order Executed), `'X'` (Order Cancel).
  - Nanosecond timestamp reconstruction from 6-byte big-endian integers.
- **MQTT v5 Packet Framing**:
  - Variable-byte integer length decoding.
  - Topic matching filters (`#` and `+`).
- **Modbus RTU / TCP (Industrial SCADA)**:
  - CRC16-IBM calculation for RTU frames:
  ```rust
  pub fn calculate_modbus_crc16(buf: &[u8]) -> u16 {
      let mut crc: u16 = 0xFFFF;
      for &b in buf {
          crc ^= b as u16;
          for _ in 0..8 {
              if (crc & 0x0001) != 0 {
                  crc = (crc >> 1) ^ 0xA001;
              } else {
                  crc >>= 1;
              }
          }
      }
      crc
  }
  ```

---

## 3. Autonomous Protocol Synthesis Workflow

1. **Protocol Ingestion**: Analyze input schema (`.proto`, `.graphql`, `.sql`, or binary spec file).
2. **Type Mapping**: Map protocol primitive and complex types to Tagisan intermediate types.
3. **Codec & Transport Synthesis**: Generate client connection pools, serialization encoders/decoders, and error handlers.
4. **CLI Harness Assembly**: Bind transport methods to CLI subcommands with `--json` envelope support.
5. **Round-Trip Invariant Verification**: Generate test fixtures to verify `decode(encode(sample)) == sample` across all supported messages.
