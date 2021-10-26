#[macro_use]
extern crate rocket;
extern crate dotenv;

use dotenv::dotenv;
use rocket::response;
use rocket::response::Redirect;
use rocket::response::Responder;
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use std::env;

use rocket::http::Status;
use rocket::request;
use rocket::request::FromRequest;
use rocket::request::Outcome;
use rocket::Request;
use rocket::Response;

#[get("/")]
fn index() -> Template {
    let ctx: HashMap<&str, &str> = HashMap::from([(
        "fun_fact",
        "The airspeed velocity of an unladen swallow is 9 meters per second.",
    )]);
    Template::render("linktree", ctx)
}

#[get("/cool-news")]
fn cool_news() -> Redirect {
    // An interesting CNN report
    Redirect::to("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
}

#[derive(Debug)]
struct RequiresBasicAuthentication;

impl<'r> Responder<'r, 'static> for RequiresBasicAuthentication {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Ok(Response::build()
            .status(Status::new(401))
            .raw_header("WWW-Authenticate", "Basic")
            .finalize())
    }
}

struct AdminUser;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = RequiresBasicAuthentication;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
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
fn admin_unauthenticated() -> RequiresBasicAuthentication {
    RequiresBasicAuthentication
}

#[get("/admin", rank = 1)]
fn admin_authenticated(_admin: AdminUser) -> String {
    String::from("yay")
}

fn ensure_environment(key: &str) {
    if env::var(key).is_err() {
        panic!("Required environment variable not provided: {}", key)
    }
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    ensure_environment("ADMIN_USER");
    ensure_environment("ADMIN_PASSWORD");

    rocket::build().attach(Template::fairing()).mount(
        "/",
        routes![index, cool_news, admin_authenticated, admin_unauthenticated],
    )
}
