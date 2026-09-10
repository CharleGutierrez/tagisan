---
name: drizzle-orm
description: Use when the user mentions Drizzle ORM, drizzle-kit, database schema with pgTable/mysqlTable/sqliteTable,
  Drizzle migrations, Drizzle queries, Drizzle relations, or drizzle.config.ts. Also
  trigger when writing TypeScript ORM code with eq/and/or/sql operators from drizzle-orm.
...
---
# Drizzle ORM Skill

Help users define schemas, write queries, run migrations, and configure Drizzle ORM projects.

## When to Fetch Live Docs

Use `WebFetch` against `https://orm.drizzle.team/docs/...` when:
- User asks about MySQL or SQLite column types (inline covers PostgreSQL)
- Advanced relation queries or prepared statements
- Drizzle with specific drivers (Neon, PlanetScale, Turso, D1)
- Latest drizzle-kit flags or new features

Useful doc URLs:
- Get started: `https://orm.drizzle.team/docs/get-started`
- PostgreSQL columns: `https://orm.drizzle.team/docs/column-types/pg`
- MySQL columns: `https://orm.drizzle.team/docs/column-types/mysql`
- SQLite columns: `https://orm.drizzle.team/docs/column-types/sqlite`
- Select: `https://orm.drizzle.team/docs/select`
- Insert: `https://orm.drizzle.team/docs/insert`
- Update: `https://orm.drizzle.team/docs/update`
- Delete: `https://orm.drizzle.team/docs/delete`
- Joins: `https://orm.drizzle.team/docs/joins`
- Relations: `https://orm.drizzle.team/docs/relations`
- Filters & operators: `https://orm.drizzle.team/docs/operators`
- Transactions: `https://orm.drizzle.team/docs/transactions`
- Migrations: `https://orm.drizzle.team/docs/migrations`
- drizzle-kit overview: `https://orm.drizzle.team/docs/kit-overview`
- drizzle.config.ts: `https://orm.drizzle.team/docs/drizzle-config-file`
- Prepared statements: `https://orm.drizzle.team/docs/perf-queries`

---

## Schema Declaration (PostgreSQL)

```ts
import { pgTable, serial, text, integer, boolean, timestamp, uuid, json, varchar } from 'drizzle-orm/pg-core';

export const users = pgTable('users', {
  id: serial('id').primaryKey(),
  name: text('name').notNull(),
  email: varchar('email', { length: 255 }).unique().notNull(),
  age: integer('age'),
  active: boolean('active').default(true),
  metadata: json('metadata').$type<{ role: string }>(),
  createdAt: timestamp('created_at').defaultNow().notNull(),
});

export const posts = pgTable('posts', {
  id: uuid('id').defaultRandom().primaryKey(),
  title: text('title').notNull(),
  content: text('content'),
  authorId: integer('author_id').references(() => users.id).notNull(),
  publishedAt: timestamp('published_at'),
});
```

### PostgreSQL Column Types

| Type | Import | Notes |
|---|---|---|
| `integer` / `int4` | `integer()` | 4-byte signed |
| `smallint` / `int2` | `smallint()` | 2-byte signed |
| `bigint` / `int8` | `bigint()` | 8-byte, `mode: 'number' \| 'bigint'` |
| `serial` | `serial()` | Auto-increment 4-byte |
| `bigserial` | `bigserial()` | Auto-increment 8-byte |
| `numeric` / `decimal` | `numeric()` | Exact, configurable precision |
| `real` / `float4` | `real()` | 4-byte float |
| `double precision` | `doublePrecision()` | 8-byte float |
| `text` | `text()` | Unlimited string |
| `varchar(n)` | `varchar({ length: n })` | Variable with limit |
| `char(n)` | `char({ length: n })` | Fixed-length |
| `boolean` | `boolean()` | true/false |
| `json` | `json()` | Text JSON |
| `jsonb` | `jsonb()` | Binary JSON |
| `uuid` | `uuid()` | Use `.defaultRandom()` |
| `timestamp` | `timestamp()` | `mode: 'date' \| 'string'`, `{ withTimezone: true }` |
| `date` | `date()` | Calendar date |
| `time` | `time()` | Time of day |
| `interval` | `interval()` | Time span |
| `bytea` | `bytea()` | Binary data |

