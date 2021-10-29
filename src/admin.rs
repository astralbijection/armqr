use std::env;

use crate::Responder;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    response, Request, Response,
};

use askama::Template;
use rocket::{response::content::Html, State};

use crate::{config::Config, ArmQRState};

#[derive(Template)]
#[template(path = "admin.html")]
pub struct AdminPage<'a> {
    pub config: &'a Config,
}

#[derive(Debug)]
pub struct RequiresBasicAuthentication;

impl<'r> Responder<'r, 'static> for RequiresBasicAuthentication {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Ok(Response::build()
            .status(Status::new(401))
            .raw_header("WWW-Authenticate", "Basic")
            .finalize())
    }
}

pub struct AdminUser;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = RequiresBasicAuthentication;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let expected_auth = format!(
            "Basic {}",
            base64::encode(
                format!(
                    "{}:{}",
                    env::var("ADMIN_USER").unwrap(),
                    // Yes, the password is plaintext. Yes, I use a password manager.
                    env::var("ADMIN_PASSWORD").unwrap()
                )
                .as_bytes()
            )
        );

        if let Some(header) = req.headers().get_one("Authorization") {
            if header.trim() == expected_auth {
                return Outcome::Success(AdminUser);
            }
            return Outcome::Failure((Status::Forbidden, RequiresBasicAuthentication));
        }

        Outcome::Forward(())
    }
}

#[get("/admin", rank = 2)]
pub fn admin_unauthenticated() -> RequiresBasicAuthentication {
    RequiresBasicAuthentication
}

#[get("/admin", rank = 1)]
pub async fn admin_page(_admin: AdminUser, state: &State<ArmQRState>) -> Html<String> {
    let page = {
        let lock = state.config.lock().await;
        let config = lock.read();
        AdminPage { config }.render().unwrap()
    };
    Html(page)
}
