import { sqliteGpt } from "."

console.info(
  sqliteGpt(
    ":memory:",
    "SELECT gpt('Name of tallest mountain, no explanation') as tallest_mountain"
  )
)

console.info(
  sqliteGpt(
    "example.sqlite",
    `
      SELECT
        country,
        gpt('capital of ' || country || ', no explanation') as capital
      FROM users
    `
  )
)
