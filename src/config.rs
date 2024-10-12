use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use caps_shortcut::Key;
use key_from_str::KeyFromStr;
use serde_json::Value;
use yaml_rust2::{Yaml, YamlLoader};

use crate::{feature::{InputKey, InputKeyAction, InputTextAction, MultifunctionalAction}, i18n::{self, text_as_language, Language}, units::{file_io, string::TrimCharMatches}};

static CONFIG_DIR: &str = "config";
static CAPSLOCK_FORGET_CONFIG_FILE_NAME: &str = "capslock_forget_config.yaml";
static REPLACE_TEXT_FIEL_NAME: &str = "replace_text.ini";
static DEFAULT_CAPSLOCK_FORGET_CONFIG_BYTE: &[u8] = include_bytes!("../default_config/capslock_forget_config.yaml");
static DEFAULT_REPLACE_TEXT_BYTE: &[u8] = include_bytes!("../default_config/replace_text.ini");

mod key_from_str;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModifierKey {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

impl ModifierKey {
    pub fn match_key(&self, ctrl: bool, shift: bool, alt: bool, meta: bool) -> bool {
        self.ctrl == ctrl && self.shift == shift && self.alt == alt && self.meta == meta
    }

    pub fn new() -> Self {
        Self { 
            ctrl: false,
            shift: false,
            alt: false,
            meta: false
        }
    }
}

#[derive(Debug)]
pub struct ShortcutKeyConfig {
    pub key: Key,
    pub modifier_key: ModifierKey,
    pub feature: CapslockForgetFeature,
}

impl ShortcutKeyConfig {
    fn from_entry(entry: &Value) -> Result<Self, ShortcutKeyConfigFileFormatError> {
        let (key, modifier_key) = match entry["key"].as_str() {
            Some(s) => {
                parse_shortcut_key_text(s)?
            }
            None => return Err(ShortcutKeyConfigFileFormatError::JsonError(entry.to_string()))
        };

        let feature = CapslockForgetFeature::from_value(entry)?;
        
        Ok(Self {
            key,
            modifier_key,
            feature
        })
    }
}

#[derive(Debug)]
pub enum CapslockForgetFeature {
    InputText(InputTextAction),
    Input(InputKeyAction),
    Multifunctional(MultifunctionalAction),
}

impl CapslockForgetFeature {
    fn from_value(value: &Value) -> Result<Self, ShortcutKeyConfigFileFormatError> {

        fn to_string_list(value: &Value) -> Result<Vec<String>, ShortcutKeyConfigFileFormatError> {
            let mut text_list = vec![];
            if let Some(array) = value.as_array() {
                let array = array.iter().map(|v| v.to_string().trim_char_matches("\"").to_string());
                text_list.extend(array);
            } if let Some(value) = value.as_str() {
                text_list.push(value.to_string());
            }
            if text_list.is_empty() {
                return Err(ShortcutKeyConfigFileFormatError::ValueError(value.to_string()))
            }
            Ok(text_list)
        }

        match value["feature"].as_str() {
            Some(feature) => {
                match feature {
                    "input_text" => {
                        let text_list = to_string_list(&value["text"])?;
                        let text_list = InputTextAction::new(text_list);
                        Ok(Self::InputText(text_list))
                    }

                    "input" => {
                        let action_list = to_string_list(&value["action"])?;
                        let delay = value["delay"].as_u64().unwrap_or(0);

                        let mut list = vec![];

                        for action in action_list {
                            let mut action_key = if let Ok(action_key) = InputKey::with_str(&action) { action_key } else {
                                return Err(ShortcutKeyConfigFileFormatError::ValueError(action.to_string()))
                            };
                            action_key.delay = delay;

                            list.push(action_key);    
                        }
                        Ok(Self::Input(InputKeyAction::new(list)))
                    }

                    "multifunctional" => {
                        Ok(Self::Multifunctional(MultifunctionalAction))
                    }
                    
                    _ => return Err(ShortcutKeyConfigFileFormatError::FeatureError(feature.to_string())),
                }
            },
            None => return Err(ShortcutKeyConfigFileFormatError::JsonError(value.to_string())),
        }
    }
}

#[derive(Debug)]
pub enum ShortcutKeyConfigFileFormatError {
    JsonError(String),
    KeyError(String),
    ValueError(String),
    FeatureError(String),
}

impl std::fmt::Display for ShortcutKeyConfigFileFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "config file illegal")
    }
}



