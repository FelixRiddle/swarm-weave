use std::error::Error;

use jsonwebtoken::{decode, DecodingKey, Algorithm, Validation};
use serde::{Deserialize, Serialize};

use crate::config::env::secret_token;

/// Define the token verification function
/// 
/// 
pub fn verify_token(token: &str) -> Result<bool, Box<dyn Error>> {
    let secret = secret_token();
    
    let secret_key = DecodingKey::from_secret(secret.as_bytes());
    
    decode::<TokenData>(
        token,
        &secret_key,
        &Validation::new(Algorithm::HS256)
    )?;
    
    // Return true if the token is valid
    Ok(true)
}

/// Define the token data struct
#[derive(Debug, Serialize, Deserialize)]
struct TokenData {
    user_id: i64,
    exp: i64,
}
