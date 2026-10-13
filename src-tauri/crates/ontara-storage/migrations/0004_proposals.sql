-- Proposed changes and the author's policy for assistants (phase E).
--
-- A proposal is what an AI provider or an MCP client suggests. It lives apart from the model:
-- nothing in it is applied until the author approves it, change by change, and applying goes
-- through the same editor as any other edit. The changes are stored as the closed set of
-- operations `ProposedChange` defines; an unknown operation cannot be read back.

CREATE TABLE proposal (
    id         TEXT PRIMARY KEY,
    source     TEXT NOT NULL CHECK (source IN ('ai', 'mcp')),
    author     TEXT NOT NULL,
    title      TEXT NOT NULL,
    rationale  TEXT NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

CREATE TABLE proposal_change (
    proposal_id TEXT NOT NULL REFERENCES proposal (id) ON DELETE CASCADE,
    position    INTEGER NOT NULL CHECK (position >= 0),
    change      TEXT NOT NULL,
    status      TEXT NOT NULL CHECK (status IN ('pending', 'applied', 'rejected', 'failed')),
    error       TEXT,
    PRIMARY KEY (proposal_id, position)
) STRICT;

-- What assistants may reach in this project. Both start closed: a project shares nothing with a
-- provider on the internet, and offers nothing to MCP clients, until its author says so.
CREATE TABLE ai_policy (
    id            INTEGER PRIMARY KEY CHECK (id = 1),
    cloud_allowed INTEGER NOT NULL CHECK (cloud_allowed IN (0, 1)),
    mcp_allowed   INTEGER NOT NULL CHECK (mcp_allowed IN (0, 1))
) STRICT;

INSERT INTO ai_policy (id, cloud_allowed, mcp_allowed) VALUES (1, 0, 0);
