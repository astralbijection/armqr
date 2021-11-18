mod admin;
mod config;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde;
extern crate dotenv;

use crate::admin::activate_profile_form;
use crate::admin::admin_page;
use crate::admin::admin_unauthenticated;
use crate::admin::delete_profile_form;
use crate::admin::new_profile_form;
use crate::config::ConfigFile;
use config::Action;
use rocket::response::content::Html;
use rocket::tokio::sync::Mutex;
use rocket::State;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use askama::Template;
use dotenv::dotenv;
use rocket::response::Redirect;
use rocket::response::Responder;

#[get("/")]
async fn index(state: &State<ArmQRState>) -> Redirect {
    let profile = {
        let lock = state.config.lock().await;
        lock.read().current_profile().clone()
    };

    match profile.action {
        Action::Redirect(uri) => Redirect::to(uri),
        Action::Linktree => Redirect::to("/landing"),
    }
}

#[get("/landing")]
fn linktree() -> Html<String> {
    let fun_fact = "The airspeed velocity of an unladen swallow is 9 meters per second.";
    Html(LinktreeTemplate { fun_fact }.render().unwrap())
}

#[derive(Template)]
#[template(path = "linktree.html")]
struct LinktreeTemplate<'a> {
    fun_fact: &'a str,
}

#[get("/cool-news")]
fn cool_news() -> Html<String> {
    Html(
        RedirectTemplate {
            // An interesting CNN report
            escaped_url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
        }
        .render()
        .unwrap(),
    )
}

#[derive(Template)]
#[template(path = "redirect.html")]
struct RedirectTemplate<'a> {
    escaped_url: &'a str,
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
        routes![
            index,
            linktree,
            cool_news,
            admin_page,
            admin_unauthenticated,
            new_profile_form,
            activate_profile_form,
            delete_profile_form,
        ],
    )
}
