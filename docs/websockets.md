---
title: WebSockets
parent: Guides
nav_order: 8
---

Shaperail includes WebSocket session and broadcast primitives in the runtime,
but the scaffolded app does not automatically load channel YAML files or
register `/ws/...` routes.

## Current implementation status

Today the runtime gives you:

- channel/session primitives
- JWT-aware upgrade helpers
- room subscription support
- broadcast message types

What the scaffold does **not** do automatically:

- scan `channels/*.channel.yaml`
- create one route per channel
- start cross-instance broadcast plumbing for those channels

Treat channel YAMLs as a declaration format you can adopt in your own bootstrap,
not something the generated app wires for you by default.

## Channel definition format

The current channel file shape is:

```yaml
channel: notifications
auth: [member, admin]
rooms: true
hooks:
  on_connect: [log_connect]
  on_disconnect: [log_disconnect]
  on_message: [validate_message]
```

| Field | Type | Description |
| --- | --- | --- |
| `channel` | string | Channel name. |
| `auth` | string or list | Access rule for the connection. |
| `rooms` | bool | Whether room subscriptions are allowed. |
| `hooks` | object | Connection lifecycle hooks. |

## Typical route shape

If you wire WebSockets manually, a common route shape is:

```text
ws://<host>/ws/<channel>?token=<jwt>
```

The runtime primitives support JWT validation on upgrade and room-based
subscriptions, but the route itself must be registered by your application.

## Message shapes

The runtime session model uses JSON messages with an `action` field from the
client:

```json
{ "action": "subscribe", "room": "org:123" }
```

```json
{ "action": "unsubscribe", "room": "org:123" }
```

```json
{ "action": "message", "room": "org:123", "data": { "text": "hello" } }
```

Server messages use a `type` field, for example:

```json
{ "type": "subscribed", "room": "org:123" }
```

```json
{ "type": "broadcast", "room": "org:123", "event": "user.created", "data": { "id": "abc" } }
```

## Hooks and broadcasting

The runtime channel model supports:

- `on_connect`
- `on_disconnect`
- `on_message`

It also has broadcast primitives that can be used from event-driven code.

Current limitation: the default scaffold does not connect event subscriber
targets or Redis pub/sub to channel routes for you. If you want a full
real-time pipeline, you need to register routes and broadcast handlers
explicitly.
