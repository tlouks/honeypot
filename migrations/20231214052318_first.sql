-- Add migration script here
CREATE TABLE stats (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address VARCHAR(255) NOT NULL UNIQUE,
    ping_count INT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
