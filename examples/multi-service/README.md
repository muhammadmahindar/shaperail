# Multi-service workspace example

Demonstrates a Shaperail workspace with two services, cross-service
controllers, and a distributed saga.

## Structure

```
ecommerce/
├── shaperail.workspace.yaml    # Workspace definition
├── sagas/
│   └── create_order.saga.yaml  # Distributed saga
└── services/
    ├── users-api/
    │   └── resources/
    │       ├── users.yaml              # Resource definition
    │       └── users.controller.rs     # User controllers
    └── orders-api/
        └── resources/
            ├── orders.yaml             # Resource definition
            └── orders.controller.rs    # Order controllers
```

## Running

```bash
docker compose up -d            # Start shared Postgres + Redis
shaperail serve --workspace     # Start all services in dependency order
```

The `orders-api` depends on `users-api`, so the users service starts first.

## Service registry

Both services register in Redis on startup. Use `redis-cli` to inspect:

```bash
redis-cli KEYS "shaperail:services:*"
redis-cli GET "shaperail:services:users-api"
```

## Controllers

Controllers add custom business logic that runs before or after the generated
CRUD handler. Each controller function receives a mutable `Context` and returns
a `ControllerResult`. See `agent_docs/hooks-system.md` for the full API.

### users-api controllers

| Function                | Phase  | Endpoint | Purpose |
|-------------------------|--------|----------|---------|
| `prepare_user`          | before | create   | Normalize email to lowercase, hash password with bcrypt, reject blocked email domains, set default role to `member` |
| `provision_defaults`    | after  | create   | Log new user creation with request ID, add `X-User-Created` response header |
| `validate_user_update`  | before | update   | Non-admins cannot change roles; cannot demote the last admin in the system |

### orders-api controllers

| Function                | Phase  | Endpoint | Purpose |
|-------------------------|--------|----------|---------|
| `validate_order`        | before | create   | Cross-service user validation via DB query, enforce positive total, override status to `pending`, generate `order_number`, auto-fill `created_by` from JWT |
| `enforce_order_status`  | before | update   | Enforce the order state machine (see below), restrict `shipped`/`delivered` to admins, flag refunds on paid-order cancellations |

### Cross-service validation

The `validate_order` controller queries the `users` table to confirm the
referenced `user_id` exists before allowing order creation. In this example
both services share a Postgres instance, so validation is a direct SQL query.
In a split-database deployment, replace the query with a typed HTTP client call
to the users-api service.

### Order status state machine

```
pending ──> paid ──> shipped ──> delivered
  │           │
  └──> cancelled <──┘
```

Rules enforced by `enforce_order_status`:

- **pending** can transition to `paid` or `cancelled`.
- **paid** can transition to `shipped` (admin only) or `cancelled` (triggers refund flag).
- **shipped** can transition to `delivered` (admin only).
- **cancelled** and **delivered** are terminal states; no further updates allowed.
- When a paid order is cancelled and `total > 0`, the controller sets
  `refund_required: true` and `refund_amount` on the input for downstream
  processing.

## Saga

The `create_order` saga validates the user exists before creating an order.
If order creation fails, no compensating action is needed for the read-only
user validation step.
