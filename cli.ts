#! /usr/bin/env node

import { sqliteGpt } from "./index.js"


const [database, sqlQuery] = process.argv.slice(2)

if (!database || !sqlQuery) {
  console.error("Usage: sqlitegpt <database> <sql-query>")
  process.exit(1)
}

console.info(
  sqliteGpt(database, sqlQuery)
)
