use actix_web::HttpRequest;
use std::fmt::Display;

#[derive(Debug)]
pub struct AuthToken(pub String);

impl Display for AuthToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AuthToken:{}", self.0)
    }
}

impl TryFrom<HttpRequest> for AuthToken {
    type Error = &'static str;
    fn try_from(req: HttpRequest) -> Result<Self, Self::Error> {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok());
        if let Some(auth_value) = auth_header {
            if auth_value.starts_with("Bearer ") {
                return Ok(AuthToken(auth_value[7..].to_string()));
            }
        }
        Err("No Authorization header found")
    }
}
