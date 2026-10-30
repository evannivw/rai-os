//! Local secret loading. Never execute a dotenv file or include secrets in errors.
use std::{fs::File, io::Read, path::Path};
pub struct Secret(String); // Deliberately no Debug/Serialize implementation.
impl Secret {
    pub fn value(&self) -> &str {
        &self.0
    }
    fn parse(value: String) -> Result<Self, &'static str> {
        if !(16..=512).contains(&value.len())
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b))
        {
            return Err("invalid_api_key_format");
        }
        Ok(Self(value))
    }
}
pub fn bounded_file(path: &Path, limit: u64) -> Result<String, &'static str> {
    let file = File::open(path).map_err(|_| "cannot_open_local_file")?;
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot_read_local_file")?;
    if bytes.len() as u64 > limit {
        return Err("local_file_too_large");
    }
    String::from_utf8(bytes).map_err(|_| "local_file_not_utf8")
}
fn parse_dotenv(text: &str) -> Result<Secret, &'static str> {
    let mut key = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let (name, value) = line.split_once('=').ok_or("invalid_local_configuration")?;
        if name.trim() != "MODEL_API_KEY" {
            continue;
        }
        if key.is_some() {
            return Err("duplicate_api_key_setting");
        }
        let value = value.trim();
        let value = value
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .or_else(|| value.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
            .unwrap_or(value);
        key = Some(Secret::parse(value.to_owned())?);
    }
    key.ok_or("missing_MODEL_API_KEY")
}
pub fn load() -> Result<Secret, &'static str> {
    match std::env::var("MODEL_API_KEY") {
        Ok(value) => Secret::parse(value),
        Err(std::env::VarError::NotPresent) => {
            parse_dotenv(&bounded_file(Path::new(".env.local"), 4096)?)
        }
        Err(_) => Err("invalid_api_key_encoding"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn secret_parser_rejects_injection_without_echoing_input() {
        let dummy = "LLM_000000000_dummy_test_key";
        assert!(parse_dotenv(&format!("# comment\nexport MODEL_API_KEY = '{dummy}'\n")).is_ok());
        for text in [
            format!("MODEL_API_KEY={dummy}\nMODEL_API_KEY={dummy}"),
            "MODEL_API_KEY=$(touch /tmp/never)".into(),
            "MODEL_API_KEY=\"bad\\nheader\"".into(),
            "MODEL_API_KEY=".into(),
            "MODEL_API_KEY=short".into(),
        ] {
            let result = parse_dotenv(&text);
            assert!(result.is_err());
            assert!(!result.err().unwrap().contains(dummy));
        }
    }
}
