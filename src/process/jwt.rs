use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    aud: String,
    exp: usize,
}

impl Claims {
    pub fn new(sub: String, aud: String, exp: usize) -> Self {
        Self { sub, aud, exp }
    }
}

pub fn process_jwt_sign(sub: &str, aud: &str, exp: usize, secret: &str) -> anyhow::Result<String> {
    let claims = Claims::new(sub.to_string(), aud.to_string(), exp);
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;
    Ok(token)
}

pub fn process_jwt_verify(token: &str, secret: &str) -> bool {
    let mut validation = Validation::default();
    validation.validate_aud = false;
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    );
    match token_data {
        Ok(token) => {
            println!("{:?}", token.claims);
            true
        }
        Err(e) => {
            println!("{:?}", e);
            false
        }
    }
}
