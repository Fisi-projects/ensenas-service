use anyhow::{Context, Result};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

use crate::user::dtos::FirebaseClaims;

static GOOGLE_CERTS_URL: &str =
    "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| Client::builder().build().unwrap());

#[derive(Debug, Deserialize)]
struct FirebaseJwtClaims {
    sub: String,
    email: String,
    name: String,
    iss: String,
    aud: String,
    exp: usize,
    iat: usize,
}

pub async fn verify_token(id_token: &str) -> Result<FirebaseClaims> {
    // 1. Decode header to find `kid`
    let header = decode_header(id_token).context("Failed to decode token header")?;
    let kid = header.kid.context("Missing `kid` in token header")?;

    // 2. Fetch Google's public keys
    let certs: HashMap<String, String> = HTTP_CLIENT
        .get(GOOGLE_CERTS_URL)
        .send()
        .await
        .context("Failed to fetch Firebase public keys")?
        .json()
        .await
        .context("Failed to parse Firebase public keys")?;

    let public_key_pem = certs
        .get(&kid)
        .context("Matching public key not found for `kid`")?;

    let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
        .context("Invalid PEM in Firebase public key")?;

    // 3. Set up validation
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&["your-firebase-project-id"]); // replace!
    validation.set_issuer(&["https://securetoken.google.com/your-firebase-project-id"]); // replace!

    // 4. Decode token
    let token_data = decode::<FirebaseJwtClaims>(id_token, &decoding_key, &validation)
        .context("Failed to verify Firebase token")?;

    let claims = token_data.claims;

    Ok(FirebaseClaims {
        uid: claims.sub,
        email: claims.email,
        name: claims.name,
    })
}
