use hmac::Mac;

use crate::credentials::error::SigningRelatedError;
pub mod error;
pub mod header;
pub mod provider;
pub mod signing;
pub type Hmac256 = hmac::Hmac<sha2::Sha256>;
pub fn sha256_from_bytes(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
pub fn sign_content(content: &str, signing_key: &[u8]) -> Result<String, SigningRelatedError> {
    let mut mac = Hmac256::new_from_slice(signing_key)?;
    mac.update(content.as_bytes());
    let result = mac.finalize();
    let signature = hex::encode(result.into_bytes());
    Ok(signature)
}
