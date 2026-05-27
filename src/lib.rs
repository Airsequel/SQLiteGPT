use std::env;
use std::os::raw::{c_char, c_int};

use rusqlite::ffi;
use rusqlite::functions::FunctionFlags;
use rusqlite::types::{ToSqlOutput, Value};
use rusqlite::{Connection, Error, Result};
use serde_json::json;

const OPENAI_URL: &str = "https://api.openai.com/v1/chat/completions";
const DEFAULT_MODEL: &str = "gpt-5.4-mini";

fn call_openai(prompt: &str) -> std::result::Result<String, String> {
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| "Missing OPENAI_API_KEY environment variable".to_string())?;
    let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());

    let body = json!({
        "model": model,
        "messages": [{ "role": "user", "content": prompt }],
    });

    let response: serde_json::Value = ureq::post(OPENAI_URL)
        .header("Authorization", format!("Bearer {api_key}"))
        .send_json(&body)
        .map_err(|e| format!("OpenAI request failed: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("Failed to parse OpenAI response: {e}"))?;

    response["choices"][0]["message"]["content"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("Unexpected OpenAI response shape: {response}"))
}

fn register(db: Connection) -> Result<bool> {
    db.create_scalar_function(
        c"gpt",
        1,
        FunctionFlags::SQLITE_UTF8,
        |ctx| {
            let prompt: String = ctx.get(0)?;
            match call_openai(&prompt) {
                Ok(text) => Ok(ToSqlOutput::Owned(Value::Text(text))),
                Err(msg) => Err(Error::UserFunctionError(msg.into())),
            }
        },
    )?;
    Ok(false)
}

/// Loadable-extension entry point. Resolved by SQLite from the shared library
/// file name (`libsqlitegpt.{dylib,so,dll}` → `sqlite3_sqlitegpt_init`).
/// # Safety
/// Invoked by SQLite with valid pointers per the loadable-extension contract.
#[no_mangle]
pub unsafe extern "C" fn sqlite3_sqlitegpt_init(
    db: *mut ffi::sqlite3,
    pz_err_msg: *mut *mut c_char,
    p_api: *mut ffi::sqlite3_api_routines,
) -> c_int {
    Connection::extension_init2(db, pz_err_msg, p_api, register)
}
