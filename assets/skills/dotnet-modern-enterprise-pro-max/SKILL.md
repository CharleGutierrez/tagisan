---
name: dotnet-modern-enterprise-pro-max
description: Autonomous Master Engine for Modern .NET 9/10 & C# 13/14 High-Performance Enterprise Architecture. Covers zero-allocation memory pipelines, Native AOT compilation, Entity Framework Core 9 query optimization, Vertical Slice Architecture, and resilient microservices with Microsoft.Extensions.Resilience. Triggers: dotnet, csharp, c#, aspnet, efcore, native-aot, span, mediatr, masstransit, wolverine, zero-allocation.
version: 1.0.0
tags:
  - dotnet
  - csharp
  - efcore
  - native-aot
  - aspnetcore
compatibility: ">=0.2.0"
---

# Modern .NET & C# Enterprise Architecture Pro Max

## Purpose & Scope
The `dotnet-modern-enterprise-pro-max` skill automates architectural design, performance auditing, and code synthesis for enterprise .NET 9/10 and C# 13/14 systems.

## 1. Core Architectural Pillars

### Pillar 1: C# 13/14 Modern Idioms & Zero-Allocation Hot Paths
- **Primary Constructors & Record Types**: Use immutable records and concise primary constructors for domain entities, commands, and DTOs.
- **Span & Memory Zero-Copy**:
  - Use `Span<T>` and `ReadOnlySpan<char>` for parsing and string manipulation.
  - Prefer `ArrayPool<T>.Shared` or stack allocation (`stackalloc`) for transient buffers.
  - Return `ValueTask<T>` on asynchronous methods that frequently complete synchronously.
  - Use UTF-8 string literals (`"sample"u8`) to avoid runtime ASCII/UTF-8 transcoding overhead.

### Pillar 2: EF Core 9 Query Optimization & Invariants
- **Avoid Cartesian Explosions**: Enforce `.AsSplitQuery()` on queries including multiple 1:N collections.
- **No-Tracking by Default**: Use `.AsNoTracking()` for all read-only query endpoints.
- **Compiled Models & Interceptors**: Generate compiled models (`dotnet ef dbcontext optimize`) for microservices to minimize startup cold start.
- **Batch Operations**: Use `ExecuteUpdateAsync()` and `ExecuteDeleteAsync()` to bypass in-memory change tracking when mutating bulk rows.

### Pillar 3: Resilient ASP.NET Core & Native AOT
- **Standard Resilience Pipeline**: Configure `AddStandardResilienceHandler()` (retries with exponential backoff + jitter, circuit breaker, timeout, rate limiting).
- **Native AOT Compatibility**:
  - Eliminate reflection calls, dynamic assembly generation, and unbound JSON serialization.
  - Use source-generated System.Text.Json serializers (`[JsonSerializable]`).
  - Verify zero trimming warnings with `<PublishAot>true</PublishAot>`.

### Pillar 4: Modular Architecture (Vertical Slice & CQRS)
- Group features by business capability rather than technical layers.
- Implement MediatR, Wolverine, or FastEndpoints handlers with fluent validation pipelines.

## 2. Strict Invariants
1. **No Sync-Over-Async**: Never call `.Result` or `.Wait()` on Task/ValueTask; prevent thread-pool starvation.
2. **CancellationToken Propagation**: Every asynchronous API must accept and propagate a `CancellationToken`.
3. **Structured Logging**: Use high-performance source-generated `[LoggerMessage]` rather than string formatting in logs.
