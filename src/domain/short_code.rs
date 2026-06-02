use rand::Rng;
use thiserror::Error;

pub const BASE62_ALPHABET: &[u8; 62] =
    b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ShortCodeError {
    #[error("short code must be between {min} and {max} characters")]
    InvalidLength { min: usize, max: usize },
    #[error("short code can only contain letters, numbers, hyphen, and underscore")]
    InvalidCharacters,
    #[error("short code is reserved")]
    Reserved,
}

pub fn encode_base62(mut value: u128) -> String {
    if value == 0 {
        return "0".to_string();
    }

    let mut encoded = Vec::new();

    while value > 0 {
        let index = (value % 62) as usize;
        encoded.push(BASE62_ALPHABET[index] as char);
        value /= 62;
    }

    encoded.iter().rev().collect()
}

pub fn random_base62_code(length: usize) -> String {
    let mut rng = rand::rng();

    (0..length)
        .map(|_| {
            let index = rng.random_range(0..BASE62_ALPHABET.len());
            BASE62_ALPHABET[index] as char
        })
        .collect()
}

pub fn validate_custom_code(code: &str) -> Result<(), ShortCodeError> {
    const MIN: usize = 3;
    const MAX: usize = 64;
    const RESERVED: &[&str] = &["health", "urls", "metrics"];

    if code.len() < MIN || code.len() > MAX {
        return Err(ShortCodeError::InvalidLength { min: MIN, max: MAX });
    }

    if !code
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(ShortCodeError::InvalidCharacters);
    }

    if RESERVED
        .iter()
        .any(|reserved| code.eq_ignore_ascii_case(reserved))
    {
        return Err(ShortCodeError::Reserved);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_unsigned_integer_to_base62() {
        assert_eq!(encode_base62(0), "0");
        assert_eq!(encode_base62(1), "1");
        assert_eq!(encode_base62(9), "9");
        assert_eq!(encode_base62(10), "a");
        assert_eq!(encode_base62(35), "z");
        assert_eq!(encode_base62(36), "A");
        assert_eq!(encode_base62(61), "Z");
        assert_eq!(encode_base62(62), "10");
        assert_eq!(encode_base62(3843), "ZZ");
        assert_eq!(encode_base62(3844), "100");
    }

    #[test]
    fn random_code_uses_only_base62_characters() {
        let code = random_base62_code(32);

        assert_eq!(code.len(), 32);
        assert!(code.bytes().all(|byte| BASE62_ALPHABET.contains(&byte)));
    }

    #[test]
    fn validates_custom_codes() {
        assert!(validate_custom_code("gabriel_123").is_ok());
        assert!(validate_custom_code("my-link").is_ok());
        assert_eq!(
            validate_custom_code("ab"),
            Err(ShortCodeError::InvalidLength { min: 3, max: 64 })
        );
        assert_eq!(
            validate_custom_code("bad/code"),
            Err(ShortCodeError::InvalidCharacters)
        );
        assert_eq!(validate_custom_code("urls"), Err(ShortCodeError::Reserved));
    }
}
