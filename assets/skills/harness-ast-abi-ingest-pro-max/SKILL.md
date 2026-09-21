---
name: harness-ast-abi-ingest-pro-max
description: Deep AST & Multi-Language ABI Ingestion for Tagisan. Parses Tree-sitter CSTs, extracts C/C++ native headers and DWARF/ELF symbols via libclang/gimli, analyzes Go ASTs with struct tags, disassembles JVM classfile bytecode, and extracts Erlang/Elixir BEAM module exports. Enforces complete parameter type resolution, docstring preservation, default value extraction, and handling of variadics and generics.
version: 1.0.0
tags:
  - ast-parsing
  - abi-ingestion
  - tree-sitter
  - libclang
  - dwarf-elf
  - go-ast
  - jvm-bytecode
  - beam-exports
  - type-resolution
triggers:
  - ast-abi-ingest
  - tree-sitter-parse
  - c-header-ingest
  - dwarf-symbol-extract
  - elf-abi
  - go-ast-tags
  - jvm-classfile
  - beam-module-export
  - abi-extractor
  - type-signature-extractor
compatibility: ">=0.2.0"
---

# Deep AST & Multi-Language ABI Ingestion: Infallible Symbol, Syntax & Type Extraction

## Purpose & Scope
High-level AI agents cannot synthesize reliable harnesses without first obtaining ground-truth architectural facts about functions, structs, types, memory layouts, and calling conventions. Relying on regex-based code heuristics fails on macro expansions, generics, nested structures, struct tags, and compiled binary interfaces.

The `harness-ast-abi-ingest-pro-max` skill provides deep, multi-language ABI and AST extraction across source code, object files, and compiled bytecodes. It extracts complete function prototypes, parameter types, documentation, struct tag metadata, and calling conventions, converting them into a unified, machine-readable Tagisan ABI Schema.

---

## 1. Operational Invariants

