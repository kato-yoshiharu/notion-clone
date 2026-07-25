CREATE SCHEMA notion;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE notion.pages (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    title TEXT NOT NULL,
    text TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE notion.page_relationships (
    ancestor UUID NOT NULL REFERENCES notion.pages(id) ON DELETE CASCADE,
    descendant UUID NOT NULL REFERENCES notion.pages(id) ON DELETE CASCADE,
    weight INTEGER NOT NULL CHECK (weight >= 0),
    PRIMARY KEY (ancestor, descendant)
);

CREATE TABLE notion.page_sibling_relationships (
    ancestor UUID NOT NULL REFERENCES notion.pages(id) ON DELETE CASCADE,
    descendant UUID NOT NULL REFERENCES notion.pages(id) ON DELETE CASCADE,
    weight INTEGER NOT NULL CHECK (weight >= 0),
    PRIMARY KEY (ancestor, descendant)
);
