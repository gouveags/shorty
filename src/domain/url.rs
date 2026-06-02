use thiserror::Error;
use url::Url;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LongUrlError {
    #[error("url must be absolute http or https")]
    Invalid,
}

pub fn validate_long_url(value: &str) -> Result<(), LongUrlError> {
    let parsed = Url::parse(value).map_err(|_| LongUrlError::Invalid)?;

    match parsed.scheme() {
        "http" | "https" if parsed.host().is_some() => Ok(()),
        _ => Err(LongUrlError::Invalid),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_http_and_https_urls() {
        assert!(validate_long_url("https://example.com/a").is_ok());
        assert!(validate_long_url("http://example.com/a").is_ok());
    }

    #[test]
    fn rejects_relative_or_unsupported_urls() {
        assert_eq!(validate_long_url("/local"), Err(LongUrlError::Invalid));
        assert_eq!(
            validate_long_url("ftp://example.com"),
            Err(LongUrlError::Invalid)
        );
    }
}
