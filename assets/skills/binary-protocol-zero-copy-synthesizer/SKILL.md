---
name: binary-protocol-zero-copy-synthesizer
description: Zero-copy binary wire protocol synthesis using zerocopy and nom. Enforces strict byte-alignment validation, infallible bounds checks without panicking, buffer overflow elimination, and compile-time layout verification for wire protocol parsers.
version: 1.0.0
tags:
  - zero-copy
  - binary-protocols
  - zerocopy
  - nom
  - wire-format
  - endianness
  - alignment
  - memory-safety
triggers:
  - zero-copy
  - binary-protocol
  - wire-protocol
  - zerocopy
  - nom-parser
  - byte-alignment
  - unaligned-access
  - packet-parsing
  - deserializer-synthesis
compatibility: ">=0.2.0"
---

# Binary Protocol & Zero-Copy Wire Synthesizer: Hardware-Aligned Infallible Serialization

## Purpose & Scope
High-performance networked systems, distributed storage engines, telemetry pipelines, and low-latency financial feeds achieve optimal throughput and sub-microsecond latency by completely bypassing heap allocations, intermediate data copies, and parsing deserialization overheads.

This skill synthesizes production-grade, zero-copy wire protocol decoders and encoders in safe Rust using `zerocopy` and `nom`. It enforces strict byte-alignment validation, compile-time layout verification, infallible bounds checks without panicking, and endianness normalization.

---

## 1. Operational Invariants (Zero-Copy Wire Protocol Protocol)

### Invariant 1: Byte-Alignment & Transmutation Safety Proof
- **MANDATORY**: All packet headers and fixed-size wire structures must be annotated with `#[repr(C)]` or `#[repr(C, packed)]`. Alignments must be validated to match target hardware architecture.
- **ALWAYS**: Derive or implement `zerocopy::{FromBytes, IntoBytes, Unaligned, KnownLayout, Immutable}` on wire structures to guarantee that any bit pattern constitutes a valid representation and that pointer transmutation is completely sound.
- **STRICT_REJECT**: Reject any zero-copy cast or transmutation on multi-byte integer types (`u16`, `u32`, `u64`, `f32`, `f64`) that does not explicitly guarantee memory alignment or derive `Unaligned`. Unaligned reads on strict architectures trigger bus errors (SIGBUS) or severe latency penalties.

### Invariant 2: Infallible Bounds Checks & Panicking Elimination
- **MANDATORY**: Parsing and slicing wire buffers must be completely infallible and panic-free. Every byte slice access must use non-panicking boundary methods (e.g. `slice.get(..N)`, `split_at_checked`, or `nom::combinator::verify`).
- **NEVER**: Allow raw indexing (`buf[i]`), unconditional slice ranges (`buf[start..end]`), or `.unwrap()` / `.expect()` in production packet deserialization routines.
- **ALWAYS**: Return strongly-typed error enums (`Result<T, WireParseError>`) that convey the exact byte shortfall, unaligned offset, or invalid magic header without terminating the host process.

### Invariant 3: Endianness Normalization & Wire Invariants
- **MANDATORY**: Wire protocol definitions must explicitly declare byte order for every multi-byte scalar field. Fields transmitted across networks or disks must use endian-aware wrappers (e.g. `zerocopy::byteorder::{U16, U32, U64, BE, LE}`) or explicit conversion calls (`u32::from_be_bytes`, `u32::from_le_bytes`).
- **NEVER**: Rely on host native endianness (`from_ne_bytes`) for wire protocols or persistent storage formats.
- **ALWAYS**: Validate length fields, CRC/checksums, and payload bounds before dispatching payload slices to downstream handlers.

### Invariant 4: Zero-Copy Parsing Pipeline with `nom` / Safe Transmutation
- **MANDATORY**: Zero-copy packet decoders must borrow directly from input slices (`&'a [u8]`) yielding borrowed reference views (`&'a WireHeader`, `&'a [u8]` payload) with zero heap allocations (`alloc::vec::Vec`).
- **STRICT_REJECT**: Reject designs that perform intermediate copying or `Vec<u8>` cloning during packet inspection, deserialization, or payload forwarding.

---

## 2. Canonical Zero-Copy Synthesis Patterns

### Hardware-Aligned Zero-Copy Wire Header (Rust)
```rust
use zerocopy::{FromBytes, IntoBytes, KnownLayout, Immutable, byteorder::{U16, U32, BE}};

#[derive(Clone, Copy, Debug, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct WireHeader {
    pub magic: [u8; 4],
    pub version: u8,
    pub flags: u8,
    pub sequence: U16<BE>,
    pub payload_len: U32<BE>,
    pub checksum: U32<BE>,
}

impl WireHeader {
    pub const SIZE: usize = core::mem::size_of::<Self>();

    #[inline]
    pub fn parse_from_prefix(buf: &[u8]) -> Result<(&Self, &[u8]), WireError> {
        if buf.len() < Self::SIZE {
            return Err(WireError::BufferTooShort { needed: Self::SIZE, available: buf.len() });
        }
        let (header_bytes, payload) = buf.split_at(Self::SIZE);
        // zerocopy safe transmutation with compile-time alignment proof
        let header = zerocopy::Ref::<_, Self>::from_bytes(header_bytes)
            .map_err(|_| WireError::AlignmentMismatch)?;
        
        let payload_len = header.payload_len.get() as usize;
        if payload.len() < payload_len {
            return Err(WireError::IncompletePayload { expected: payload_len, got: payload.len() });
        }
        
        Ok((header.into_ref(), &payload[..payload_len]))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum WireError {
    BufferTooShort { needed: usize, available: usize },
    AlignmentMismatch,
    IncompletePayload { expected: usize, got: usize },
    InvalidMagic,
}
```

---

## 3. Tool Invocations

Use `binary_protocol_synthesizer` to inspect wire layouts and generate zero-copy Rust code:
- `binary_protocol_synthesizer(action: "analyze_wire_format", protocol_name: "SensorTelemetry", wire_spec: "{\"fields\": [{\"name\": \"id\", \"type\": \"u32\"}, {\"name\": \"val\", \"type\": \"f64\"}]}")`
- `binary_protocol_synthesizer(action: "synthesize_zerocopy", protocol_name: "NetworkFrame", wire_spec: "{\"fields\": [...]}")`
- `binary_protocol_synthesizer(action: "verify_safety", protocol_name: "NetworkFrame", wire_spec: "{\"fields\": [...]}")`
