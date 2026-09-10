# Development Document Template

Use this template only when the user explicitly asks to maintain the project development document.

This is a public, scenario-neutral template focused on explaining how the current code actually works.

## Standard Structure

```md
# XXX - Current Implementation Notes

## Functional Goal

## Current Progress

## Document Purpose

## Layering and Responsibilities

## Core Classes and Responsibilities

## Key Method Notes

## Key Objects, Fields, and Statuses

## Full Process Flow

## Method Call Chain

## Current Characteristics and Limitations

## Edge Cases and Special Notes

## Maintenance and Extension Guidance

## Known Issues and Improvement Ideas

## Current Error Handling

## Pending Work

## Known Risks and Notes

## Summary
```

## Writing Rules

1. Describe the current implementation, not a future design.
2. Explain the high-level structure before listing detailed flows.
3. If there are too many classes or methods, document only the core ones.
4. The method call chain is mandatory.
5. In the method call chain, every line should include:
   - input parameters
   - return value
   - purpose

## Method Call Chain Example

```md
## Method Call Chain

Overall chain summary:
The request first enters the entry layer, then moves into the main service for orchestration, persists core data, executes post-processing, and finally returns a result.

Request enters the system
-> `XXXController.handleSubmit(request)`
   - Input: `request`
   - Output: `Result`
   - Purpose: Accept the request and invoke the main service
-> `XXXGateway.process(command, allOrNone)`
   - Input: `command`, `allOrNone`
   - Output: `ProcessResult`
   - Purpose: Route the request into the next service layer
-> `XXXService.submit(commands, allOrNone)`
   - Input: `commands`, `allOrNone`
   - Output: `List<Result>`
   - Purpose: Execute the main business flow and aggregate results
-> `XXXService.buildContext(command)`
   - Input: `command`
   - Output: `Context`
   - Purpose: Build validation and execution context
-> `XXXService.persistCoreData(context)`
   - Input: `context`
   - Output: persisted data result or no direct return
   - Purpose: Persist the core business data
-> `XXXPostService.handleAfterPersist(context)`
   - Input: `context`
   - Output: no return or post-processing result
   - Purpose: Handle post-processing, notifications, or async work
-> Return `Result`
   - Purpose: Return the final result to the caller
```

## Public Use Notes

- Keep the template neutral and reusable.
- Do not hard-code project-specific business chapters into the public template.
- Add extra sections only when the current task truly needs them.
