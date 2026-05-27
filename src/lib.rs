use std::os::raw::{c_char, c_int};

use rusqlite::ffi;
use rusqlite::{Connection, Result};

fn register(conn: Connection) -> Result<bool> {
  sqlitegpt_core::register_functions(&conn)?;
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
