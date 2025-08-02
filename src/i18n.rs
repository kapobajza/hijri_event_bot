use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Language {
    Ba,
}

#[derive(Deserialize, Clone)]
struct Messages {
    messages: HashMap<String, String>,
}

#[derive(Clone)]
pub struct I18n {
    translations: HashMap<Language, Messages>,
    current_language: Language,
}

impl I18n {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut translations = HashMap::new();

        let ba_content = include_str!("../locales/ba.toml");
        let ba_messages: Messages = toml::from_str(ba_content)?;

        let current_language = Language::Ba;

        translations.insert(current_language.clone(), ba_messages);

        Ok(Self {
            translations,
            current_language,
        })
    }

    #[allow(dead_code)]
    pub fn set_language(&mut self, language: Language) {
        self.current_language = language;
    }

    pub fn t(&self, key: &str) -> String {
        let translations = self.translations.get(&self.current_language);

        if let Some(translations) = translations {
            return translations
                .messages
                .get(key)
                .cloned()
                .unwrap_or(key.to_string());
        }

        key.to_string()
    }

    pub fn t_with_args(&self, key: &str, args: HashMap<&str, String>) -> String {
        let mut translation = self.t(key);

        for (k, v) in args {
            translation = translation.replace(&format!("{{{}}}", k), v.as_str());
        }

        translation
    }
}