pub fn capslock_forget_config_file_path() -> PathBuf {
    let current_exe = std::env::current_exe().unwrap();
    let current_dir = current_exe.parent().unwrap();
    let config_dir = current_dir.join(CONFIG_DIR);
    config_dir.join(CAPSLOCK_FORGET_CONFIG_FILE_NAME)
}

pub fn replace_text_file_path() -> PathBuf {
    let current_exe = std::env::current_exe().unwrap();
    let current_dir = current_exe.parent().unwrap();
    let config_dir = current_dir.join(CONFIG_DIR);
    config_dir.join(REPLACE_TEXT_FIEL_NAME)
}

pub fn init() {
    let capslock_forge_config_file_path = capslock_forget_config_file_path();
    if !capslock_forge_config_file_path.is_file() {
        let mut bytes = Vec::from(i18n::ConfigCommentText::auto_select_language().as_bytes());
        bytes.extend(b"\n");
        bytes.extend(DEFAULT_CAPSLOCK_FORGET_CONFIG_BYTE);
        let _ = file_io::write(capslock_forge_config_file_path, bytes);
    }
    
    let replace_text_file_path = replace_text_file_path();
    if !replace_text_file_path.is_file() {
        let _ = file_io::write(replace_text_file_path, DEFAULT_REPLACE_TEXT_BYTE);
    }
}

fn parse_yaml_file(path: PathBuf) -> Result<Result<Option<Vec<Yaml>>, yaml_rust2::scanner::ScanError>, std::io::Error> {
    let mut text = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut text)?;
    let yaml_item_list = match YamlLoader::load_from_str(&*text) {
        Ok(yaml_item_list) => yaml_item_list,
        Err(err) => return Ok(Err(err)),
    };
    if yaml_item_list.is_empty() {
        return Ok(Ok(None));
    }
    Ok(Ok(Some(yaml_item_list)))
}


pub fn language() -> Result<Language, ()> {
    let capslock_forge_config_file_path = capslock_forget_config_file_path();
    let yaml_list = parse_yaml_file(capslock_forge_config_file_path).unwrap_or(Ok(Some(vec![]))).unwrap_or(Some(vec![])).unwrap_or(vec![]);
    if yaml_list.is_empty() {
        return Err(())
    }

    let yaml = &yaml_list[0];

    let language = match yaml["language"].as_str() {
        Some(language) => text_as_language(language),
        None => return Err(()),
    };
    
    Ok(language)
}

pub fn language_or_auto() -> Language {
    match language() {
        Ok(language) => language,
        Err(_) => text_as_language("auto"),
    }
}

pub fn replace_text_config() -> Result<HashMap<String, String>, ini::Error> {
    let replace_text_file_path = replace_text_file_path();

    let mut map = HashMap::new();
    let ini = ini::Ini::load_from_file(replace_text_file_path)?;

    let mut insert_and_check_repeat_key = |k: String, v: String| {
        if let Some(old_key) = map.insert(k, v) {
            eprintln!("先前存在的值: {}", old_key)
        }
    };

    for (sec, prop) in ini.iter() {
        if let Some(sec) = sec {
            if sec == "Multifunctional" {
                for (k, v) in prop.iter() {
                    if v.starts_with("[") && v.ends_with("]") {
                        let v_list = v.trim_start_matches("[").trim_end_matches("]").split(",").collect::<Vec<_>>();
                        if v_list.len() > 1 {
                            let mut v_list_iter = v_list.iter().peekable();
                            let head = v_list_iter.peek().unwrap().trim().to_string();
                            // println!("{} -> {}", k, head.clone());
                            // map.insert(k.to_string(), head.clone());
                            insert_and_check_repeat_key(k.to_string(), head.clone());
                            
                            while let Some(v) = v_list_iter.next() {
                                if let Some(v_next) = v_list_iter.peek() {
                                    // println!("{} -> {}", v.trim().to_string(), v_next.trim().to_string());
                                    // map.insert(v.trim().to_string(), v_next.trim().to_string());
                                    insert_and_check_repeat_key(v.trim().to_string(), v_next.trim().to_string());

                                } else {
                                    // println!("{} -> {}", v.trim().to_string(), head.clone());
                                    // map.insert(v.trim().to_string(), head.clone());
                                    insert_and_check_repeat_key(v.trim().to_string(), head.clone());
                                }
                            }
                        } else {
                            // println!("{} -> {}", k, v);
                            // map.insert(k.to_string(), v.to_string());
                            insert_and_check_repeat_key(k.to_string(), v.to_string());
                        }
                    } else {
                        // println!("{} -> {}", k, v);
                        // map.insert(k.to_string(), v.to_string());
                        insert_and_check_repeat_key(k.to_string(), v.to_string());
                    }
                }

                break;
            }
        }
    }

    Ok(map)
}

