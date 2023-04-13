# SQLiteGPT

SQL function for SQLite to directly query OpenAI's ChatGPT.


## CLI

### Installation

```sh
npm install --global sqlitegpt
```


### Usage

```sh
export OPENAI_API_KEY="sk-XXX"
sqlitegpt example.sqlite "SELECT gpt('Name of tallest mountain')"
```


## Library Usage

```ts
import { sqliteGpt } from "sqlitegpt"

sqliteGpt(
  ":memory:",
  "SELECT gpt('Name of tallest mountain') as tallest_mountain"
)

sqliteGpt(
  "example.sqlite",
  `
    SELECT
      country,
      gpt('capital of ' || country || ', no explanation') as capital
    FROM users
  `
)
```


## Known Issues

- This implementation currently uses `sync-request`
    which forces HTTP requests to be synchronous.
    You should not use this in production!
    However, it's currently the only way to make it work with `better-sqlite3`.
- It's only implemented as an
    [application-defined SQL function](https://sqlite.org/appfunc.html)
    and can therefore only be used in JavaScript projects.
    To make it work in every SQLite environment,
    it should be implemented as a
    [loadable extension](https://www.sqlite.org/loadext.html).


---

**If you like SQLiteGPT, please star the project
and I will re-implement it as a proper loadable extension in Rust with
[sqlite-loadable-rs](https://github.com/asg017/sqlite-loadable-rs).**

---
