# Changelog

All notable changes to this project are documented in this file.


## 2026-05-27 - 0.2.0

### Changed

- Rewrote the project in Rust as a SQLite [loadable extension]
  (`libsqlitegpt.{dylib,so,dll}`), replacing the previous TypeScript CLI.
  `gpt()` is now invoked from SQL via `.load` instead of run as a separate
  command-line tool.
- Split the crate into a workspace: `sqlitegpt-core` holds the logic and
  tests (linked against bundled SQLite), `sqlitegpt` is the cdylib shim
  exposing `sqlite3_sqlitegpt_init`.

### Added

- Registered `gpt()` with `SQLITE_DIRECTONLY` so it cannot be invoked from
  views, triggers, `CHECK`/`DEFAULT` clauses, expression/partial indexes, or
  generated columns. Prevents a hostile database file from triggering
  network calls (and API spend) just by being opened.
- `NULL` input propagation: `gpt(NULL)` returns `NULL` without reading env
  vars or contacting OpenAI.
- 60-second global HTTP timeout via a shared `ureq` agent, so a hung
  request cannot stall a query indefinitely.
- In-process response cache keyed by `(model, prompt)`. Repeating an
  identical prompt within the same SQLite session reuses the prior answer.
  Set `SQLITEGPT_NO_CACHE` to disable.
- `OPENAI_MODEL` environment variable to override the default model.
- Test suite covering response parsing, cache behaviour, `NULL`
  propagation, `DIRECTONLY` enforcement, and HTTP timeout.

[loadable extension]: https://www.sqlite.org/loadext.html


## 2023-04-13 - 0.1.1

- Bumped package version; no user-visible changes.


## 2023-04-12 - 0.1.0

- Initial TypeScript implementation: CLI wrapper that registered a `gpt()`
  function against a SQLite database and proxied prompts to the OpenAI
  chat completions API.
