use crate::authentication::{get_authorization, JwtClaims, User};
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

#[derive(Clone)]
pub struct JwtSecret(String);

impl JwtSecret {
    pub fn new<S: Into<String>>(secret: S) -> Self {
        Self(secret.into())
    }
}

pub async fn jwt_auth_middleware<B>(
    State(jwt_secret): State<JwtSecret>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    match get_authorization(request.headers()) {
        Some((scheme, credentials)) => {
            if !scheme.eq_ignore_ascii_case("Bearer") {
                return Err(StatusCode::UNAUTHORIZED);
            }

            let token_message = decode::<JwtClaims>(
                credentials,
                &DecodingKey::from_secret(jwt_secret.0.as_ref()),
                &Validation::new(Algorithm::HS256),
            );

            if let Ok(token) = token_message {
                request.extensions_mut().insert::<User>(User {
                    subject: token.claims.subject,
                    role: token.claims.role,
                    country: token.claims.country,
                    currency: token.claims.currency,
                });
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }

            Ok(next.run(request).await)
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
