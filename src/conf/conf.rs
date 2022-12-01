use {
    crate::*,
    anyhow::*,
    serde::{Deserialize, Serialize},
    std::{
        collections::{HashMap, HashSet},
        fs,
        io::Write,
        path::PathBuf,
    },
};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Conf {
    pub props: HashMap<String, String>,
    pub watched_users: HashSet<String>,
}

impl Conf {
    pub fn path() -> Result<PathBuf> {
        app_dirs().map(|dirs| dirs.config_dir().join("config.json"))
    }
    /// read the configuration from its standard location
    /// or return the default
    pub fn read() -> Result<Self> {
        let path = Self::path()?;
        if path.exists() {
            let file_content = fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&file_content)?)
        } else {
            Ok(Self::default())
        }
    }
    /// write the conf at its standard location
    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        fs::create_dir_all(path.parent().unwrap())?;
        let mut file = fs::File::create(&path)?;
        let json = serde_json::to_string_pretty(self)?;
        write!(&mut file, "{}", json)?;
        println!("Configuration saved in {:?}.", &path);
        Ok(())
    }
    pub fn set(&mut self, name: String, value: String) {
        self.props.insert(name, value);
    }
    pub fn get(&self, name: &str) -> Option<&str> {
        self.props.get(name).map(|s| s.as_str())
    }
    pub fn follow(&mut self, name: String) {
        self.watched_users.insert(name);
    }
    pub fn unfollow(&mut self, name: &str) {
        self.watched_users.remove(name);
    }
}
