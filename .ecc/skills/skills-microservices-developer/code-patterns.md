# Code Patterns Reference
# Used by: skills-microservices-developer/SKILL.md — Steps 3, 4, 5
# Shows canonical patterns for domain models, services, and traceability per stack.

## Traceability Header (mandatory on all generated classes)

Every generated class must include a doc comment with these four tags:

```
@specification  spec.md Section X.Y — <section title>
@requirement    FR-{SVC}-{NNN} — <requirement name>
@task           TASK-{SVC}-{NNN} — <task name>
@governance     shopizer-context-studio MCP server
```

---

## Spring Boot — Domain Model Pattern

```java
/**
 * Inventory aggregate root.
 *
 * @specification spec.md Section 7.1 - Aggregate Roots
 * @requirement   FR-INV-001 - Create Inventory
 * @task          TASK-INV-003 - Generate Domain Model
 * @governance    shopizer-context-studio MCP server
 */
@Entity
@Table(name = "inventory")
@Getter @Setter @NoArgsConstructor
public class Inventory {

    @Id @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(nullable = false, unique = true)
    private String sku;

    @Column(nullable = false)
    private Integer quantity;

    @Column(nullable = false)
    private Integer reserved;

    @Column(nullable = false)
    private Boolean available;

    /**
     * Reserve inventory for an order.
     *
     * @specification spec.md Section 4 - CMD-INV-003
     * @businessRule  BR-INV-001 - Product availability check
     * @businessRule  BR-INV-004 - Reservation expiry (15 min TTL)
     */
    public ReservationResult reserve(String orderId, Integer quantity) {
        // BR-INV-001: Availability check
        if (!canReserve(quantity)) {
            throw new InsufficientInventoryException(sku, getAvailableQuantity(), quantity);
        }
        Reservation reservation = new Reservation(
            UUID.randomUUID().toString(), orderId, quantity,
            LocalDateTime.now().plusMinutes(15) // BR-INV-004
        );
        this.reserved += quantity;
        this.reservations.add(reservation);
        return new ReservationResult(reservation.getId(), true);
    }

    private boolean canReserve(Integer qty) { return available && getAvailableQuantity() >= qty; }
    private Integer getAvailableQuantity()  { return quantity - reserved; }
}
```

## Spring Boot — Service Pattern

```java
/**
 * Inventory service implementation.
 *
 * @specification spec.md Section 2 - Functional Requirements
 * @plan          plan.md Section 2.2 - Application Layer
 * @task          TASK-INV-006 - Generate Service Layer
 * @governance    shopizer-context-studio MCP server
 */
@Service @RequiredArgsConstructor @Slf4j @Transactional
public class InventoryServiceImpl implements InventoryService {

    private final InventoryRepository repository;
    private final InventoryMapper mapper;
    private final InventoryEventPublisher eventPublisher;
    private final AuditService auditService;

    /**
     * Create inventory record.
     *
     * @specification spec.md FR-INV-001
     * @businessRule   BR-INV-002 - Non-negative quantity
     */
    @Override
    public InventoryResponse create(CreateInventoryCommand cmd, String correlationId) {
        log.info("Creating inventory sku={} correlationId={}", cmd.getSku(), correlationId);
        validateBusinessRules(cmd);
        Inventory entity = mapper.toEntity(cmd);
        Inventory saved  = repository.save(entity);
        eventPublisher.publishInventoryCreated(saved, correlationId);   // EVT-INV-001
        auditService.log("CREATE", saved.getId(), null, saved, correlationId);
        log.info("Inventory created id={} sku={}", saved.getId(), saved.getSku());
        return mapper.toResponse(saved);
    }

    private void validateBusinessRules(CreateInventoryCommand cmd) {
        if (cmd.getQuantity() == null)
            throw new ValidationException("VAL-012", "Quantity is required");
        if (cmd.getQuantity() < 0)
            throw new BusinessRuleException("BR-INV-002", "Quantity cannot be negative");
    }
}
```

---

## Python FastAPI — Domain Model Pattern

