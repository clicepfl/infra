use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Package {
    pub name: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    Published { package: Package },
}

#[derive(Deserialize, Debug)]
pub struct Repository {
    pub default_branch: String,
}

#[derive(Deserialize, Debug)]
pub struct Push {
    pub repository: Repository,
    #[serde(rename = "ref")]
    pub r#ref: String,
}
