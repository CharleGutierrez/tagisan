---
name: data-dictionary-minispecs
description: "Structured System Specification (Tom DeMarco): formal data dictionary definitions, structured English mini-specifications, process specifications, and DFD conservation balancing."
triggers: ["data dictionary", "structured english", "mini-specs", "process specification", "demarco", "balancing rule", "data schema notation", "formal specification"]
---

# Structured Specification & Mini-Specs (Tom DeMarco)

This skill instructs the agent to apply Tom DeMarco's rigorous data modeling and process specification techniques (*Structured Analysis and System Specification*) to define unambiguous data structures and algorithmic pseudo-specs.

## 1. Formal Data Dictionary Notation
All system data flows and data store models must be documented using DeMarco's algebraic data syntax:
- `=` is composed of (definition)
- `+` AND (concatenation of required fields)
- `[ | ]` OR (exclusive choice / discriminated union)
- `{}` Iteration / Array (0 or more occurrences; `{item}^N` indicates maximum $N$)
- `()` Optional field (0 or 1 occurrence)
- `*...*` Semantic comment or unit invariant

### Example:
```text
CivilCaseFiling = CaseId + CaseTitle + PlaintiffInfo + DefendantInfo + ClaimAmount + [ RealPropertyClaim | MonetaryDamages | Injunction ] + (FeeExemptionCertificate)
ClaimAmount     = Currency + * must be non-negative Decimal *
PlaintiffInfo   = 1{PersonOrEntityName + Address + ContactNumber}10
```

## 2. Structured English Mini-Specifications
For every leaf-level process bubble in a system, write a Mini-Spec using **Structured English**:
1. **Imperative Action Verbs**: Use only clear, actionable verbs (`COMPUTE`, `VALIDATE`, `LOOKUP`, `DISPATCH`, `PERSIST`, `EMIT`).
2. **Deterministic Control Structures**:
   - `IF <condition> THEN ... ELSE ... ENDIF`
   - `CASE <expression> OF ... ENDCASE`
   - `FOR EACH <item> IN <collection> DO ... ENDFOR`
   - `WHILE <condition> DO ... ENDWHILE`
3. **Zero Ambiguity**: Eliminate vague English adjectives (e.g., "reasonable", "appropriate", "as needed"). All thresholds must be concrete constants or configured variables.

### Example Mini-Spec:
```text
PROCESS: AssessLegalFee
INPUT: CivilCaseFiling
OUTPUT: LegalFeeAssessmentReceipt

1. VALIDATE that ClaimAmount >= 0; IF invalid, EMIT ValidationError("Claim amount must be non-negative") AND STOP.
2. LOOKUP basic filing fee bracket from Rule141Table using ClaimAmount.
3. COMPUTE JDFAllocation = BasicFee * 0.20.
4. COMPUTE SAJAllocation = BasicFee * 0.80.
5. COMPUTE TotalFee = BasicFee + LegalResearchFundFee + MediationFee.
6. PERSIST AssessmentRecord into AssessmentLedger.
7. EMIT LegalFeeAssessmentReceipt containing breakdown and TotalFee.
```

## 3. The Conservation / Balancing Rule
- **Rule of Data Conservation**: A process cannot create data out of thin air or discard necessary data.
- **Rule of Leveled Balancing**: The input and output data flows of a child diagram must exactly balance the inputs and outputs of the corresponding parent process bubble.
