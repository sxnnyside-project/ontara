# Security and privacy

What can go wrong with assistants, and what holds it. Every row names the test that fails if it
stops being true. The decision is [0009](decisions/0009-assistants-proposals-and-mcp.md).

| Threat | What holds it | Test |
|---|---|---|
| Text of the universe leaves the computer without the author meaning it | Cloud sending is off per project; the request text is shown first; nothing is sent until Ask | `assistant::tests::a_provider_outside_the_computer_is_refused_until_the_project_allows_it`, `nothing_is_sent_without_a_configured_provider_or_before_it_is_run` |
| A provider key is leaked to the interface, a file or a log | Stored in the system keychain; the provider view has only `has_key`; the key is only in a header | `a_key_is_never_handed_back_and_a_bad_address_is_refused`, `a_key_is_written_to_no_file_of_the_project_or_of_the_configuration` |
| A key travels in the clear | HTTPS required outside this computer | `assistant::providers::tests::plain_http_is_only_accepted_on_this_machine` |
| A host that only looks local is trusted | Locality is the parsed host, not a prefix | `only_this_machine_counts_as_local` |
| A model changes the universe directly | Answers become proposals; the operation set is closed | `ontara-core::proposal::tests::only_the_closed_set_of_operations_can_be_read`, `an_answer_becomes_a_proposal_with_only_what_can_be_applied` |
| Hostile text in an entity steers the model into destructive operations | Universe text is delimited as data; operations outside the set are dropped; nothing applies without approval | `text_inside_the_universe_cannot_widen_what_an_assistant_may_do` |
| An assistant makes its work canon, or hides its origin | Created records are `provisional` and carry `ai` or `mcp` | `what_an_assistant_creates_says_so_and_is_not_canon` |
| A half-applied change corrupts the model | One savepoint per change; a refusal leaves nothing | `ontara-storage::proposals::a_proposal_changes_nothing_until_it_is_approved` |
| An approval cannot be taken back | The decision is one change set | `approving_is_one_edit_that_undo_takes_back_with_the_statuses` |
| An MCP client reads a project that did not offer it | Policy is checked on every call | `mcp::tests::a_project_answers_no_one_until_the_author_turns_access_on` |
| An MCP client edits the model | There is no tool that edits; `propose_changes` only adds a proposal | `with_access_on_a_client_reads_and_can_only_propose` |
| Hidden or archived material is exposed | Archived entities are left out of context and of MCP results | `archived_entities_are_not_part_of_what_is_sent`, `archived_entities_and_oversized_answers_stay_back` |
| A provider error echoes the request | Errors are reduced to a status and a short reason | `a_provider_failure_is_reported_without_the_request` |
| The core stops working when a provider is down | No core path calls a provider | `an_unreachable_provider_leaves_the_universe_working_and_unchanged`, `workflow::an_author_builds_checks_protects_and_exports_a_universe` |
| Oversized answers exhaust the review | At most 40 changes, 20,000 characters each, MCP results capped | `ontara-core::proposal::tests::a_proposal_needs_a_title_and_changes_a_person_can_review` |

## What is not covered

- What a provider does with text once the author has allowed it. Local providers avoid the
  question; for the others, the provider's own terms apply.
- Prompt injection can still make a model *suggest* something misleading. The review exists for
  that, and shows the exact change.
- The operating system's keychain is trusted as the operating system is.
- `ontara-mcp` ships beside the application when built with `cargo build`; installers do not yet
  bundle it as a sidecar.