### Column Modifiers

`.notNull()`, `.primaryKey()`, `.default(value)`, `.defaultNow()`, `.defaultRandom()`, `.$defaultFn(() => ...)`, `.$onUpdateFn(() => ...)`, `.unique()`, `.references(() => table.col)`, `.$type<T>()`

### Enums

```ts
import { pgEnum } from 'drizzle-orm/pg-core';

export const statusEnum = pgEnum('status', ['active', 'inactive', 'pending']);

// Use in table:
status: statusEnum('status').default('active'),
```

### Indexes

```ts
import { pgTable, index, uniqueIndex } from 'drizzle-orm/pg-core';

export const users = pgTable('users', {
  id: serial('id').primaryKey(),
  email: text('email').notNull(),
}, (table) => [
  index('email_idx').on(table.email),
  uniqueIndex('email_unique_idx').on(table.email),
]);
```

---

## Queries

### Connection Setup

```ts
import { drizzle } from 'drizzle-orm/node-postgres';
import { Pool } from 'pg';

// Standard
const pool = new Pool({ connectionString: process.env.DATABASE_URL });
const db = drizzle(pool);

// Serverless (max 1 connection, no prepared statements)
const pool = new Pool({
  connectionString: process.env.DATABASE_URL,
  max: 1,
});
const db = drizzle(pool);
```

### Select

```ts
import { eq, ne, gt, gte, lt, lte, like, ilike, and, or, not, inArray, isNull, between, sql, asc, desc } from 'drizzle-orm';

// All rows
const allUsers = await db.select().from(users);

// Partial select
const names = await db.select({ id: users.id, name: users.name }).from(users);

// Where
await db.select().from(users).where(eq(users.id, 42));
await db.select().from(users).where(and(gt(users.age, 18), eq(users.active, true)));
await db.select().from(users).where(or(eq(users.name, 'Alice'), eq(users.name, 'Bob')));
await db.select().from(users).where(like(users.email, '%@gmail.com'));
await db.select().from(users).where(inArray(users.id, [1, 2, 3]));
await db.select().from(users).where(isNull(users.age));
await db.select().from(users).where(between(users.age, 18, 65));

// Order, limit, offset
await db.select().from(users).orderBy(desc(users.createdAt)).limit(10).offset(20);

// Count / aggregation
await db.select({ count: sql<number>`cast(count(*) as int)` }).from(users);

// Group by + having
await db.select({
  age: users.age,
  count: sql<number>`cast(count(*) as int)`,
}).from(users).groupBy(users.age).having(({ count }) => gt(count, 1));

// Distinct
await db.selectDistinct().from(users);
```

### Insert

```ts
// Single
await db.insert(users).values({ name: 'Alice', email: 'alice@example.com' });

// Multiple
await db.insert(users).values([
  { name: 'Alice', email: 'alice@example.com' },
  { name: 'Bob', email: 'bob@example.com' },
]);

// Returning
const [newUser] = await db.insert(users).values({ name: 'Alice', email: 'alice@example.com' }).returning();

// On conflict (upsert)
await db.insert(users).values({ id: 1, name: 'Alice', email: 'alice@example.com' })
  .onConflictDoUpdate({ target: users.email, set: { name: 'Alice Updated' } });

await db.insert(users).values({ name: 'Alice', email: 'alice@example.com' })
  .onConflictDoNothing();
```

### Update

```ts
await db.update(users).set({ name: 'Bob' }).where(eq(users.id, 1));

// Returning
const [updated] = await db.update(users).set({ active: false }).where(eq(users.id, 1)).returning();
```

