use std::collections::HashMap;

use serde::Deserialize;

use crate::utils::parse_json;

#[derive(Deserialize)]
pub struct LocaleHandler {
    pub localizations: HashMap<String, String>,
    #[serde(skip_deserializing)]
    pub current_locale: String,
    #[serde(skip_deserializing)]
    pub language_data: HashMap<String, String>
}

impl LocaleHandler {
    pub fn new() -> Self {
        parse_json("static/localizations.json").expect("no localization data provieded")
    }

    pub fn set_locale(&mut self, code: String) {
        self.current_locale = code.clone();
        self.language_data = parse_json(&format!("static/{}/general.json", code)).expect("no such language");
    }
}