```
+---------------------------------------------------------------------------------------------------+
|                                     AST & ABI INGESTION INVARIANTS                                |
+---------------------------------------------------------------------------------------------------+
|  1. COMPLETE PARAMETER TYPE RESOLUTION                                                            |
|     All types must be fully resolved to their canonical primitive or qualified name. Aliases      |
|     (e.g., `size_t`, `uintptr_t`, `String`, `custom_t`) must be traced to their underlying types.  |
+---------------------------------------------------------------------------------------------------+
|  2. PRESERVATION OF DOCSTRINGS & COMMENTS                                                         |
|     Docstrings (Rust doc comments `///`, Javadoc `/** ... */`, Go comments `//`, Python docstrings)  |
|     must be preserved verbatim and associated directly with their respective AST/ABI nodes.       |
+---------------------------------------------------------------------------------------------------+
|  3. DEFAULT VALUE PRESERVATION                                                                    |
|     Default values for arguments or struct fields (e.g., Python `= None`, C++ `= 42`, Rust       |
|     `Default::default()`) must be captured and reflected in the synthesized ABI.                 |
+---------------------------------------------------------------------------------------------------+
|  4. ROBUST HANDLING OF VARIADICS & GENERICS                                                       |
|     Variadic parameters (`...`, `*args`, `params T[]`) and generic type parameters (`<T: Clone>`) |
|     must be explicitly represented with their trait bounds and constraints preserved.             |
+---------------------------------------------------------------------------------------------------+
|  5. CALLING CONVENTION & SYMBOL MANGLE SAFETY                                                     |
|     Native binaries (ELF/DWARF, C/C++) must identify calling conventions (`cdecl`, `systemv`,     |
|     `stdcall`, `fastcall`) and demangle symbols (`abi::__cxa_demangle`) for accurate linkage.      |
+---------------------------------------------------------------------------------------------------+
```

---

## 2. Canonical Tagisan ABI Schema (Unified Intermediate Representation)

All AST and ABI ingestors emit the canonical JSON representation:

```json
{
  "$schema": "https://tagisan.dev/schemas/v1/abi-spec.json",
  "module_name": "crypto_engine",
  "source_type": "c_header",
  "version": "1.0.0",
  "types": [
    {
      "name": "HashAlgorithm",
      "kind": "enum",
      "variants": ["BLAKE3", "SHA256", "SHA512"],
      "underlying_type": "u32"
    },
    {
      "name": "DigestConfig",
      "kind": "struct",
      "fields": [
        {
          "name": "algorithm",
          "type": "HashAlgorithm",
          "doc": "Target hashing algorithm",
          "default": "BLAKE3"
        },
        {
          "name": "key_bytes",
          "type": "Option<Vec<u8>>",
          "doc": "Optional HMAC key",
          "default": null
        }
      ]
    }
  ],
  "functions": [
    {
      "name": "compute_digest",
      "mangled_symbol": "_Z14compute_digestPKhmP12DigestConfig",
      "calling_convention": "cdecl",
      "doc": "Computes cryptographic digest over buffer in memory.",
      "parameters": [
        {
          "name": "buffer",
          "type": "*const u8",
          "is_variadic": false,
          "doc": "Pointer to source byte buffer"
        },
        {
          "name": "len",
          "type": "usize",
          "is_variadic": false,
          "doc": "Length of buffer in bytes"
        },
        {
          "name": "config",
          "type": "*const DigestConfig",
          "is_variadic": false,
          "doc": "Hashing configuration"
        }
      ],
      "return_type": "i32"
    }
  ]
}
```

---

## 3. Deep Ingestion Implementations by Language & Format

### 1. Tree-sitter Concrete Syntax Tree (CST) Ingestion
Tree-sitter provides error-tolerant, incremental CST parsing across dozens of languages.
- **Node Traversal**: Traverse nodes by kind (`function_item`, `impl_item`, `struct_item`, `parameter`).
- **Capture Queries**: Use Tree-sitter S-expression queries:
  ```scheme
  (function_item
    name: (identifier) @fn.name
    parameters: (parameters) @fn.params
    return_type: (type_identifier)? @fn.return
    doc: (line_comment)* @fn.doc)
  ```

### 2. C/C++ Native Header & DWARF/ELF Symbol Extraction
- **libclang Clang AST**:
  - Traverses `CXCursor_FunctionDecl`, `CXCursor_StructDecl`, `CXCursor_EnumDecl`.
  - Resolves typedef chains (`clang_getCanonicalType`) and extracts doxygen comments (`clang_Cursor_getRawCommentText`).
- **DWARF & ELF Symbol Parser (`gimli` & `object`)**:
  ```rust
  use object::{Object, ObjectSection, ObjectSymbol};
  use gimli::{Dwarf, EndianSlice, RunTimeEndian};

  pub fn extract_elf_symbols(data: &[u8]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
      let file = object::File::parse(data)?;
      let mut symbols = Vec::new();
      for sym in file.symbols() {
          if let Ok(name) = sym.name() {
              if sym.is_global() && !sym.is_undefined() {
                  symbols.push(name.to_string());
              }
          }
      }
      Ok(symbols)
  }
  ```

### 3. Go AST & Struct Tag Reflection
- Inspects Go packages using `go/parser`, `go/ast`, and `go/types`.
- Extracts `struct` tags (`json:"..."`, `db:"..."`, `yaml:"..."`):
  ```go
  // Ingestion parses struct field declarations:
  type UserPayload struct {
      UserID    int64  `json:"user_id" db:"pk,id" validate:"required,min=1"`
      Username  string `json:"username" db:"user_name" validate:"required,alphanum"`
  }
  ```
- Detects receiver methods, pointer vs value receivers, interfaces, and embedded struct composition.

### 4. JVM Bytecode & Classfile Disassembly
- Parses binary `.class` format according to the JVM Specification (Java 8 through 25):
  - **Constant Pool**: Disassembles `CONSTANT_Utf8`, `CONSTANT_Methodref`, `CONSTANT_Class`.
  - **Methods (`method_info`)**: Parses `access_flags`, `name_index`, `descriptor_index` (e.g. `(Ljava/lang/String;I)Z`), and `Code` attribute.
  - **RuntimeVisibleAnnotations**: Preserves annotations (`@JsonProperty`, `@NotNull`, `@Route`).

### 5. Erlang/Elixir BEAM Module Chunk Extraction
- Reads BEAM binary format and extracts the following standard chunks:
  - `Atom` / `AtU8`: Atom table (module name, function names).
  - `ExpT`: Export table containing `{FunctionAtom, Arity, Label}` tuples.
  - `Attr`: Module attributes and compile metadata (`@spec` type specifications, `@doc` documentation).
  - `Code`: BEAM opcodes for control-flow verification.

---

## 4. Extraction & Ingestion Pipeline

1. **Source/Artifact Identification**: Detect file type (`.h`, `.hpp`, `.so`, `.dylib`, `.go`, `.class`, `.beam`, `.rs`, `.ts`).
2. **Parser Initialization**: Spin up the corresponding native parser or Tree-sitter grammar.
3. **AST Traversal & Type Graph Construction**: Build an internal directed acyclic graph of types, resolving dependencies and struct fields.
4. **Invariant Validation**: Verify that no parameter types remain unresolved, docstrings are attached, and default values are recorded.
5. **Tagisan ABI Schema Emission**: Serialize the resulting ABI schema into JSON for downstream polyglot CLI synthesis.
