CREATE TABLE "analytics_identities" (
    "visitor_id" TEXT NOT NULL,
    "display_name" TEXT NOT NULL,
    "note" TEXT,
    "first_submitted_at" BIGINT NOT NULL,
    "updated_at" BIGINT NOT NULL,
    PRIMARY KEY ("visitor_id"),
    CONSTRAINT "analytics_identities_visitor_digest"
        CHECK ("visitor_id" ~ '^[0-9a-f]{64}$'),
    CONSTRAINT "analytics_identities_display_name"
        CHECK (
            char_length("display_name") BETWEEN 1 AND 80
            AND "display_name" = btrim("display_name")
            AND "display_name" !~ '[[:cntrl:]]'
        ),
    CONSTRAINT "analytics_identities_note"
        CHECK (
            "note" IS NULL
            OR (
                char_length("note") BETWEEN 1 AND 400
                AND "note" = btrim("note")
                AND "note" !~ '[[:cntrl:]]'
            )
        ),
    CONSTRAINT "analytics_identities_timestamps"
        CHECK (
            "first_submitted_at" BETWEEN 0 AND 253402300799
            AND "updated_at" >= "first_submitted_at"
            AND "updated_at" <= 253402300799
        )
);
CREATE TABLE "analytics_events" (
    "id" TEXT NOT NULL,
    "visitor_id" TEXT NOT NULL,
    "session_id" TEXT NOT NULL,
    "occurred_at" BIGINT NOT NULL,
    "kind" TEXT NOT NULL,
    "page_path" TEXT NOT NULL,
    "referrer_kind" TEXT NOT NULL,
    "referrer_host" TEXT,
    "referrer_path" TEXT,
    "country_code" TEXT,
    "timezone" TEXT,
    "language" TEXT,
    "device_kind" TEXT NOT NULL,
    "browser" TEXT NOT NULL,
    "operating_system" TEXT NOT NULL,
    "viewport_kind" TEXT NOT NULL,
    "navigation_kind" TEXT,
    "local_hour" BIGINT,
    "local_weekday" BIGINT,
    "engagement_seconds" BIGINT,
    "scroll_percent" BIGINT,
    "lcp_milliseconds" BIGINT,
    "cls_thousandths" BIGINT,
    "navigation_milliseconds" BIGINT,
    "target_host" TEXT,
    "utm_source" TEXT,
    "utm_medium" TEXT,
    "utm_campaign" TEXT,
    PRIMARY KEY ("id"),
    CONSTRAINT "analytics_events_uuid"
        CHECK (
            "id" ~
                '^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$'
        ),
    CONSTRAINT "analytics_events_digests"
        CHECK (
            "visitor_id" ~ '^[0-9a-f]{64}$'
            AND "session_id" ~ '^[0-9a-f]{64}$'
        ),
    CONSTRAINT "analytics_events_time"
        CHECK ("occurred_at" BETWEEN 0 AND 253402300799),
    CONSTRAINT "analytics_events_kind"
        CHECK ("kind" IN ('pageview', 'engagement', 'outbound')),
    CONSTRAINT "analytics_events_page_path"
        CHECK (
            octet_length("page_path") BETWEEN 1 AND 300
            AND left("page_path", 1) = '/'
            AND position('?' IN "page_path") = 0
            AND position('#' IN "page_path") = 0
            AND position(chr(92) IN "page_path") = 0
            AND substring("page_path" FROM 2 FOR 1) <> '/'
            AND "page_path" !~ '[[:cntrl:]]'
            AND "page_path" NOT LIKE '/api/%'
            AND "page_path" NOT LIKE '/_topcoat/%'
        ),
    CONSTRAINT "analytics_events_referrer_kind"
        CHECK (
            "referrer_kind" IN
                ('direct', 'internal', 'search', 'social', 'ai', 'referral')
        ),
    CONSTRAINT "analytics_events_referrer_shape"
        CHECK (
            (
                "referrer_kind" = 'direct'
                AND "referrer_host" IS NULL
                AND "referrer_path" IS NULL
            )
            OR (
                "referrer_kind" = 'internal'
                AND "referrer_host" IS NULL
            )
            OR (
                "referrer_kind" IN ('search', 'social', 'ai', 'referral')
                AND "referrer_host" IS NOT NULL
                AND "referrer_path" IS NULL
            )
        ),
    CONSTRAINT "analytics_events_referrer_host"
        CHECK (
            "referrer_host" IS NULL
            OR (
                octet_length("referrer_host") BETWEEN 1 AND 253
                AND "referrer_host" = lower("referrer_host")
                AND "referrer_host" !~ '[[:space:]/?#]'
            )
        ),
    CONSTRAINT "analytics_events_referrer_path"
        CHECK (
            "referrer_path" IS NULL
            OR (
                octet_length("referrer_path") BETWEEN 1 AND 300
                AND left("referrer_path", 1) = '/'
                AND position('?' IN "referrer_path") = 0
                AND position('#' IN "referrer_path") = 0
                AND position(chr(92) IN "referrer_path") = 0
                AND substring("referrer_path" FROM 2 FOR 1) <> '/'
                AND "referrer_path" !~ '[[:cntrl:]]'
            )
        ),
    CONSTRAINT "analytics_events_country"
        CHECK (
            "country_code" IS NULL
            OR "country_code" ~ '^[A-Z]{2}$'
        ),
    CONSTRAINT "analytics_events_timezone"
        CHECK (
            "timezone" IS NULL
            OR (
                octet_length("timezone") BETWEEN 1 AND 64
                AND "timezone" ~ '^[A-Za-z0-9_/+-]+$'
            )
        ),
    CONSTRAINT "analytics_events_language"
        CHECK (
            "language" IS NULL
            OR (
                octet_length("language") BETWEEN 1 AND 24
                AND "language" ~ '^[A-Za-z0-9-]+$'
            )
        ),
    CONSTRAINT "analytics_events_device"
        CHECK (
            "device_kind" IN ('desktop', 'mobile', 'tablet', 'unknown')
        ),
    CONSTRAINT "analytics_events_browser"
        CHECK (
            "browser" IN ('Edge', 'Firefox', 'Opera', 'Chrome', 'Safari', 'Other')
        ),
    CONSTRAINT "analytics_events_operating_system"
        CHECK (
            "operating_system" IN
                ('iOS', 'Android', 'Windows', 'macOS', 'Linux', 'Other')
        ),
    CONSTRAINT "analytics_events_viewport"
        CHECK (
            "viewport_kind" IN
                ('phone', 'tablet', 'desktop', 'wide', 'unknown')
        ),
    CONSTRAINT "analytics_events_navigation"
        CHECK (
            "navigation_kind" IS NULL
            OR "navigation_kind" IN
                ('navigate', 'reload', 'back_forward', 'prerender')
        ),
    CONSTRAINT "analytics_events_local_time"
        CHECK (
            ("local_hour" IS NULL) = ("local_weekday" IS NULL)
            AND ("local_hour" IS NULL OR "local_hour" BETWEEN 0 AND 23)
            AND (
                "local_weekday" IS NULL
                OR "local_weekday" BETWEEN 0 AND 6
            )
        ),
    CONSTRAINT "analytics_events_metrics"
        CHECK (
            (
                "engagement_seconds" IS NULL
                OR "engagement_seconds" BETWEEN 0 AND 7200
            )
            AND (
                "scroll_percent" IS NULL
                OR "scroll_percent" BETWEEN 0 AND 100
            )
            AND (
                "lcp_milliseconds" IS NULL
                OR "lcp_milliseconds" BETWEEN 0 AND 120000
            )
            AND (
                "cls_thousandths" IS NULL
                OR "cls_thousandths" BETWEEN 0 AND 100000
            )
            AND (
                "navigation_milliseconds" IS NULL
                OR "navigation_milliseconds" BETWEEN 0 AND 120000
            )
        ),
    CONSTRAINT "analytics_events_target_host"
        CHECK (
            (
                "kind" = 'outbound'
                AND "target_host" IS NOT NULL
            )
            OR (
                "kind" <> 'outbound'
                AND "target_host" IS NULL
            )
        ),
    CONSTRAINT "analytics_events_target_host_format"
        CHECK (
            "target_host" IS NULL
            OR (
                octet_length("target_host") BETWEEN 1 AND 253
                AND "target_host" = lower("target_host")
                AND "target_host" !~ '[[:space:]/?#]'
            )
        ),
    CONSTRAINT "analytics_events_campaigns"
        CHECK (
            (
                "utm_source" IS NULL
                OR (
                    char_length("utm_source") BETWEEN 1 AND 80
                    AND "utm_source" !~ '[[:cntrl:]]'
                )
            )
            AND (
                "utm_medium" IS NULL
                OR (
                    char_length("utm_medium") BETWEEN 1 AND 80
                    AND "utm_medium" !~ '[[:cntrl:]]'
                )
            )
            AND (
                "utm_campaign" IS NULL
                OR (
                    char_length("utm_campaign") BETWEEN 1 AND 80
                    AND "utm_campaign" !~ '[[:cntrl:]]'
                )
            )
        )
);
CREATE INDEX "index_analytics_events_by_kind_and_occurred_at" ON "analytics_events" ("kind", "occurred_at");
CREATE INDEX "index_analytics_events_by_occurred_at" ON "analytics_events" ("occurred_at");
CREATE INDEX "index_analytics_events_by_session" ON "analytics_events" ("visitor_id", "session_id", "occurred_at", "id") WHERE "kind" = 'pageview';
CREATE INDEX "index_analytics_events_page_country" ON "analytics_events" ("occurred_at", "country_code", "visitor_id") WHERE "kind" = 'pageview' AND "country_code" IS NOT NULL;
CREATE INDEX "index_analytics_events_page_referrer" ON "analytics_events" ("occurred_at", "referrer_kind", "referrer_host", "visitor_id") WHERE "kind" = 'pageview';
CREATE INDEX "index_analytics_events_outbound_target" ON "analytics_events" ("occurred_at", "target_host", "visitor_id") WHERE "kind" = 'outbound' AND "target_host" IS NOT NULL;
