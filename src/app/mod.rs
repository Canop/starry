use {
    anyhow::*,
    directories_next::ProjectDirs,
};

/// return the instance of ProjectDirs holding the app specific paths
pub fn app_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("org", "dystroy", "starry")
        .context("Unable to find app directories")
}
