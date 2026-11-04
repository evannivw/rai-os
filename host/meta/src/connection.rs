//! Explicit credential checks, independent of guest execution and application data.
use crate::{config, provider};
use serde_json::json;

pub fn run(live: bool) -> Result<(), String> {
    let key = match config::load() {
        Ok(key) => key,
        Err(code) => {
            println!(
                "{}",
                json!({"event":"meta_connection","status":"error","code":code})
            );
            return Err("meta_connection_check_failed".into());
        }
    };
    if !live {
        println!(
            "{}",
            json!({"event":"meta_connection","status":"configured","model":provider::MODEL})
        );
        return Ok(());
    }
    match provider::check_connection(&key) {
        Ok(answer) => {
            println!(
                "{}",
                json!({"event":"meta_connection","status":"ready","model":provider::MODEL,"diagnostic":answer.diagnostic})
            );
            Ok(())
        }
        Err(error) => {
            println!(
                "{}",
                json!({"event":"meta_connection","status":"error","code":error.code,"diagnostic":error.diagnostic})
            );
            Err("meta_connection_check_failed".into())
        }
    }
}
