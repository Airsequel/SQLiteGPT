# SQLiteGPT

SQLite [loadable extension](https://www.sqlite.org/loadext.html) that exposes
a `gpt()` SQL function for querying OpenAI's chat completions API directly
from SQL.

Implemented in Rust with [rusqlite](https://github.com/rusqlite/rusqlite).


## Build

```sh
make build
```

This produces `target/release/libsqlitegpt.dylib` on macOS,
`libsqlitegpt.so` on Linux, or `sqlitegpt.dll` on Windows.


## Usage

```sh
export OPENAI_API_KEY="sk-XXX"
sqlite3 example.sqlite
```

```sql
.load ./target/release/libsqlitegpt
SELECT gpt('Name of the tallest mountain, no explanation');

SELECT
  country,
  gpt('capital of ' || country || ', no explanation') AS capital
FROM users;
```

The extension's entry point is derived from the file name, so plain
`.load ./target/release/libsqlitegpt` resolves to `sqlite3_sqlitegpt_init`.


## Configuration

| Environment variable | Default          | Purpose                                     |
| -------------------- | ---------------- | ------------------------------------------- |
| `OPENAI_API_KEY`     | *(required)*     | OpenAI API key                              |
| `OPENAI_MODEL`       | `gpt-5.4-mini`   | Chat completions model                      |
| `SQLITEGPT_NO_CACHE` | *(unset)*        | Set to any value to disable response caching |


## Notes

- `gpt()` is synchronous: each row blocks on the HTTP round-trip.
  For non-trivial result sets this is slow and consumes API credits per row.
- Responses are cached in-process by `(model, prompt)`. Repeating an identical
  prompt within the same SQLite session reuses the prior answer. Set
  `SQLITEGPT_NO_CACHE` to disable.
- The HTTP client uses a 60s global timeout, so a hung request cannot stall
  a query indefinitely.
- `NULL` input propagates: `gpt(NULL)` returns `NULL` without contacting OpenAI.
- The function is registered with `SQLITE_DIRECTONLY`, so it cannot be invoked
  from views, triggers, `CHECK`/`DEFAULT` clauses, expression/partial indexes,
  or generated columns. This prevents a hostile database file from triggering
  network calls (and API spend) just by being opened.
- The function is **not** marked `SQLITE_DETERMINISTIC` because the model's
  output is not stable across calls.


## Development

The crate is split into a workspace: `sqlitegpt-core` holds the logic and
tests (linked against bundled SQLite), and the top-level `sqlitegpt` crate
is the cdylib shim that exposes `sqlite3_sqlitegpt_init`.

```sh
make test   # cargo test -p sqlitegpt-core --release
```
