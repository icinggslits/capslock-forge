use std::sync::mpsc::Receiver;

use tray_item::{IconSource, TrayItem};

use crate::{feature, i18n::I18nText};


pub enum TrayEndEvent {
    Exit,
    Reload,
}

pub struct Tray {
    tray: TrayItem,
    reload_id: u32,
    quit_id: u32,
    // tx: SyncSender<Message>,
    rx: Receiver<Message>,
    reload_cb: Option<Box<dyn FnMut()>>,
}

impl Tray {
    pub fn new() -> Self {
        let mut tray = TrayItem::new(
            "CapsLock Forge",
            IconSource::Resource("app-icon"),
        ).unwrap();
        
        let tray_inner = tray.inner_mut();
        
        let i18n_text = I18nText::global();
        
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        
        let reload_id = {
            let tx = tx.clone();
            tray_inner.add_menu_item_with_id(&i18n_text.reload(), move || {
                let _ = tx.send(Message::Reload);
            }).unwrap()
        };
        
        let quit_id = {
            let tx = tx.clone();
            tray_inner.add_menu_item_with_id(&i18n_text.quit(), move || {
                let _ = tx.send(Message::Quit);
            }).unwrap()
        };

        {   
            match feature::reload() {
                Ok(_) => {
                    let _handle = std::thread::spawn(move || {
                        loop {
                            println!("启动");
                            // 大概因为监控时又使用控制输入的各种库，有时run()会停止，所以需要重启
                            feature::run();
                        }
                    });
                },
                Err(err) => {
                    println!("Reload Error: {:?}", err);
                    tray.set_icon(IconSource::Resource("app-config-error-icon")).unwrap();
                },
            }
        }
        
        Self {
            tray,
            reload_id,
            quit_id,
            rx,
            reload_cb: None,
        }
    }

    pub fn run(&mut self) -> TrayEndEvent {
        let mut event = TrayEndEvent::Exit;
        loop {
            match self.rx.recv() {
                Ok(Message::Reload) => {
                    println!("Reload");
                    event = TrayEndEvent::Reload;
                    if let Some(reload_cb) = self.reload_cb.as_mut() {
                        reload_cb();
                    }
                    self.reload();
                }
                Ok(Message::Quit) => {
                    break;
                }
                _ => ()
            }
        }
        event
    }

    fn reload(&mut self) {
        let i18n_text = I18nText::global();
        let tray = self.tray.inner_mut();
        let _ = tray.set_menu_item_label(&i18n_text.reload(), self.reload_id);
        let _ = tray.set_menu_item_label(&i18n_text.quit(), self.quit_id);

        match feature::reload() {
            Ok(_) => {
                tray.set_icon(IconSource::Resource("app-icon")).unwrap();
            },
            Err(err) => {
                println!("Reload Error: {:?}", err);
                feature::clear();
                tray.set_icon(IconSource::Resource("app-config-error-icon")).unwrap();
            },
        }
    }

    pub fn reload_with(&mut self, reload_cb: Box<dyn FnMut()>) {
        self.reload_cb = Some(reload_cb);
    }
}
 
enum Message {
    Reload,
    Quit,
}
