use super::{app_state::AppState, error::Error};
use crate::util::constants::ACCESS_TOKEN;
use anyhow::Result as AnyResult;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rocket::{
    request::{FromRequest, Outcome},
    serde::{Deserialize, Serialize},
    Request,
};

pub trait Token {
    fn encode(&self, secret: &str) -> AnyResult<String>
    where
        Self: Serialize,
    {
        let token_string = encode(
            &Header::default(),
            &self,
            &EncodingKey::from_secret(secret.as_bytes()),
        )?;

        Ok(token_string)
    }

    fn decode(token: &str, secret: &str) -> AnyResult<Self>
    where
        Self: Sized,
        for<'de> Self: Deserialize<'de>,
    {
        let token = decode::<Self>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )?;

        Ok(token.claims)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct AccessToken {
    pub exp: usize,
    pub uid: i64,
    pub permission: i8,
}

impl Token for AccessToken {}

impl AccessToken {
    pub fn new(uid: i64, permission: i8) -> Self {
        let token_expire_days: i64 = 7;
        let expire_time = chrono::Utc::now().timestamp() + token_expire_days * 24 * 60 * 60;

        AccessToken {
            exp: expire_time as usize,
            uid,
            permission,
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AccessToken {
    type Error = Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(token_str) = req.cookies().get(ACCESS_TOKEN) {
            if let Some(state) = req.rocket().state::<AppState>() {
                if let Ok(secret) = state.get_secret() {
                    if let Ok(token) = AccessToken::decode(token_str.value(), &secret) {
                        return Outcome::Success(token);
                    }
                }
            }
        }

        Outcome::Success(AccessToken::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_should_work() {
        let secret = "mySitePassword";
        let claim = AccessToken::new(1, 9);
        let token = claim.encode(secret).unwrap();
        let validate = AccessToken::decode(&token, secret).unwrap();
        assert_eq!(validate.permission, 9);
    }

    #[test]
    #[should_panic]
    fn test_token_should_panic() {
        let mut claim = AccessToken::new(1, 9);
        claim.exp -= 7 as usize * 24 * 60 * 60 + 1;
        let token = claim.encode("secret").unwrap();
        let validate = AccessToken::decode(&token, "secret").unwrap();
        assert_eq!(validate.permission, 9);
    }
}
