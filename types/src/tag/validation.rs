use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TagError {
    #[error("Invalid tag key: {0}")]
    InvalidKey(String),
    #[error("Invalid tag value: {0}")]
    InvalidValue(String),
}
pub fn validate_tag_key(key: &str) -> Result<(), TagError> {
    if key.is_empty() || key.len() > 128 {
        return Err(TagError::InvalidKey(key.to_string()));
    }
    if !key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(TagError::InvalidKey(key.to_string()));
    }
    Ok(())
}
pub fn validate_tag_value(value: &str) -> Result<(), TagError> {
    if value.is_empty() || value.len() > 256 {
        return Err(TagError::InvalidValue(value.to_string()));
    }
    Ok(())
}
