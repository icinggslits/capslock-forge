#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]



mod config;

mod feature;

mod tray;

mod i18n;

mod units;

fn main() {
    fn init() {
        config::init();
    }
    
    init();
    
    let mut tray = tray::Tray::new();
    tray.reload_with(Box::new(|| {
        init();
    }));
    
    tray.run();

}
