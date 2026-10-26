//! Pure operations and protocol validation: no terminal or process access.
pub enum Operation<'a> {
    Help,
    Info,
    Echo(&'a str),
}

pub struct Reply {
    pub success: bool,
    pub payload: String,
}

pub fn parse_input(line: &str) -> Result<Operation<'_>, &'static str> {
    match line {
        "help" => Ok(Operation::Help),
        "info" => Ok(Operation::Info),
        "echo" => Ok(Operation::Echo("")),
        _ => line
            .strip_prefix("echo ")
            .map(Operation::Echo)
            .ok_or("Use help, info, echo <text>, or exit"),
    }
}

pub fn encode(id: u32, operation: &Operation<'_>) -> Result<String, &'static str> {
    if id == 0 {
        return Err("ID zero is reserved for transport errors");
    }
    let body = match operation {
        Operation::Help => "help".to_owned(),
        Operation::Info => "info".to_owned(),
        Operation::Echo(text) => {
            if !text.bytes().all(|b| b.is_ascii_graphic() || b == b' ') {
                return Err("echo accepts printable ASCII, without TAB or line breaks");
            }
            format!("echo\t{text}")
        }
    };
    let frame = format!("@rai1\t{id}\t{body}");
    if frame.len() > 256 {
        return Err("Request exceeds the 256-byte UART limit");
    }
    Ok(frame + "\n")
}

pub fn decode(id: u32, line: &str) -> Result<Reply, &'static str> {
    let fields: Vec<_> = line.split('\t').collect();
    if fields.len() != 4 || fields[0] != "@rai1" {
        return Err("Unexpected response format");
    }
    if fields[1] != id.to_string() {
        return Err("Unexpected response ID; cannot match this request");
    }
    if !fields[3].bytes().all(|b| b.is_ascii_graphic() || b == b' ') {
        return Err("Response contains control characters or non-ASCII bytes");
    }
    let success = match fields[2] {
        "ok" => true,
        "err" => false,
        _ => return Err("Unknown response status"),
    };
    Ok(Reply {
        success,
        payload: fields[3].to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_injection_and_oversized_requests() {
        for text in ["a\n@rai1\t2\tinfo", "a\rb", "a\tb", "\x1b[2J", "ñ"] {
            assert!(encode(1, &Operation::Echo(text)).is_err());
        }
        let prefix = "@rai1\t1\techo\t";
        let payload = "x".repeat(256 - prefix.len());
        assert_eq!(encode(1, &Operation::Echo(&payload)).unwrap().len(), 257);
        assert!(encode(1, &Operation::Echo(&(payload + "x"))).is_err());
    }

    #[test]
    fn rejects_wrong_request_and_malformed_responses() {
        for frame in [
            "@rai1\t2\tok\tx",
            "@rai2\t1\tok\tx",
            "@rai1\t1\tmaybe\tx",
            "@rai1\t1\tok\tx\textra",
            "@rai1\t1\tok\t\x1b[2J",
        ] {
            assert!(decode(1, frame).is_err());
        }
        assert!(decode(1, "@rai1\t1\tok\t").unwrap().success);
        assert!(!decode(1, "@rai1\t1\terr\tbad_args").unwrap().success);
    }
}
