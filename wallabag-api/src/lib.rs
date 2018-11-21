// std libs
use std::collections::HashMap;

// crates
use reqwest;
use serde_derive::Deserialize;

pub struct API {
    auth_info: AuthInfo,
}

impl API {
    pub fn new(config: Config) -> Self {
        API {
            auth_info: config.auth_info,
        }
    }

    fn retrieve_access_token(&self) -> Result<AccessTokenResponse, reqwest::Error> {
        let base_url = "https://framabag.org";
        let path = "/oauth/v2/token";
        let url = format!("{}{}", base_url, path);

        let mut fields = HashMap::new();

        fields.insert("grant_type", "password");
        fields.insert("client_id", &self.auth_info.client_id);
        fields.insert("client_secret", &self.auth_info.client_secret);
        fields.insert("username", &self.auth_info.username);
        fields.insert("password", &self.auth_info.password);

        let client = reqwest::Client::new();
        let mut res = client.post(&url).json(&fields).send()?;

        let token_info: AccessTokenResponse = res.json()?;

        Ok(token_info)
    }

    pub fn get_entries(&self) -> Result<Entries, reqwest::Error> {
        let token = self.retrieve_access_token()?.access_token;

        let base_url = "https://framabag.org";
        let path = "/api/entries.json";
        let url = format!("{}{}", base_url, path);

        let client = reqwest::Client::new();
        let mut res = client
            .get(&url)
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
            .send()?;

        println!("{:?}", res.text()?);

        Ok(vec![])
    }
}

pub type Entries = Vec<Entry>;

#[derive(Deserialize, Debug)]
pub struct Entry {
    todo: String,
}

#[derive(Deserialize, Debug)]
struct AccessTokenResponse {
    access_token: String,
    expires_in: u32,
    token_type: String,
    scope: Option<String>,
    refresh_token: String,
}

#[derive(Debug)]
pub struct AuthInfo {
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct Config {
    pub auth_info: AuthInfo,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
