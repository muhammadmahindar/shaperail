---
title: Migrations and schema changes
parent: Guides
nav_order: 4
---

Shaperail treats resource YAML as the schema source of truth, but the running
database still changes through SQL files in `migrations/`.

## Starting state

`shaperail init` creates:

- a starter resource
- an initial SQL migration

That means a new project should be able to boot with:

```bash
docker compose up -d
shaperail serve
```

without writing SQL by hand first.

## Workflow when a resource changes

1. Edit `resources/*.yaml`
2. Validate the resource file
3. Create a new migration
4. Review the generated SQL
5. Run the app

Commands:

```bash
shaperail validate resources/posts.yaml
shaperail migrate
shaperail serve
```

## Important distinction

- `shaperail migrate` creates new SQL migration files
- `shaperail serve` applies the SQL files already present in `migrations/`

## Step-by-step migration workflow

### 1. Make the schema change

Edit the resource YAML. For example, add a `bio` field to `resources/users.yaml`:

```yaml
schema:
  # existing fields ...
  bio: { type: string, max: 2000 }
```

### 2. Validate

```bash
shaperail validate resources/users.yaml
```

If the file has errors (unknown fields, bad types), the validator rejects it
immediately with an explicit message.

### 3. Generate the migration

```bash
shaperail migrate
```

This diffs the current resource YAML against the existing migrations and writes
a new SQL file into `migrations/`. The file is named sequentially, for example:

```
migrations/0003_alter_users_add_bio.sql
```

### 4. Review the SQL

Open the generated file and check:

- Column type is correct
- `NOT NULL` constraints match your intent
- Default values are sensible
- Foreign keys and indexes are present where expected

### 5. Apply and run

```bash
shaperail serve
```

On startup, Shaperail applies any unapplied migrations in order.

## Concrete migration examples

### Add a column

Resource change:

```yaml
schema:
  # ...existing fields...
  avatar_url: { type: string, max: 500 }
```

Generated SQL:

```sql
ALTER TABLE users ADD COLUMN avatar_url VARCHAR(500);
```

### Add a required column with a default for existing rows

If you add a required column to a table that already has data, you must provide
a default so existing rows are valid:

```yaml
schema:
  status: { type: enum, values: [active, inactive], default: active, required: true }
```

Generated SQL:

```sql
ALTER TABLE users ADD COLUMN status VARCHAR(20) NOT NULL DEFAULT 'active';
-- CHECK constraint for enum values
ALTER TABLE users ADD CONSTRAINT chk_users_status
  CHECK (status IN ('active', 'inactive'));
```

Review carefully: the `DEFAULT` value is applied to all existing rows. If you
need different values for existing rows, write a data migration manually.

### Drop a column

Remove the field from the YAML. The generated migration adds:

```sql
ALTER TABLE users DROP COLUMN IF EXISTS legacy_field;
```

Always check that no other resource references the dropped column through
`ref:` before removing it.

### Rename a column

Shaperail does not auto-detect renames (a remove + add looks the same as a
rename). To rename a column:

1. Remove the old field from YAML
2. Add the new field to YAML
3. Run `shaperail migrate`
4. Replace the generated `DROP` + `ADD` SQL with:

```sql
ALTER TABLE users RENAME COLUMN full_name TO display_name;
```

5. Also add any constraint updates if the column type changed

### Change a column type

Change the `type` in the YAML:

```yaml
# Before
  score: { type: integer }
# After
  score: { type: float }
```

Generated SQL:

```sql
ALTER TABLE items ALTER COLUMN score TYPE DOUBLE PRECISION USING score::DOUBLE PRECISION;
```

Review the `USING` clause. Postgres requires an explicit cast when changing
types. If the cast can fail (e.g., string to integer), write the migration
manually with error handling.

### Add an index

Add an entry to the `indexes` section:

```yaml
indexes:
  - { fields: [org_id, role] }
  - { fields: [email], unique: true }
```

Generated SQL:

```sql
CREATE INDEX idx_users_org_id_role ON users (org_id, role);
CREATE UNIQUE INDEX idx_users_email ON users (email);
```

### Add a composite unique constraint

```yaml
indexes:
  - { fields: [org_id, name], unique: true }
```

Generated SQL:

```sql
CREATE UNIQUE INDEX idx_projects_org_id_name ON projects (org_id, name);
```

## Testing migrations before applying

### Dry run against a throwaway database

```bash
# Start a fresh Postgres container
docker run --rm -d --name pg-test -e POSTGRES_PASSWORD=test -p 5499:5432 postgres:16

# Point to the test database
export DATABASE_URL=postgresql://postgres:test@localhost:5499/postgres

# Apply all migrations
shaperail serve --migrate-only

# If it succeeds, tear down
docker stop pg-test
```

