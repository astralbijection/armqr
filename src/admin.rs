use rocket::{form::Form, response::content::RawHtml, Phase, Rocket, State};
use std::str::FromStr;
use uuid::Uuid;

use crate::{
    config::{Action, Profile},
    Responder,
};
use rocket::{
    form::FromForm,
    http::Status,
    request::{FromRequest, Outcome},
    response::{self, Redirect},
    Request, Response,
};

use askama::Template;

use crate::{config::Config, ArmQRState};

#[derive(Template)]
#[template(path = "admin.html")]
pub struct AdminPage<'a> {
    pub config: &'a Config,
    pub error: Option<&'a str>,
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

impl AdminUser {
    pub fn extract_password(rocket: &Rocket<impl Phase>) -> String {
        #[derive(Deserialize)]
        #[serde(crate = "rocket::serde")]
        struct AdminPassword {
            // Plaintext password. Yes, this is probably fine.
            admin_password: String,
        }

        rocket
            .figment()
            .extract::<AdminPassword>()
            .expect("admin_password was not provided!")
            .admin_password
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = RequiresBasicAuthentication;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let expected_auth = format!(
            "Basic {}",
            base64::encode(format!("admin:{}", Self::extract_password(req.rocket())).as_bytes())
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

#[get("/admin?<error>", rank = 1)]
pub async fn admin_page(
    _admin: AdminUser,
    error: Option<&'_ str>,
    state: &State<ArmQRState>,
) -> RawHtml<String> {
    let page = {
        let lock = state.config.lock().await;
        let config = lock.read();
        AdminPage { config, error }.render().unwrap()
    };
    RawHtml(page)
}

#[derive(FromForm)]
pub struct NewProfileForm<'r> {
    name: Option<&'r str>,
    redirect_uri: &'r str,
}

#[post("/admin/profiles", data = "<form>")]
pub async fn new_profile_form(
    _admin: AdminUser,
    form: Form<NewProfileForm<'_>>,
    state: &State<ArmQRState>,
) -> Redirect {
    if form.redirect_uri.is_empty() {
        return Redirect::to("/admin?error=bad_uri");
    }

    let mut config = {
        let lock = state.config.lock().await;
        lock.read().clone()
    };

    let name = match form.name {
        Some(x) => x.to_string(),
        None => format!("Redirect: {}", form.redirect_uri),
    };
    let id = Uuid::new_v4();
    config.profiles.insert(
        id,
        Profile {
            name,
            action: Action::Redirect(form.redirect_uri.to_string()),
        },
    );

    {
        let mut lock = state.config.lock().await;
        lock.store(config).await;
    }

    Redirect::to("/admin")
}

#[derive(FromForm)]
pub struct ActivateProfileForm<'a> {
    id: &'a str,
}

#[post("/admin/activateProfile", data = "<form>")]
pub async fn activate_profile_form(
    _admin: AdminUser,
    form: Form<ActivateProfileForm<'_>>,
    state: &State<ArmQRState>,
) -> Redirect {
    let uuid = match Uuid::from_str(form.id) {
        Ok(uuid) => uuid,
        Err(_) => return Redirect::to("/admin?error=bad_uuid"),
    };

    let mut config = {
        let lock = state.config.lock().await;
        lock.read().clone()
    };

    if !config.profiles.contains_key(&uuid) {
        return Redirect::to("/admin?error=bad_uuid");
    }

    config.current_profile_id = uuid;

    {
        let mut lock = state.config.lock().await;
        lock.store(config).await;
    }

    Redirect::to("/admin")
}

#[derive(FromForm)]
pub struct DeleteProfileForm<'a> {
    id: &'a str,
}

#[post("/admin/deleteProfile", data = "<form>")]
pub async fn delete_profile_form(
    _admin: AdminUser,
    form: Form<DeleteProfileForm<'_>>,
    state: &State<ArmQRState>,
) -> Redirect {
    let uuid = match Uuid::from_str(form.id) {
        Ok(uuid) => uuid,
        Err(_) => return Redirect::to("/admin?error=bad_uuid"),
    };

    let mut config = {
        let lock = state.config.lock().await;
        lock.read().clone()
    };

    config.profiles.remove(&uuid);

    {
        let mut lock = state.config.lock().await;
        lock.store(config).await;
    }

    Redirect::to("/admin")
}