```python
"""
Inventory aggregate root.

Specification: spec.md Section 7.1 - Aggregate Roots
Requirement:   FR-INV-001 - Create Inventory
Task:          TASK-INV-003 - Generate Domain Model
Governance:    shopizer-context-studio MCP server
"""
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from typing import List, Optional
from uuid import uuid4

@dataclass
class Inventory:
    id: Optional[int] = None
    sku: str = ""
    quantity: int = 0
    reserved: int = 0
    available: bool = True
    reservations: List["Reservation"] = field(default_factory=list)

    @property
    def available_quantity(self) -> int:
        return self.quantity - self.reserved

    def reserve(self, order_id: str, quantity: int, ttl_minutes: int = 15) -> str:
        """
        Reserve inventory for an order.

        Specification: spec.md Section 4 - CMD-INV-003
        Business Rule: BR-INV-001 - Product availability check
        Business Rule: BR-INV-004 - Reservation expiry
        """
        if not self.available or self.available_quantity < quantity:  # BR-INV-001
            raise InsufficientInventoryError(
                f"SKU {self.sku}: available={self.available_quantity}, requested={quantity}"
            )
        reservation_id = str(uuid4())
        expires_at = datetime.utcnow() + timedelta(minutes=ttl_minutes)  # BR-INV-004
        self.reserved += quantity
        self.reservations.append(
            Reservation(id=reservation_id, order_id=order_id,
                        quantity=quantity, expires_at=expires_at)
        )
        return reservation_id
```

---

## Node.js — Domain Model Pattern (TypeScript)

```typescript
/**
 * Inventory aggregate root.
 *
 * @specification spec.md Section 7.1 - Aggregate Roots
 * @requirement   FR-INV-001 - Create Inventory
 * @task          TASK-INV-003 - Generate Domain Model
 * @governance    shopizer-context-studio MCP server
 */
export class Inventory {
  constructor(
    public id?: number,
    public sku: string = '',
    public quantity: number = 0,
    public reserved: number = 0,
    public available: boolean = true,
    public reservations: Reservation[] = []
  ) {}

  get availableQuantity(): number { return this.quantity - this.reserved; }

  /**
   * Reserve inventory for an order.
   * @specification spec.md Section 4 - CMD-INV-003
   * @businessRule BR-INV-001 - Product availability check
   * @businessRule BR-INV-004 - Reservation expiry (15 min TTL)
   */
  reserve(orderId: string, quantity: number, ttlMinutes = 15): string {
    if (!this.available || this.availableQuantity < quantity) {   // BR-INV-001
      throw new InsufficientInventoryError(
        `SKU ${this.sku}: available=${this.availableQuantity}, requested=${quantity}`
      );
    }
    const reservationId = uuidv4();
    const expiresAt = new Date(Date.now() + ttlMinutes * 60_000);  // BR-INV-004
    this.reserved += quantity;
    this.reservations.push({ id: reservationId, orderId, quantity, expiresAt, status: 'ACTIVE' });
    return reservationId;
  }
}
```

---

## Business Rule Exception Pattern

All business rule violations must throw a typed exception referencing the rule ID:

**Java**:
```java
throw new BusinessRuleException("BR-INV-001", "Quantity cannot be negative");
throw new ValidationException("VAL-012", "Quantity is required");
```

**Python**:
```python
raise BusinessRuleError(rule_id="BR-INV-001", message="Quantity cannot be negative")
raise ValidationError(rule_id="VAL-012", message="Quantity is required")
```

**TypeScript**:
```typescript
throw new BusinessRuleError('BR-INV-001', 'Quantity cannot be negative');
throw new ValidationError('VAL-012', 'Quantity is required');
```

---

## Correlation ID Pattern

Every service method and controller endpoint must accept and propagate `X-Correlation-ID`:

**Java controller**:
```java
@GetMapping("/{id}")
public ResponseEntity<InventoryResponse> getById(
    @PathVariable Long id,
    @RequestHeader("X-Correlation-ID") String correlationId
) {
    log.info("getById id={} correlationId={}", id, correlationId);
    // pass correlationId to service and downstream calls
}
```

**Python FastAPI**:
```python
@router.get("/{id}")
async def get_by_id(id: int, x_correlation_id: str = Header(...)):
    logger.info("get_by_id", id=id, correlation_id=x_correlation_id)
```
