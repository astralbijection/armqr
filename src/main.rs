mod admin;
mod config;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde;
extern crate dotenv;

use crate::admin::admin_page;
use crate::admin::admin_unauthenticated;
use crate::config::ConfigFile;
use rocket::response::content::Html;
use rocket::tokio::sync::Mutex;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use askama::Template;
use dotenv::dotenv;
use rocket::response::Redirect;
use rocket::response::Responder;

#[get("/")]
fn index() -> Html<String> {
    let fun_fact = "The airspeed velocity of an unladen swallow is 9 meters per second.";
    Html(LinktreeTemplate { fun_fact }.render().unwrap())
}

#[derive(Template)]
#[template(path = "linktree.html")]
struct LinktreeTemplate<'a> {
    fun_fact: &'a str,
}

#[get("/cool-news")]
fn cool_news() -> Redirect {
    // An interesting CNN report
    Redirect::to("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
}

fn ensure_environment(key: &str) {
    if env::var(key).is_err() {
        panic!("Required environment variable not provided: {}", key)
    }
}

pub struct ArmQRState {
    config: Arc<Mutex<ConfigFile>>,
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    ensure_environment("ADMIN_USER");
    ensure_environment("ADMIN_PASSWORD");

    let state = ArmQRState {
        config: Arc::new(Mutex::new(ConfigFile::new(PathBuf::from("./armqr.json")))),
    };

    rocket::build().manage(state).mount(
        "/",
        routes![index, cool_news, admin_page, admin_unauthenticated],
    )
}
