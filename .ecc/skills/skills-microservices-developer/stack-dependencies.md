# Stack Dependencies Reference
# Used by: skills-microservices-developer/SKILL.md — Step 2 (project scaffold)

## Spring Boot 3.x (Java 17)

### pom.xml dependencies
```xml
<dependencies>
    <!-- Web -->
    <dependency>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-web</artifactId>
    </dependency>

    <!-- Data -->
    <dependency>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-data-jpa</artifactId>
    </dependency>
    <dependency>
        <groupId>org.postgresql</groupId>
        <artifactId>postgresql</artifactId>
    </dependency>
    <dependency>
        <groupId>org.flywaydb</groupId>
        <artifactId>flyway-core</artifactId>
    </dependency>

    <!-- Security -->
    <dependency>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-security</artifactId>
    </dependency>
    <dependency>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-oauth2-resource-server</artifactId>
    </dependency>

    <!-- Messaging -->
    <dependency>
        <groupId>org.springframework.kafka</groupId>
        <artifactId>spring-kafka</artifactId>
    </dependency>

    <!-- Cache -->
    <dependency>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-data-redis</artifactId>
    </dependency>

    <!-- Observability -->
    <dependency>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-actuator</artifactId>
    </dependency>
    <dependency>
        <groupId>io.micrometer</groupId>
        <artifactId>micrometer-registry-prometheus</artifactId>
    </dependency>
    <dependency>
        <groupId>io.opentelemetry</groupId>
        <artifactId>opentelemetry-api</artifactId>
    </dependency>

    <!-- Utilities -->
    <dependency>
        <groupId>org.projectlombok</groupId>
        <artifactId>lombok</artifactId>
    </dependency>
    <dependency>
        <groupId>org.mapstruct</groupId>
        <artifactId>mapstruct</artifactId>
    </dependency>

    <!-- Testing -->
    <dependency>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-test</artifactId>
        <scope>test</scope>
    </dependency>
    <dependency>
        <groupId>org.testcontainers</groupId>
        <artifactId>postgresql</artifactId>
        <scope>test</scope>
    </dependency>
</dependencies>
```

### Package structure
```
src/main/java/com/shopizer/{service}/
  api/
    controller/    REST controllers (@RestController)
    dto/           Request/Response DTOs
    mapper/        DTO ↔ Domain mappers (MapStruct)
  application/
    service/       Service interfaces + implementations (@Service)
    command/       Command handlers
    query/         Query handlers
  domain/
    model/         Aggregate roots, entities, value objects
    event/         Domain events
    repository/    Repository interfaces (no Spring Data annotations)
    service/       Domain services (pure business logic)
  infrastructure/
    persistence/   JPA repositories, entity mappers
    messaging/     Kafka producers/consumers, outbox relay
    cache/         Redis cache configuration
    client/        External service REST clients (WebClient/Feign)
  config/          Spring @Configuration classes
  Application.java
```

---

## Python FastAPI (Python 3.11+)

### requirements.txt
```
fastapi==0.104.1
uvicorn[standard]==0.24.0
pydantic==2.5.0
pydantic-settings==2.1.0
sqlalchemy==2.0.23
asyncpg==0.29.0
alembic==1.13.0
aiokafka==0.8.1
redis==5.0.1
python-jose[cryptography]==3.3.0
passlib[bcrypt]==1.7.4
prometheus-client==0.19.0
opentelemetry-api==1.21.0
opentelemetry-sdk==1.21.0
opentelemetry-instrumentation-fastapi==0.42b0
structlog==23.2.0
httpx==0.25.2
pytest==7.4.3
pytest-asyncio==0.21.3
testcontainers[postgres]==3.7.1
```

### Package structure
```
app/
  api/
    routes/        FastAPI routers
    schemas/       Pydantic request/response models
    dependencies/  FastAPI dependency injection
  application/
    services/      Service classes
    commands/      Command handlers
    queries/       Query handlers
  domain/
    models/        Dataclass aggregate roots and entities
    events/        Domain event dataclasses
    repositories/  Abstract repository interfaces
    services/      Domain service classes
  infrastructure/
    persistence/   SQLAlchemy models and repositories
    messaging/     Kafka producer/consumer, outbox
    cache/         Redis client and cache manager
    clients/       External HTTP clients (httpx)
  config/          Settings (pydantic-settings)
  main.py          FastAPI app factory
```

---

## Node.js Express (Node 20+, TypeScript)

### package.json
```json
{
  "dependencies": {
    "express": "^4.18.2",
    "typescript": "^5.3.3",
    "@types/express": "^4.17.21",
    "pg": "^8.11.3",
    "typeorm": "^0.3.19",
    "kafkajs": "^2.2.4",
    "ioredis": "^5.3.2",
    "jsonwebtoken": "^9.0.2",
    "bcrypt": "^5.1.1",
    "helmet": "^7.1.0",
    "cors": "^2.8.5",
    "winston": "^3.11.0",
    "prom-client": "^15.1.0",
    "express-validator": "^7.0.1",
    "dotenv": "^16.3.1",
    "axios": "^1.6.2"
  },
  "devDependencies": {
    "@types/node": "^20.10.6",
    "ts-node": "^10.9.2",
    "nodemon": "^3.0.2",
    "jest": "^29.7.0",
    "@types/jest": "^29.5.11",
    "testcontainers": "^10.2.1"
  }
}
```

### Package structure
```
src/
  api/
    controllers/   Express route handlers
    dto/           Request/response interfaces
    middleware/    Auth, correlation ID, validation middleware
    routes/        Express Router configuration
  application/
    services/      Service interfaces and implementations
    commands/      Command handler classes
    queries/       Query handler classes
  domain/
    models/        Aggregate root and entity classes
    events/        Domain event interfaces
    repositories/  Repository interfaces
    services/      Domain service classes
  infrastructure/
    persistence/   TypeORM entities and repositories
    messaging/     Kafka producer/consumer, outbox
    cache/         Redis client and cache
    clients/       Axios-based external service clients
  config/          Config loader (dotenv)
  index.ts         Express app factory and startup
```
