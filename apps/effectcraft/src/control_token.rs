//! The control server's bearer token (`docs/control-protocol.md` › Authentication).
//!
//! Every connection must send `{"method": "auth", "params": {"token": "<64 hex>"}}` first. The
//! token comes from `EFFECTCRAFT_CONTROL_TOKEN`, else from the file named by
//! `EFFECTCRAFT_CONTROL_TOKEN_FILE` (created with a fresh token, mode 0600 on Unix, when it does not
//! exist), else a fresh one is made for this launch and printed on standard error.

use std::io::Write;
use std::path::Path;

/// Hex digits in a token (256 bits).
pub const TOKEN_LEN: usize = 64;

/// A fresh random token from the operating system's generator.
pub fn generate() -> Result<String, String> {
    let mut bytes = [0u8; TOKEN_LEN / 2];
    getrandom::fill(&mut bytes).map_err(|e| format!("no random numbers for the control token: {e}"))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// Whether `token` is a well-formed token: exactly 64 hexadecimal digits.
pub fn is_valid(token: &str) -> bool {
    token.len() == TOKEN_LEN && token.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Compare a presented token with the server's in constant time (for equal lengths).
pub fn matches(presented: &str, expected: &str) -> bool {
    let (a, b) = (presented.as_bytes(), expected.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Read the token in `path`, or create the file with a fresh token when it does not exist.
pub fn load_or_create(path: &Path) -> Result<String, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let token = text.trim().to_string();
            if is_valid(&token) { Ok(token) } else { Err(format!("{}: not a control token (expected {TOKEN_LEN} hexadecimal digits)", path.display())) }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let token = generate()?;
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(path).map_err(|e| format!("can't create {}: {e}", path.display()))?;
            file.write_all(token.as_bytes()).and_then(|()| file.flush()).map_err(|e| format!("can't write {}: {e}", path.display()))?;
            Ok(token)
        }
        Err(e) => Err(format!("can't read {}: {e}", path.display())),
    }
}

/// Where this launch's token came from, for the start-up message.
#[derive(Debug, PartialEq)]
pub enum Source {
    Env,
    File,
    Generated,
}

/// The token for this launch, from `EFFECTCRAFT_CONTROL_TOKEN` (`env_token`), the
/// `EFFECTCRAFT_CONTROL_TOKEN_FILE` file (`env_file`), or a fresh one, in that order. Empty values
/// count as unset.
pub fn resolve(env_token: Option<String>, env_file: Option<std::path::PathBuf>) -> Result<(String, Source), String> {
    if let Some(token) = env_token.map(|t| t.trim().to_string()).filter(|t| !t.is_empty()) {
        return if is_valid(&token) { Ok((token, Source::Env)) } else { Err(format!("EFFECTCRAFT_CONTROL_TOKEN must be {TOKEN_LEN} hexadecimal digits")) };
    }
    if let Some(path) = env_file.filter(|p| !p.as_os_str().is_empty()) {
        return load_or_create(&path).map(|t| (t, Source::File));
    }
    generate().map(|t| (t, Source::Generated))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_tokens_are_fresh_and_well_formed() {
        let (a, b) = (generate().unwrap(), generate().unwrap());
        assert!(is_valid(&a) && is_valid(&b));
        assert_ne!(a, b);
    }

    #[test]
    fn every_single_character_difference_is_refused() {
        let t = generate().unwrap();
        assert!(matches(&t, &t));
        for i in 0..t.len() {
            let mut other = t.clone().into_bytes();
            other[i] = if other[i] == b'0' { b'1' } else { b'0' };
            assert!(!matches(std::str::from_utf8(&other).unwrap(), &t), "position {i}");
        }
        assert!(!matches(&t[..63], &t));
        assert!(!matches(&format!("{t}0"), &t));
        assert!(!matches("", &t));
    }

    #[test]
    fn the_token_file_is_created_private_and_then_reused() {
        let dir = std::env::temp_dir().join(format!("effectcraft-token-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("control.token");
        let _ = std::fs::remove_file(&path);
        let (first, source) = resolve(None, Some(path.clone())).unwrap();
        assert_eq!(source, Source::File);
        assert!(is_valid(&first));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        }
        assert_eq!(resolve(None, Some(path.clone())).unwrap().0, first, "an existing file is reused");
        std::fs::write(&path, "not a token").unwrap();
        assert!(resolve(None, Some(path.clone())).is_err(), "a bad file is an error, not a fresh token");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_environment_token_wins_and_must_be_well_formed() {
        let t = "ab".repeat(32);
        assert_eq!(resolve(Some(t.clone()), Some("/nonexistent/dir/x".into())).unwrap(), (t, Source::Env));
        assert!(resolve(Some("short".into()), None).is_err());
        let (generated, source) = resolve(Some(" ".into()), Some("".into())).unwrap();
        assert_eq!(source, Source::Generated);
        assert!(is_valid(&generated));
    }
}