pub fn shortcut_key_config() -> Result<Result<Option<Result<Vec<Result<ShortcutKeyConfig, ShortcutKeyConfigFileFormatError>>, serde_json::Error>>, yaml_rust2::scanner::ScanError>, std::io::Error> {
    let capslock_forget_config_file_path = capslock_forget_config_file_path();
    
    let yaml = match parse_yaml_file(capslock_forget_config_file_path)? {
        Ok(yaml) => {
            match yaml {
                Some(yaml) => yaml,
                None => return Ok(Ok(None)),
            }
        },
        Err(err) => return Ok(Err(err)),
    };

    let yaml = &yaml[0];

    match yaml["capslock_shortcut"].as_str() {
        Some(capslock_shortcut_json) => {
        
            let mut shortcut_key_config_list = vec![];
            
            let json: Value = match serde_json::from_str(capslock_shortcut_json) {
                Ok(json) => json,
                   Err(err) => return Ok(Ok(Some(Err(err)))),
            };
        
            let json = match json.as_array() {
                Some(json) => json,
                None => return Ok(Ok(Some(Ok(Vec::new())))),
            };
        
            for entry in json {
                let config_entry = ShortcutKeyConfig::from_entry(entry);
                shortcut_key_config_list.push(config_entry);
            }

            Ok(Ok(Some(Ok(shortcut_key_config_list))))
        },
        None => return Ok(Ok(None)),
    }
}


pub fn parse_shortcut_key_text<S: Into<String>>(s: S) -> Result<(Key, ModifierKey), ShortcutKeyConfigFileFormatError> {
    let s: String = s.into();
    let s = &*s;
    let mut key_list = vec![];

    let original_key_str = s.to_string();
    let key_str_list = s.split("+").map(|s| s.trim()).collect::<Vec<_>>();

    let mut modifier_key = ModifierKey::new();
    
    let mut key = None;

    if key_str_list.len() == 1 {
        match Key::from_str(key_str_list[0]) {
            Ok(key_) => key = Some(key_),
            Err(_) => return Err(ShortcutKeyConfigFileFormatError::KeyError(key_str_list[0].to_string())),
        }
    } else {
        for key_str in key_str_list {
            let key = {
                match key_str.to_ascii_lowercase().as_str() {
                    "ctrl" => modifier_key.ctrl = true,
                    "shift" => modifier_key.shift = true,
                    "alt" => modifier_key.alt = true,
                    "meta" => modifier_key.meta = true,
                    other_key => {
                        match Key::from_str(other_key) {
                            Ok(key_) => {
                                key = Some(key_);
                            },
                            Err(_) => {
                                return Err(ShortcutKeyConfigFileFormatError::KeyError(original_key_str))
                            },
                        }
                        break;
                    },
                };
            };
    
            key_list.push(key);
        }
    
        if key.is_none() {
            return Err(ShortcutKeyConfigFileFormatError::KeyError(original_key_str));
        }
    }

    Ok((key.unwrap(), modifier_key))
}