### Validate SQL syntax without applying

Open each `.sql` file and run it through `psql` in a transaction that rolls
back:

```bash
psql "$DATABASE_URL" -c "BEGIN; $(cat migrations/0003_alter_users_add_bio.sql); ROLLBACK;"
```

If the SQL has syntax errors, psql reports them without changing data.

### Check for destructive changes

Before committing a migration, search for:

- `DROP COLUMN` -- data loss
- `DROP TABLE` -- data loss
- `ALTER COLUMN ... TYPE` -- potential data truncation
- `NOT NULL` without `DEFAULT` -- fails if rows exist

If any of these appear, add a comment explaining why the change is safe or
write a two-step migration (see zero-downtime patterns below).

## Rollback strategies and safeguards

### Roll back a recent migration batch

```bash
shaperail migrate --rollback
```

Use this for local recovery if the latest migration batch needs to be reversed.

### Manual rollback

If `--rollback` does not cover your case, write a reverse migration:

```sql
-- migrations/0004_revert_add_bio.sql
ALTER TABLE users DROP COLUMN IF EXISTS bio;
```

### Safeguards

1. **Always review generated SQL before committing.** Generated SQL should not
   be treated as invisible build output.
2. **Back up before applying migrations in production.** Use `pg_dump` or your
   managed database's snapshot feature.
3. **Never delete applied migration files.** The migration table tracks which
   files have been applied. Deleting a file causes drift.
4. **Test migrations on a copy of production data.** Restore a recent backup
   into a staging database and run migrations there first.

## Zero-downtime migration patterns

When deploying to production with no downtime, some changes require a multi-step
approach.

### Adding a required column (two-step)

**Step 1** -- deploy a migration that adds the column as nullable with a default:

```sql
ALTER TABLE users ADD COLUMN status VARCHAR(20) DEFAULT 'active';
```

**Step 2** -- after backfilling existing rows, deploy a second migration:

```sql
ALTER TABLE users ALTER COLUMN status SET NOT NULL;
```

### Renaming a column (three-step)

**Step 1** -- add the new column, deploy code that writes to both:

```sql
ALTER TABLE users ADD COLUMN display_name VARCHAR(200);
UPDATE users SET display_name = full_name;
```

**Step 2** -- deploy code that reads from the new column. Backfill any remaining
rows.

**Step 3** -- drop the old column:

```sql
ALTER TABLE users DROP COLUMN full_name;
```

### Adding an index without locking

For large tables, use `CONCURRENTLY` to avoid blocking writes:

```sql
CREATE INDEX CONCURRENTLY idx_users_email ON users (email);
```

Note: `CONCURRENTLY` cannot run inside a transaction. If your migration runner
wraps each file in a transaction, put this in its own migration file and mark
it accordingly.

### Dropping a column (two-step)

**Step 1** -- deploy code that no longer reads or writes the column.

**Step 2** -- deploy the migration that drops the column:

```sql
ALTER TABLE users DROP COLUMN IF EXISTS legacy_field;
```

## Handling existing data when adding required columns

When you add a `required: true` field to a resource that already has rows in the
database, the migration must handle existing data. There are three approaches:

### 1. Provide a default in the schema

```yaml
status: { type: enum, values: [active, inactive], default: active, required: true }
```

The generated SQL uses `DEFAULT` so existing rows get the value automatically.

### 2. Backfill manually before adding the constraint

Edit the generated migration to split it into two statements:

```sql
-- Add column as nullable first
ALTER TABLE users ADD COLUMN department VARCHAR(100);

-- Backfill existing rows
UPDATE users SET department = 'general' WHERE department IS NULL;

-- Now add the NOT NULL constraint
ALTER TABLE users ALTER COLUMN department SET NOT NULL;
```

### 3. Compute the value from existing data

```sql
ALTER TABLE users ADD COLUMN full_name VARCHAR(400);
UPDATE users SET full_name = first_name || ' ' || last_name;
ALTER TABLE users ALTER COLUMN full_name SET NOT NULL;
```

After backfilling, you can optionally drop the old columns in a separate
migration.

## Review the SQL before commit

Generated SQL should not be treated as invisible build output. Check:

- table names
- `NOT NULL` constraints
- enum checks
- foreign keys
- indexes
- whether a delete route should be hard delete or soft delete

## Tooling note

Today, `shaperail migrate` relies on `sqlx-cli`:

```bash
cargo install sqlx-cli
```

## Example flow

The [Blog API example]({{ '/blog-api-example/' | relative_url }}) includes two checked-in
migrations that match its resource files, so you can inspect the schema-to-SQL
relationship directly.
