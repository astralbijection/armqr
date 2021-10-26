#[macro_use]
extern crate rocket;

use rocket::http::Status;
use rocket::Response;
use rocket::Request;
use rocket::Route;
use rocket::http::Method;
use rocket::response::status;
use rocket::response::{self, Responder};
use rocket_basicauth::BasicAuth;

#[get("/")]
fn index() -> String {
    String::from("test")
}

struct RequiresBasicAuthentication;

impl<'r> Responder<'r, 'static> for RequiresBasicAuthentication {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Ok(Response::build()
            .status(Status::new(401))
            .raw_header("WWW-Authenticate", r#"Basic"#)
            .finalize())
    }
}


#[get("/admin", rank=2)]
fn admin_unauthenticated() -> RequiresBasicAuthentication {
    RequiresBasicAuthentication
}

#[get("/admin", rank=1)]
fn admin_authenticated(auth: BasicAuth) -> String {
    String::from(auth.username)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, admin_authenticated, admin_unauthenticated])
}
