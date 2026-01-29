use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

use crate::identity::claims::Claims;
use crate::identity::user::UserIdentity;

pub trait TokenService: Send + Sync {
    fn create_access_token(&self, user: &UserIdentity) -> Result<String>;

    fn validate_access_token(&self, token: &str) -> Result<Claims>;

    fn create_refresh_token(&self, user_id: &str) -> Result<String>;

    fn validate_refresh_token(&self, token: &str) -> Result<String>;

    fn revoke_refresh_token(&self, token: &str) -> Result<()>;
}

pub struct JwtTokenService {
    secret: String,
    issuer: String,
    access_token_expiration_minutes: i64,
    refresh_token_expiration_days: i64,
}

impl JwtTokenService {
    pub fn new(secret: String, issuer: String) -> Self {
        Self {
            secret,
            issuer,
            access_token_expiration_minutes: 60,
            refresh_token_expiration_days: 7,
        }
    }

    pub fn with_expiration(mut self, access_min: i64, refresh_days: i64) -> Self {
        self.access_token_expiration_minutes = access_min;
        self.refresh_token_expiration_days = refresh_days;
        self
    }
}

impl TokenService for JwtTokenService {
    fn create_access_token(&self, user: &UserIdentity) -> Result<String> {
        let iat = Utc::now();
        let exp = iat + Duration::minutes(self.access_token_expiration_minutes);

        let mut claims = user.claims().clone();
        claims.sub = Some(user.id().to_string());
        claims.iat = Some(iat.timestamp() as usize);
        claims.exp = Some(exp.timestamp() as usize);
        claims.iss = Some(self.issuer.clone());

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )?;
        Ok(token)
    }

    fn validate_access_token(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.issuer]);

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &validation,
        )?;

        Ok(token_data.claims)
    }

    fn create_refresh_token(&self, user_id: &str) -> Result<String> {
        let iat = Utc::now();
        let exp = iat + Duration::days(self.refresh_token_expiration_days);

        let mut claims = Claims::new();
        claims.sub = Some(user_id.to_string());
        claims.iat = Some(iat.timestamp() as usize);
        claims.exp = Some(exp.timestamp() as usize);
        claims.iss = Some(format!("{}-refresh", self.issuer));

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )?;
        Ok(token)
    }

    fn validate_refresh_token(&self, token: &str) -> Result<String> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&format!("{}-refresh", self.issuer)]);

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &validation,
        )?;

        token_data
            .claims
            .sub
            .ok_or_else(|| anyhow!("Refresh token missing sub claim"))
    }

    fn revoke_refresh_token(&self, _token: &str) -> Result<()> {
        Ok(())
    }
}
