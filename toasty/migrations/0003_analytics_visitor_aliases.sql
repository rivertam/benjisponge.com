CREATE TABLE "analytics_visitor_aliases" (
    "token_hash" TEXT NOT NULL,
    "visitor_id" TEXT NOT NULL,
    "created_at" BIGINT NOT NULL,
    PRIMARY KEY ("token_hash"),
    CONSTRAINT "analytics_visitor_aliases_token_digest"
        CHECK ("token_hash" ~ '^[0-9a-f]{64}$'),
    CONSTRAINT "analytics_visitor_aliases_visitor_digest"
        CHECK ("visitor_id" ~ '^[0-9a-f]{64}$'),
    CONSTRAINT "analytics_visitor_aliases_timestamp"
        CHECK ("created_at" BETWEEN 0 AND 253402300799)
);
