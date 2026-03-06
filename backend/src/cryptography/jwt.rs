//! JWT (JSON Web Token) encoding and decoding utilities
//!
//! This module provides functionality for creating, validating, and refreshing
//! JWTs with configurable algorithms.
//! 
//! **Supports:**
//! 
//! * **HS256**
use std::sync::LazyLock;
use chrono::{
    Utc, 
    TimeDelta
};
use serde::{
    Serialize, 
    Deserialize, 
    de::DeserializeOwned
};
use jsonwebtoken::{
    Header, 
    Validation, 
    Algorithm, 
    DecodingKey, 
    EncodingKey
};

pub mod error {

    /// Configuration-related errors that occur during encoder initialization
    #[derive(thiserror::Error, Debug, Clone)]
    pub enum InvalidConfiguration {
        /// Environment variable not found or invalid
        #[error("Invalid Configuration: Environment Error")] Env(#[from] std::env::VarError),
        /// Failed to parse a configuration value (e.g., expiration time)
        #[error("Invalid Configuration: Parsing Error")] ParsingError { key: &'static str },
        /// Unsupported or invalid JWT algorithm specified
        #[error("Invalid Configuration: Invalid Algorithm")] InvalidAlgorithm,
    }

    /// Comprehensive error type for all JWT operations
    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        /// Configuration error; see [`InvalidConfiguration`]
        #[error("JWT Error: {0:?}")]
        InvalidConfiguration(InvalidConfiguration),
        /// Error from the jsonwebtoken crate; see [`jsonwebtoken::errors::Error`]
        #[error("JWT Error: {0:?}")]
        JwtError(#[from] jsonwebtoken::errors::Error),
        /// Serialization/deserialization error
        #[error("Encoding Error: Serialization Error")] 
        SerializationError,
    }

    impl From<serde_json::Error> for Error {
        fn from(_: serde_json::Error) -> Self {
            Self::SerializationError
        }
    }

}

/// Internal JWT claims structure
///
/// Contains the standard JWT claims plus a custom `sub` field that holds
/// the serialized user data.
#[derive(Serialize, Deserialize)]
struct Claims {
    /// Expiration time (seconds since UNIX epoch)
    exp: i64,
    /// Issued at time (seconds since UNIX epoch)
    iat: i64,
    /// Subject claim containing serialized token data
    sub: String,
    /// Issuer claim - who created the token
    iss: String, 
    /// Audience claim - who the token is intended for
    aud: String,
}

/// JWT encoder/decoder configuration
///
/// Supports different algorithms (currently only HS256)
pub enum Encoder {
    /// HMAC-SHA256 based encoder/decoder
    HS256 {
        /// Token issuer (typically your application name/URL)
        issuer: String,
        /// JWT header configuration
        header: Header,
        /// Intended audience for the token
        audience: String,
        /// Token validity duration
        expires: TimeDelta,
        /// Key used for signing tokens
        encoding_key: EncodingKey,
        /// Validation rules for token verification
        validation: Validation,
        /// Key used for verifying tokens
        decoding_key: DecodingKey,
    }
}

impl Encoder {

    /// Creates a new Encoder from environment variables
    ///
    /// Required environment variables:
    /// - `JWT_ALGORITHM`: The algorithm to use (currently only "HS256" supported)
    /// - `JWT_ISSUER`: The token issuer claim
    /// - `JWT_AUDIENCE`: The token audience claim
    /// - `JWT_SECRET`: Secret key for signing (for HS256)
    /// - `JWT_EXPIRES`: Token expiration time in seconds
    ///
    /// # Returns
    /// - `Ok(Encoder)` if all environment variables are valid
    /// - `Err(InvalidConfiguration)` if any variable is missing or invalid
    pub fn from_env() -> Result<Self, error::InvalidConfiguration> {
        let algorithm = std::env::var("JWT_ALGORITHM")?;
        match algorithm.to_lowercase().as_str() {
            "hs256" => {
                let issuer = std::env::var("JWT_ISSUER")?;
                let audience = std::env::var("JWT_AUDIENCE")?;
                let secret = std::env::var("JWT_SECRET")?;
                let expires = TimeDelta::seconds(
                    std::env::var("JWT_EXPIRES")?
                    .parse()
                    .map_err(|_| error::InvalidConfiguration::ParsingError { key: "JWT_EXPIRES" })?
                );
                let header = Header::new(Algorithm::HS256);
                let encoding_key = EncodingKey::from_secret(secret.as_bytes());
                let decoding_key = DecodingKey::from_secret(secret.as_bytes());
                let mut validation = Validation::new(Algorithm::HS256);
                validation.set_issuer(&vec![issuer.clone()]);
                validation.set_audience(&vec![audience.clone()]);
                validation.set_required_spec_claims(&["exp", "aud", "iss", "sub"]);
                Ok(Self::HS256 { issuer, header, audience, expires, encoding_key, validation, decoding_key })
            }
            _ => Err(error::InvalidConfiguration::InvalidAlgorithm)
        }
    }

}

