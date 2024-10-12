use std::{collections::HashMap, sync::atomic::AtomicBool};

use caps_shortcut::Key;
use enigo::Keyboard;
use rdev::EventType;
use yaml_rust2::ScanError;

use crate::config::{self, parse_shortcut_key_text, replace_text_config, CapslockForgetFeature, ModifierKey, ShortcutKeyConfigFileFormatError};


static CAPS_SHORTCUT_LISTENER_LOCK: AtomicBool = AtomicBool::new(false);

fn caps_shortcut_listener_is_lock() -> bool {
    CAPS_SHORTCUT_LISTENER_LOCK.load(std::sync::atomic::Ordering::Relaxed)
}

fn set_caps_shortcut_listener_lock(val: bool) {
    CAPS_SHORTCUT_LISTENER_LOCK.store(val, std::sync::atomic::Ordering::Relaxed);
}


#[derive(Debug)]
pub struct MultifunctionalAction;

impl MultifunctionalAction {
    fn replace_text(&self, map: &HashMap<String, String>) {
        caps_shortcut::freeze_listener();
        let text = selection::get_text();
        if let Some(target_text) = map.get(&text) {
            let target_text = target_text.clone();
            std::thread::spawn(move || {
                if let Ok(mut enigo) = enigo::Enigo::new(&enigo::Settings::default()) {
                    let _ = enigo.text(target_text.as_str());
                }
            });
        }
        caps_shortcut::unfreeze_listener();
    }
}


#[derive(Debug, Clone, Copy)]
pub struct InputKey {
    pub key: Key,
    pub modifier_key: ModifierKey,
    pub delay: u64,
}

impl InputKey {
    pub fn with(key: Key, modifier_key: ModifierKey) -> Self {
        Self { 
            key,
            modifier_key,
            delay: 0,
        }
    }

    pub fn with_str(s: &str) -> Result<Self, ShortcutKeyConfigFileFormatError> {
        let (key, modifier_key) = match parse_shortcut_key_text(s) {
            Ok(s) => s,
            Err(err) => return Err(err),
        };
        Ok(Self::with(key, modifier_key))
    }
}

#[derive(Debug)]
pub struct InputKeyAction {
    input_key_list: Vec<InputKey>,
}

impl InputKeyAction {
    pub fn new(input_key_list: Vec<InputKey>) -> Self {
        Self {
            input_key_list,
        }
    }

    pub fn execute(&self) {
        let input_key_list = self.input_key_list.clone();
        set_caps_shortcut_listener_lock(true);
        std::thread::spawn(move || {
            
            fn send_key_event(event: &EventType) {
                if let Err(err) = rdev::simulate(event) {
                    println!("We could not send {:?}", err);
                }
            }

            for input_key in input_key_list {
                let InputKey {key, modifier_key: ModifierKey {ctrl, shift, alt, meta }, delay} = input_key;
                let ctrl_event_press = EventType::KeyPress(Key::ControlLeft);
                let shift_event_press = EventType::KeyPress(Key::ShiftLeft);
                let alt_event_press = EventType::KeyPress(Key::Alt);
                let meta_event_press = EventType::KeyPress(Key::MetaLeft);

                let ctrl_event_release = EventType::KeyRelease(Key::ControlLeft);
                let shift_event_release = EventType::KeyRelease(Key::ShiftLeft);
                let alt_event_release = EventType::KeyRelease(Key::Alt);
                let meta_event_release = EventType::KeyRelease(Key::MetaLeft);

                if ctrl {
                    send_key_event(&ctrl_event_press);
                }

                if shift {
                    send_key_event(&shift_event_press);
                }

                if alt {
                    send_key_event(&alt_event_press);
                }

                if meta {
                    send_key_event(&meta_event_press);
                }
                
                let key_press_event = EventType::KeyPress(key);
                let key_release_event = EventType::KeyRelease(key);

                send_key_event(&key_press_event);
                send_key_event(&key_release_event);

                if ctrl {
                    send_key_event(&ctrl_event_release);
                }

                if shift {
                    send_key_event(&shift_event_release);
                }

                if alt {
                    send_key_event(&alt_event_release);
                }

                if meta {
                    send_key_event(&meta_event_release);
                }
                

                std::thread::sleep(std::time::Duration::from_millis(delay));
            }
            set_caps_shortcut_listener_lock(false);
        });
    }
}

#[derive(Debug)]
pub struct InputTextAction {
    text_list: Vec<String>,
    index: usize,
}

impl InputTextAction {
    pub fn new(text_list: Vec<String>) -> Self {
        Self {
            text_list,
            index: 0,
        }
    }

    pub fn input_next_text(&mut self) {
        let text = self.text_list.get(self.index).cloned().unwrap();
        if self.index >= self.text_list.len() - 1 {
            self.index = 0;
        } else {
            self.index += 1;
        }
        
        std::thread::spawn(move || {
            if let Ok(mut enigo) = enigo::Enigo::new(&enigo::Settings::default()) {
                let _ = enigo.text(text.as_str());
            }
        });
    }
}

#[derive(Debug)]
pub enum LoadError {
    ScanError(ScanError),
    FileNotFound,
    JsonError(serde_json::Error),
    ConfigError(ShortcutKeyConfigFileFormatError),
    ReplaceTextConfigError(ini::Error),
}


pub fn reload() -> Result<(), LoadError> {
    let mut list = vec![];
    if let Ok(item) = config::shortcut_key_config() {
        match item {
            Ok(item) => {
                match item {
                    Some(item) => {
                        match item {
                            Ok(item) => {
                                for c in item {
                                    match c {
                                        Ok(config) => {
                                            list.push(config);
                                        },
                                        Err(err) => {
                                            return Err(LoadError::ConfigError(err))
                                        },
                                    }
                                }
                            },
                            Err(err) => return Err(LoadError::JsonError(err)),
                        }
                    },
                    None => return Err(LoadError::FileNotFound),
                }
            },
            Err(err) => {
                return Err(LoadError::ScanError(err));
            },
        }
    }

    let map = match replace_text_config() {
        Ok(map) => map,
        Err(err) => return Err(LoadError::ReplaceTextConfigError(err)),
    };

    caps_shortcut::clear_all_listener();
    caps_shortcut::caps_listener_with(move |keyboard| {
        if !caps_shortcut_listener_is_lock() {
            for config in list.iter_mut() {
                if config.key == keyboard.key && config.modifier_key.match_key(keyboard.ctrl, keyboard.shift, keyboard.alt, keyboard.meta) {
                    match &mut config.feature {
                        CapslockForgetFeature::InputText(input_text_action) => {
                            input_text_action.input_next_text();
                        },
                        CapslockForgetFeature::Input(input_key_action) => {
                            input_key_action.execute();
                        },
                        CapslockForgetFeature::Multifunctional(multifunctional_action) => {
                            multifunctional_action.replace_text(&map);
                        },
                    }
    
                    return true
                }
            }
        }
        false
    });
    
    Ok(())
}

pub fn run() {
    caps_shortcut::run();
}

pub fn clear() {
    caps_shortcut::clear_all_listener();
}


