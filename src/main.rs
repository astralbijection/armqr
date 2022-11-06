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
use admin::AdminUser;
use config::Action;
use rocket::tokio::sync::Mutex;
use rocket::State;
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
    }
}

#[derive(Template)]
#[template(path = "redirect.html")]
struct RedirectTemplate<'a> {
    escaped_url: &'a str,
}

pub struct ArmQRState {
    config: Arc<Mutex<ConfigFile>>,
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    let state = ArmQRState {
        config: Arc::new(Mutex::new(ConfigFile::new(PathBuf::from("./armqr.json")))),
    };

    let rocket = rocket::build();
    AdminUser::extract_password(&rocket);
    rocket.manage(state).mount(
        "/",
        routes![
            index,
            admin_page,
            admin_unauthenticated,
            new_profile_form,
            activate_profile_form,
            delete_profile_form,
        ],
    )
}
