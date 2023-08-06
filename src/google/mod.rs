use google_sheets4::oauth2;
use std::{collections::HashMap, env};
use url::Url;

pub struct GoogleAuth {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    auth_url: String,
    token_url: String,
    scope: String,
}

impl GoogleAuth {
    pub fn new() -> Self {
        Self {
            client_id: env::var("GOOGLE_SHEETS_CLIENT_ID").expect("GOOGLE_SHEETS_CLIENT_ID"),
            client_secret: env::var("GOOGLE_SHEETS_CLIENT_SECRET")
                .expect("GOOGLE_SHEETS_CLIENT_SECRET"),
            redirect_uri: env::var("GOOGLE_SHEETS_REDIRECT_URI")
                .expect("GOOGLE_SHEETS_REDIRECT_URI"),
            auth_url: String::from("https://accounts.google.com/o/oauth2/v2/auth"),
            token_url: String::from("https://oauth2.googleapis.com/token"),
            scope: "https://www.googleapis.com/auth/spreadsheets".to_owned(),
        }
    }

    pub fn login_url(&self) -> Result<Url, url::ParseError> {
        let mut url = Url::parse(&self.auth_url)?;

        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("client_id".to_string(), self.client_id.clone());
        params.insert("response_type".to_string(), "code".to_string());
        params.insert("scope".to_string(), self.scope.clone());
        params.insert("redirect_uri".to_string(), self.redirect_uri.clone());
        params.insert("access_type".to_string(), "offline".to_string());
        params.insert("prompt".to_string(), "consent".to_string());

        url.query_pairs_mut().extend_pairs(params);

        Ok(url)
    }

    pub fn to_app_secret(&self) -> oauth2::ApplicationSecret {
        oauth2::ApplicationSecret {
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            ..Default::default()
        }
    }
}
