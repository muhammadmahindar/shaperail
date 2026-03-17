CREATE TABLE IF NOT EXISTS shaperail_event_log (
    event_id TEXT PRIMARY KEY,
    event TEXT NOT NULL,
    resource TEXT NOT NULL,
    action TEXT NOT NULL,
    data JSONB NOT NULL,
    timestamp TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_shaperail_event_log_resource_timestamp
    ON shaperail_event_log (resource, timestamp DESC);

CREATE TABLE IF NOT EXISTS shaperail_webhook_delivery_log (
    delivery_id TEXT PRIMARY KEY,
    event_id TEXT NOT NULL,
    url TEXT NOT NULL,
    status_code INTEGER NOT NULL,
    status TEXT NOT NULL,
    latency_ms BIGINT NOT NULL,
    error TEXT,
    attempt INTEGER NOT NULL,
    timestamp TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_shaperail_webhook_delivery_log_event_timestamp
    ON shaperail_webhook_delivery_log (event_id, timestamp DESC);
