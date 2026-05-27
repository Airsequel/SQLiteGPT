use std::collections::HashMap;
use std::env;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use rusqlite::functions::FunctionFlags;
use rusqlite::types::{ToSqlOutput, Value, ValueRef};
use rusqlite::{Connection, Error, Result};
use serde_json::json;
use ureq::Agent;

const OPENAI_URL: &str = "https://api.openai.com/v1/chat/completions";
const DEFAULT_MODEL: &str = "gpt-5.4-mini";
const HTTP_TIMEOUT: Duration = Duration::from_secs(60);

static RESPONSE_CACHE: LazyLock<Mutex<HashMap<(String, String), String>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

static AGENT: LazyLock<Agent> = LazyLock::new(build_agent);

fn build_agent() -> Agent {
  Agent::config_builder()
    .timeout_global(Some(HTTP_TIMEOUT))
    .build()
    .into()
}

fn call_openai(
  agent: &Agent,
  url: &str,
  api_key: &str,
  model: &str,
  prompt: &str,
) -> std::result::Result<String, String> {
  let body = json!({
      "model": model,
      "messages": [{ "role": "user", "content": prompt }],
  });

  let response: serde_json::Value = agent
    .post(url)
    .header("Authorization", format!("Bearer {api_key}"))
    .send_json(&body)
    .map_err(|e| format!("OpenAI request failed: {e}"))?
    .body_mut()
    .read_json()
    .map_err(|e| format!("Failed to parse OpenAI response: {e}"))?;

  parse_content(&response)
}

fn parse_content(
  response: &serde_json::Value,
) -> std::result::Result<String, String> {
  response["choices"][0]["message"]["content"]
    .as_str()
    .map(str::to_owned)
    .ok_or_else(|| format!("Unexpected OpenAI response shape: {response}"))
}

fn cached<F>(
  model: &str,
  prompt: &str,
  use_cache: bool,
  f: F,
) -> std::result::Result<String, String>
where
  F: FnOnce() -> std::result::Result<String, String>,
{
  if use_cache {
    if let Some(hit) = RESPONSE_CACHE
      .lock()
      .unwrap()
      .get(&(model.to_owned(), prompt.to_owned()))
    {
      return Ok(hit.clone());
    }
  }
  let value = f()?;
  if use_cache {
    RESPONSE_CACHE
      .lock()
      .unwrap()
      .insert((model.to_owned(), prompt.to_owned()), value.clone());
  }
  Ok(value)
}

