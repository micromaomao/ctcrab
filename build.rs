use std::error::Error;
use std::process::Command;
use std::env;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

fn main() -> Result<(), String> {
  const LAYOUT_SASS: &'static str = "templates/layout.sass";
  let layout_css = "static/layout.css";
  println!("cargo:rerun-if-changed={}", LAYOUT_SASS);
  let ret = Command::new("sassc")
      .args(&[LAYOUT_SASS, layout_css])
      .status().map_err(|e| format!("exec: {}", e))?;
  if !ret.success() {
    return Err(format!("sassc exited with {}", ret));
  }
  Ok(())
}