/// Global, lazily-initialized encoder instance from environment variables
///
/// This function provides a singleton encoder that's initialized once
/// on first access.
///
/// # Returns
/// - `Ok(&'static Encoder)` if initialization succeeds
/// - `Err(&'static error::InvalidConfiguration)` if initialization fails
pub fn env_encoder() -> Result<&'static Encoder, &'static error::InvalidConfiguration> {
    static ENCODER: LazyLock<Result<Encoder, error::InvalidConfiguration>> = LazyLock::new(|| Encoder::from_env());
    ENCODER.as_ref()
}

/// Encodes a subject into a JWT
///
/// The subject can be any serializable type. It will be serialized to JSON
/// and stored in the token's `sub` claim.
///
/// # Arguments
/// * `subject` - The data to encode in the token
///
/// # Returns
/// * `Ok(String)` - The encoded JWT as a string
/// * `Err(error::Error)` - If encoding fails
pub fn encode<'subject, Subject>(subject: &'subject Subject) -> Result<String, error::Error> 
where Subject: Serialize
{
    let encoder = env_encoder().map_err(|err| error::Error::InvalidConfiguration(err.clone()))?;
    match encoder {
        Encoder::HS256 { issuer, header, audience, expires, encoding_key, .. } => {
            let claims = Claims {
                sub: serde_json::to_string(subject)?,
                exp: (Utc::now() + *expires).timestamp(),
                iat: (Utc::now()).timestamp(),
                iss: issuer.into(),
                aud: audience.into()
            };
            Ok(jsonwebtoken::encode(
                header, &claims, encoding_key
            )?)
        }
    }
}

/// Decodes and validates a JWT, extracting the subject
///
/// # Arguments
/// * `token` - The JWT string or byte slice to decode
///
/// # Returns
/// * `Ok(Subject)` - The decoded subject data
/// * `Err(error::Error)` - If decoding or validation fails
pub fn decode<'token, Subject, Token: AsRef<[u8]>>(token: Token) -> Result<Subject, error::Error>
where Subject: DeserializeOwned
{
    let encoder = env_encoder().map_err(|err| error::Error::InvalidConfiguration(err.clone()))?;
    match encoder {
        Encoder::HS256 { validation, decoding_key, .. } => {
            let data = jsonwebtoken::decode::<Claims>(token, decoding_key, validation)?;
            Ok(serde_json::from_str::<Subject>(&data.claims.sub)?)
        }
    }
}

/// Decodes a JWT and optionally refreshes it if expiration is near
///
/// This function checks the token's remaining lifetime. If it's less than or
/// equal to the provided threshold, a new token is generated.
///
/// # Arguments
/// * `token` - The JWT string or byte slice to decode
/// * `lt` - Time threshold for refresh (if remaining time ≤ this, refresh)
///
/// # Returns
/// * `Ok((Subject, Option<String>))` - The decoded subject and an optional
///   new token if refresh was triggered
/// * `Err(error::Error)` - If decoding fails
pub fn decode_with_refresh<'token, Subject, Token: AsRef<[u8]>>(token: Token, lt: &TimeDelta) -> Result<(Subject, Option<String>), error::Error>
where Subject: DeserializeOwned + Serialize
{
    let encoder = env_encoder().map_err(|err| error::Error::InvalidConfiguration(err.clone()))?;
    match encoder {
        Encoder::HS256 { validation, decoding_key, .. } => {
            let data = jsonwebtoken::decode::<Claims>(token, decoding_key, validation)?;
            let subject = serde_json::from_str::<Subject>(&data.claims.sub)?;
            let refresh: Option<String> = if let Some(expires) = chrono::DateTime::from_timestamp(data.claims.exp, 0) {
                let remaining = expires.signed_duration_since(&Utc::now());
                if remaining <= *lt {
                    Some(encode(&subject)?)
                } else {
                    None
                }
            } else {
                None
            };
            Ok((subject, refresh))
        }
    }
}
