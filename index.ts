
import * as betterSqlite from "better-sqlite3"
import syncRequest from "sync-request"

// Connect to OpenAI
const apiKey = process.env.OPENAI_API_KEY

if (!apiKey) {
  console.error("Missing OPENAI_API_KEY environment variable")
  process.exit(1)
}


// // TODO: Use this after `db.function()` supports async callbacks
// import { Configuration, OpenAIApi } from "openai"
// const openAiConfig = new Configuration({ apiKey })
// const openAi = new OpenAIApi(openAiConfig)

// async function callOpenAi (prompt: string) {
//   console.info(prompt)

//   const completion = await openAi.createChatCompletion({
//     model: "gpt-3.5-turbo",
//     messages: [
//       {
//         role: "user",
//         content: prompt,
//       },
//     ],
//   })

//   return completion?.data?.choices?.[0]?.message?.content
// }


function callOpenAiSync (prompt: string) {
  const response = syncRequest(
    "POST",
    "https://api.openai.com/v1/chat/completions",
    {
      headers: {
        Authorization: `Bearer ${apiKey}`,
      },
      json: {
        model: "gpt-3.5-turbo",
        messages: [
          {
            role: "user",
            content: prompt,
          },
        ],
      },
    }
  )
  const data = JSON.parse(response.getBody('utf8'))

  return data?.choices?.[0]?.message?.content
}


export function sqliteGpt (database: string, sqlQuery: string) {
  const options = {}
  const db = betterSqlite(database, options)

  db.function(
    "gpt",
    // @ts-ignore-next-line
    (prompt: string) => callOpenAiSync(prompt)
  )

  return db
    .prepare(sqlQuery)
    .all()
}
