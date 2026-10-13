# 0009. Assistants only propose; the author decides

Status: accepted

## Context

Phase E lets models help maintain a universe: through providers the author configures with their
own keys, and through MCP clients on the same computer. The core of the application is offline,
local and the author's. Assistance must add to it without becoming a way for data to leave, for
canon to be rewritten, or for the model to be changed by anything but the author.

## Decision

**Assistants never write; they propose.** An answer from a provider, or a call from an MCP client,
becomes a *proposal*: a list of changes drawn from a closed set (`create_entity`, `update_entity`,
`add_relationship`, `add_note`). A proposal lives in its own tables, apart from the model. An
operation outside the set cannot be read; a field outside an operation (a name, a canon state) is
ignored, never applied. Changes that point at an entity that does not exist, a type that is not
defined, or a name nothing creates are dropped before the author sees them.

**Approving is an ordinary edit.** The author approves or rejects each change. Approved changes go
through the model editor, in a savepoint each: one the model refuses is marked failed with the
reason and leaves nothing behind, the rest still apply. The statuses and the changes are one
change set, so undo takes both back. What an assistant creates carries its origin (`ai` or `mcp`)
and the provisional canon state, which the consistency checks do not hold to canon.

**Nothing leaves without permission, and permission is a project's.** A project starts with
sending text to providers over the internet off, and MCP access off. A provider on this computer
(`localhost`, `127.0.0.1`, `[::1]`, judged from the address) needs no permission. Before any
request the author can read the exact text that would be sent, with its size, destination and
model; nothing is sent until they ask. The context holds no archived entity.

**Keys belong to the operating system.** A provider's key is stored in the system credential store
and written to no file: not the project, its backups or exports, nor the provider list. It travels
from the backend to the provider in a header; the interface can set it and learn whether one is
held, never read it. Providers over the internet must be called over HTTPS. All provider traffic
is made by the backend, so the webview's content security policy and permissions are unchanged.

**MCP is local and small.** `ontara-mcp <project>` speaks JSON-RPC over standard input and output;
it opens no port. Its tools read (`search`, `list_entities`, `get_entity`, `check_consistency`) and
propose (`propose_changes`). It answers nothing until the project's policy allows it, checked on
every call; it never returns an archived entity; its results are size-limited.

**The core does not depend on any of it.** No assistant code runs at start, none of the core
commands consult a provider, and a provider that is down or absent changes nothing: the
application reads, searches, edits, checks, backs up and exports as before.

## Consequences

- An assistant's worst case is a bad suggestion the author declines.
- A malicious text in the universe can ask a model for anything, but can only ever produce
  proposals from the closed set.
- Some data does reach a provider the author chose and allowed; the preview and the policy make
  that visible and deliberate, and local providers make it avoidable.
- `docs/security-privacy.md` lists the threats and the test that holds each one.
