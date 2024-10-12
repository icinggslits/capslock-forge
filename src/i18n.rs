use std::{collections::HashMap, sync::{OnceLock, RwLock, RwLockReadGuard}};

use crate::config;


static LANGUAGE_INI_FILE: &str = include_str!("../i18n/i18n.ini");

static CONFIG_COMMENT_TEXT: &str = include_str!("../i18n/config_comment.txt");

static I18N_TEXT: OnceLock<RwLock<I18nText>> = OnceLock::new();

static CONFIG_COMMENT: OnceLock<ConfigCommentText> = OnceLock::new();


pub struct I18nText {
    language: Language,
    data: HashMap<Language, HashMap<String, String>>
}

impl I18nText {
    pub fn new() -> Self {
        use Language::*;
        
        let i18n_ini = ini::Ini::load_from_str(LANGUAGE_INI_FILE).unwrap();
        let mut data = HashMap::new();
        
        for (sec, prop) in i18n_ini.iter() {
            if let Some(language) = sec {
                let language = match language {
                    "Chinese" => Chinese,
                    "Japanese" => Japanese,
                    _ => English,
                };

                let mut map_text = HashMap::new();
                for (k, v) in prop.iter() {
                    map_text.insert(k.to_string(), v.to_string());
                }
                data.insert(language, map_text);
            }
        }
        
        Self {
            language: config::language_or_auto(),
            data,
        }
    }

    pub fn global() -> RwLockReadGuard<'static, Self> {
        let i18n_text = I18N_TEXT.get_or_init(|| RwLock::new(Self::new()) );
        
        if let Ok(language) = config::language() {
            let mut i18n_text = i18n_text.write().unwrap();
            i18n_text.language = language;
        }
        
        i18n_text.read().unwrap()
    }

    fn get(&self, key: &str) -> &String {
        self.data.get(&self.language).unwrap().get(key).unwrap()
    }

    pub fn quit(&self) -> &String {
        self.get("quit")
    }

    pub fn reload(&self) -> &String {
        self.get("reload")
    }
    
}







#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Language {
    Chinese,
    English,
    Japanese,
}

// pub fn get_language() -> Language {
//     let locale = sys_locale::get_locale().unwrap();
//     text_as_language(locale)
// }

// 参考 https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry
// 中文（中国）：zh-CN
// 中文（台湾）：zh-TW
// 英语（美国）：en-US
// 英语（英国）：en-GB
// 日语（日本）：ja-JP
// 法语（法国）：fr-FR
// 德语（德国）：de-DE
// 西班牙语（西班牙）：es-ES
// 韩语（韩国）：ko-KR
// 俄语（俄罗斯）：ru-RU
// 意大利语（意大利）：it-IT
// 葡萄牙语（巴西）：pt-BR
// 阿拉伯语（阿联酋）：ar-AE
// 印地语（印度）：hi-IN

pub fn text_as_language<S: Into<String>>(text: S) -> Language {
    use Language::*;
    let text = text.into();
    let text = text.to_ascii_lowercase();

    fn text_as_language_inner<S: Into<String>>(text: S) -> Result<Language, ()> {
        let text: String = text.into();
        let text = text.to_ascii_lowercase();
        let language = match text.as_str() {
            "zh-cn" | "zh" | "zh-tw" | "zh-hant" => Chinese,
            "ja-jp" | "ja" => Japanese,
            other_text @ _ => {
                if other_text.starts_with("en") {
                    English
                } else {
                    return Err(())
                }
            },
        };
        Ok(language)
    }

    match text_as_language_inner(text) {
        Ok(language) => language,
        Err(_) => {
            let text = sys_locale::get_locale().unwrap();
            text_as_language_inner(text).unwrap_or(English)
        },
    }
}

pub struct ConfigCommentText {
    chinese: String,
    english: String,
}

impl ConfigCommentText {

    fn global() -> &'static Self {
        CONFIG_COMMENT.get_or_init(|| {
            let text = CONFIG_COMMENT_TEXT.to_string();
            let mut record = None::<Language>;
            let mut language_text = Vec::new();

            let mut chinese = String::new();
            let mut english = String::new();

            fn is_markinng_line(line: &str) -> Option<Language> {
                if line.starts_with("[") && line.ends_with("]") {
                    match line.trim() {
                        "[Chinese]" => Some(Language::Chinese),
                        "[English]" => Some(Language::English),
                        _ => None,
                    }
                } else {
                    None
                }
            }

            let mut peekable = text.lines().peekable();

            let mut match_language_text = |language, language_text: &mut Vec<_>| {
                match language {
                    Language::Chinese => {
                        chinese = language_text.join("\n");
                    },
                    _ => {
                        english = language_text.join("\n");        
                    },
                }
            };
            
            while let Some(line) = peekable.next() {
                if peekable.peek().is_none() {
                    if let Some(language) = record {
                        match_language_text(language, &mut language_text);
                    }
                }
                
                if let Some(language) = is_markinng_line(line) {
                    if let Some(old_language) = record {
                        match_language_text(old_language, &mut language_text);
                        language_text.clear();
                        record = Some(language);
                    } else {
                        record = Some(language);
                    }
                    continue;
                }

                if record.is_some() {
                    language_text.push(format!("# {line}"));
                }                
            }


            Self {
                chinese: chinese.trim().to_string(),
                english: english.trim().to_string(),
            }
        })
    }
    
    pub fn chinese() -> &'static String {
        &Self::global().chinese
    }

    pub fn english() -> &'static String {
        &Self::global().english
    }

    pub fn auto_select_language() -> &'static String {
        let language = config::language_or_auto();
        match language {
            Language::Chinese => Self::chinese(),
            _ => Self::english(),
        }
    }

}