pub fn register_functions(db: &Connection) -> Result<()> {
  db.create_scalar_function(
    c"gpt",
    1,
    FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DIRECTONLY,
    |ctx| {
      if let ValueRef::Null = ctx.get_raw(0) {
        return Ok(ToSqlOutput::Owned(Value::Null));
      }
      let prompt: String = ctx.get(0)?;

      let api_key = env::var("OPENAI_API_KEY").map_err(|_| {
        Error::UserFunctionError(
          "Missing OPENAI_API_KEY environment variable".into(),
        )
      })?;
      let model =
        env::var("OPENAI_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
      let use_cache = env::var("SQLITEGPT_NO_CACHE").is_err();

      let result = cached(&model, &prompt, use_cache, || {
        call_openai(&AGENT, OPENAI_URL, &api_key, &model, &prompt)
      });

      match result {
        Ok(text) => Ok(ToSqlOutput::Owned(Value::Text(text))),
        Err(msg) => Err(Error::UserFunctionError(msg.into())),
      }
    },
  )?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::{Read, Write};
  use std::net::TcpListener;
  use std::sync::atomic::{AtomicUsize, Ordering};
  use std::sync::Arc;
  use std::thread;

  #[test]
  fn parse_content_extracts_message() {
    let resp = json!({"choices": [{"message": {"content": "hello"}}]});
    assert_eq!(parse_content(&resp).unwrap(), "hello");
  }

  #[test]
  fn parse_content_errors_on_unexpected_shape() {
    let resp = json!({"error": "boom"});
    let err = parse_content(&resp).unwrap_err();
    assert!(err.contains("Unexpected OpenAI response shape"));
  }

  #[test]
  fn cache_returns_stored_value_without_recomputing() {
    let model = "cache-hit-model";
    let prompt = "cache-hit-prompt";
    let counter = Arc::new(AtomicUsize::new(0));

    let c1 = counter.clone();
    let r1 = cached(model, prompt, true, move || {
      c1.fetch_add(1, Ordering::SeqCst);
      Ok("answer".to_string())
    })
    .unwrap();

    let c2 = counter.clone();
    let r2 = cached(model, prompt, true, move || {
      c2.fetch_add(1, Ordering::SeqCst);
      Ok("answer".to_string())
    })
    .unwrap();

    assert_eq!(r1, "answer");
    assert_eq!(r2, "answer");
    assert_eq!(counter.load(Ordering::SeqCst), 1);
  }

  #[test]
  fn cache_respects_disable_flag() {
    let model = "cache-disabled-model";
    let prompt = "cache-disabled-prompt";
    let counter = Arc::new(AtomicUsize::new(0));

    let c1 = counter.clone();
    cached(model, prompt, false, move || {
      c1.fetch_add(1, Ordering::SeqCst);
      Ok("a".to_string())
    })
    .unwrap();
    let c2 = counter.clone();
    cached(model, prompt, false, move || {
      c2.fetch_add(1, Ordering::SeqCst);
      Ok("a".to_string())
    })
    .unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 2);
  }

  #[test]
  fn cache_separates_by_model() {
    let prompt = "shared-prompt-by-model";
    let counter = Arc::new(AtomicUsize::new(0));

    let c1 = counter.clone();
    let r1 = cached("model-A", prompt, true, move || {
      c1.fetch_add(1, Ordering::SeqCst);
      Ok("from-A".to_string())
    })
    .unwrap();
    let c2 = counter.clone();
    let r2 = cached("model-B", prompt, true, move || {
      c2.fetch_add(1, Ordering::SeqCst);
      Ok("from-B".to_string())
    })
    .unwrap();

    assert_eq!(r1, "from-A");
    assert_eq!(r2, "from-B");
    assert_eq!(counter.load(Ordering::SeqCst), 2);
  }

  #[test]
  fn cache_does_not_store_errors() {
    let model = "cache-err-model";
    let prompt = "cache-err-prompt";
    let counter = Arc::new(AtomicUsize::new(0));

    let c1 = counter.clone();
    let r1 = cached(model, prompt, true, move || {
      c1.fetch_add(1, Ordering::SeqCst);
      Err("boom".to_string())
    });
    assert!(r1.is_err());

    let c2 = counter.clone();
    let r2 = cached(model, prompt, true, move || {
      c2.fetch_add(1, Ordering::SeqCst);
      Ok("ok".to_string())
    })
    .unwrap();
    assert_eq!(r2, "ok");
    assert_eq!(counter.load(Ordering::SeqCst), 2);
  }

  #[test]
  fn null_input_returns_null_without_calling_api() {
    // No OPENAI_API_KEY is required because NULL must short-circuit
    // before any env lookup or HTTP call.
    env::remove_var("OPENAI_API_KEY");

    let conn = Connection::open_in_memory().unwrap();
    register_functions(&conn).unwrap();

    let value: Option<String> = conn
      .query_row("SELECT gpt(NULL)", [], |row| row.get(0))
      .unwrap();
    assert_eq!(value, None);
  }

  #[test]
  fn directonly_blocks_indirect_uses() {
    let conn = Connection::open_in_memory().unwrap();
    register_functions(&conn).unwrap();

    let attempts: [&str; 4] = [
            "CREATE VIEW v AS SELECT gpt('x') AS x",
            "CREATE TABLE t_check(x TEXT CHECK (gpt(x) = 'ok'))",
            "CREATE TABLE t_default(x TEXT DEFAULT (gpt('hi')))",
            "CREATE TRIGGER tr AFTER INSERT ON sqlite_schema BEGIN SELECT gpt('x'); END",
        ];

    let mut messages = Vec::new();
    for sql in attempts {
      if let Err(e) = conn.execute(sql, []) {
        messages.push(format!("{e}").to_lowercase());
      }
    }
    if let Err(e) = conn.prepare("SELECT * FROM v") {
      messages.push(format!("{e}").to_lowercase());
    }

    // SQLite reports DIRECTONLY violations as "unsafe use of <fn>()".
    assert!(
      messages.iter().any(|m| m.contains("unsafe use of gpt")),
      "expected a DIRECTONLY error in one of the attempts, got: {messages:?}"
    );
  }

  #[test]
  fn http_timeout_returns_error_quickly() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{addr}/v1/chat/completions");

    thread::spawn(move || {
      for stream in listener.incoming().flatten() {
        let mut s = stream;
        let mut buf = [0u8; 4096];
        let _ = s.read(&mut buf);
        thread::sleep(Duration::from_secs(5));
        let _ = s.write_all(b"");
      }
    });

    let agent: Agent = Agent::config_builder()
      .timeout_global(Some(Duration::from_millis(200)))
      .build()
      .into();

    let start = std::time::Instant::now();
    let result = call_openai(&agent, &url, "test-key", "test-model", "hi");
    let elapsed = start.elapsed();

    assert!(result.is_err(), "expected timeout error, got {result:?}");
    let msg = result.unwrap_err();
    assert!(
      msg.contains("OpenAI request failed"),
      "unexpected error message: {msg}"
    );
    assert!(
      elapsed < Duration::from_secs(2),
      "timeout should fire fast, took {elapsed:?}"
    );
  }
}
