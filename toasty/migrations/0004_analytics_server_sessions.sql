CREATE TABLE "analytics_sessions" (
    "visitor_id" TEXT NOT NULL,
    "session_id" TEXT NOT NULL,
    "last_seen_at" BIGINT NOT NULL,
    PRIMARY KEY ("visitor_id"),
    CONSTRAINT "analytics_sessions_digests"
        CHECK (
            "visitor_id" ~ '^[0-9a-f]{64}$'
            AND "session_id" ~ '^[0-9a-f]{64}$'
        ),
    CONSTRAINT "analytics_sessions_timestamp"
        CHECK ("last_seen_at" BETWEEN 0 AND 253402300799)
);

INSERT INTO "analytics_sessions" ("visitor_id", "session_id", "last_seen_at")
SELECT DISTINCT ON ("visitor_id")
       "visitor_id", "session_id", "occurred_at"
FROM "analytics_events"
ORDER BY "visitor_id", "occurred_at" DESC, "id" DESC;
