use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
};

use rocket::{Phase, Rocket};
use uuid::Uuid;

pub struct ConfigFile {
    path: PathBuf,
    cached: Config,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub current_profile_id: Uuid,
    pub profiles: HashMap<Uuid, Profile>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Profile {
    pub name: String,
    pub action: Action,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Action {
    Redirect(String),
}

impl Config {
    pub fn current_profile(&self) -> &Profile {
        &self.profiles[&self.current_profile_id]
    }
}

impl Default for Config {
    fn default() -> Self {
        let uuid = Uuid::new_v4();
        let mut map = HashMap::new();
        let profile = Profile {
            name: "https://astrid.tech".to_owned(),
            action: Action::Redirect("https://astrid.tech".to_owned()),
        };

        map.insert(uuid, profile);

        Self {
            current_profile_id: uuid,
            profiles: map,
        }
    }
}

impl ConfigFile {
    pub fn extract_from_config(rocket: &Rocket<impl Phase>) -> Self {
        #[derive(Deserialize)]
        #[serde(crate = "rocket::serde")]
        struct ConfigFilePath {
            // Plaintext password. Yes, this is probably fine.
            state_file_path: String,
        }

        let path = PathBuf::from(
            rocket
                .figment()
                .extract::<ConfigFilePath>()
                .expect("state_file_path was not provided!")
                .state_file_path,
        );

        Self::new(path)
    }

    pub fn new(path: PathBuf) -> Self {
        match ConfigFile::read_file(path.as_ref()) {
            Ok(config) => ConfigFile {
                path,
                cached: config,
            },
            Err(_) => {
                use std::fs;

                let config = Config::default();
                let json =
                    serde_json::to_string_pretty(&config).expect("error while building JSON");
                fs::write(&path, json).expect("Failure to write");
                ConfigFile {
                    path,
                    cached: config,
                }
            }
        }
    }

    fn read_file(path: &Path) -> Result<Config, Box<dyn Error>> {
        use std::fs;

        let file = fs::read_to_string(&path)?;
        let json = serde_json::from_str(&file)?;
        Ok(json)
    }

    pub async fn store(&mut self, config: Config) {
        use rocket::tokio::fs;

        self.cached = config;
        let json = serde_json::to_string_pretty(&self.cached).expect("error while building JSON");
        fs::write(&self.path, json).await.expect("Failure to write");
    }

    pub fn read(&self) -> &Config {
        &self.cached
    }
}