### Delete

```ts
await db.delete(users).where(eq(users.id, 1));

// Returning
const [deleted] = await db.delete(users).where(eq(users.id, 1)).returning();
```

---

## Joins

```ts
// Inner join
await db.select().from(users)
  .innerJoin(posts, eq(users.id, posts.authorId));

// Left join
await db.select().from(users)
  .leftJoin(posts, eq(users.id, posts.authorId));

// Right join
await db.select().from(users)
  .rightJoin(posts, eq(users.id, posts.authorId));

// Full join
await db.select().from(users)
  .fullJoin(posts, eq(users.id, posts.authorId));

// With aliases
import { alias } from 'drizzle-orm/pg-core';
const authors = alias(users, 'authors');
await db.select().from(posts).innerJoin(authors, eq(posts.authorId, authors.id));
```

---

## Relations (Query API)

```ts
import { relations } from 'drizzle-orm';

export const usersRelations = relations(users, ({ many }) => ({
  posts: many(posts),
}));

export const postsRelations = relations(posts, ({ one }) => ({
  author: one(users, {
    fields: [posts.authorId],
    references: [users.id],
  }),
}));

// Query with relations (requires passing schema to drizzle())
import * as schema from './schema';
const db = drizzle(pool, { schema });

const usersWithPosts = await db.query.users.findMany({
  with: { posts: true },
});

const post = await db.query.posts.findFirst({
  where: eq(posts.id, 1),
  with: { author: true },
});
```

---

## Transactions

```ts
await db.transaction(async (tx) => {
  const [user] = await tx.insert(users).values({ name: 'Alice', email: 'a@b.com' }).returning();
  await tx.insert(posts).values({ title: 'First Post', authorId: user.id });
});

// With rollback
await db.transaction(async (tx) => {
  await tx.insert(users).values({ name: 'test', email: 't@t.com' });
  tx.rollback(); // throws, rolls back
});
```

---

## drizzle-kit CLI

```bash
# Generate SQL migration from schema changes
npx drizzle-kit generate

# Push schema directly to DB (dev, no migration files)
npx drizzle-kit push

# Run pending migrations
npx drizzle-kit migrate

# Introspect DB and generate schema file
npx drizzle-kit pull

# Visual DB browser
npx drizzle-kit studio

# Validate migrations for conflicts
npx drizzle-kit check

# Use specific config
npx drizzle-kit push --config=drizzle-prod.config.ts
```

## drizzle.config.ts

```ts
import { defineConfig } from 'drizzle-kit';

export default defineConfig({
  dialect: 'postgresql',        // 'postgresql' | 'mysql' | 'sqlite'
  schema: './src/schema.ts',    // path to schema file(s)
  out: './drizzle',             // migrations output directory
  dbCredentials: {
    url: process.env.DATABASE_URL!,
  },
  verbose: true,                // log SQL during push/generate
  strict: true,                 // prompt before destructive changes
});
```

---

## Serverless Best Practices

```ts
// Use pooled/transaction mode connection string
// Max 1 connection per invocation
// Disable prepared statements for Supabase/Neon transaction mode
import { drizzle } from 'drizzle-orm/node-postgres';
import { Pool } from 'pg';

const pool = new Pool({
  connectionString: process.env.DATABASE_URL,
  max: 1,
  // For Supabase transaction mode pooler:
  // Add ?pgbouncer=true to connection string
});

const db = drizzle(pool);
```

### Conditional Where Clauses

```ts
function getUsers(filters: { name?: string; minAge?: number }) {
  const conditions = [];
  if (filters.name) conditions.push(eq(users.name, filters.name));
  if (filters.minAge) conditions.push(gte(users.age, filters.minAge));

  return db.select().from(users).where(
    conditions.length ? and(...conditions) : undefined
  );
}
```
