use std::{
    error::Error,
    path::{Path, PathBuf},
};

use uuid::Uuid;

pub struct ConfigFile {
    path: PathBuf,
    cached: Config,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub current_profile: Profile,
    pub profiles: Vec<Profile>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Profile {
    pub id: Uuid,
    pub name: String,
    pub action: Action,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Action {
    Redirect(String),
    Linktree,
}

impl Profile {
    pub fn is_locked(&self) -> bool {
        matches!(self.action, Action::Linktree)
    }
}

impl Default for Config {
    fn default() -> Self {
        let linktree = Profile {
            id: Uuid::new_v4(),
            name: "Linktree".to_string(),
            action: Action::Linktree,
        };
        Self {
            current_profile: linktree.clone(),
            profiles: vec![linktree],
        }
    }
}

impl ConfigFile {
    pub fn new(path: PathBuf) -> Self {
        match ConfigFile::read_file(path.as_ref()) {
            Ok(config) => ConfigFile {
                path,
                cached: config,
            },
            Err(_) => {
                use std::fs;

                let config = Config::default();
                let json = serde_json::to_string_pretty(&config).expect("error while building JSON");
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
