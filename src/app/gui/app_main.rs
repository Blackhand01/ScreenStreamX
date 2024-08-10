use eframe::{egui, App, CreationContext};
use local_ip_address::local_ip;

use super::visuals::configure_visuals;
use super::visuals::central_panel;

pub fn initialize() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "ScreenStreamX",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

pub struct MyApp {
    is_caster: bool,
    address: String,
    is_annotation_tools_active: bool,
    is_recording: bool,
}

impl MyApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        // Ottenere l'indirizzo IP locale
        let ip_address = match local_ip() {
            Ok(ip) => ip.to_string(),
            Err(_) => String::from("Unable to get IP"),
        };

        Self {
            is_caster: false,
            address: ip_address,
            is_annotation_tools_active: false,
            is_recording: false,
        }
    }

    pub fn is_caster(&self) -> bool {
        self.is_caster
    }

    pub fn set_caster(&mut self, value: bool) {
        self.is_caster = value;
    }

    pub fn is_annotation_tools_active(&self) -> bool {
        self.is_annotation_tools_active
    }

    pub fn set_annotation_tools_active(&mut self, value: bool) {
        self.is_annotation_tools_active = value;
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    pub fn set_recording(&mut self, value: bool) {
        self.is_recording = value;
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn set_address(&mut self, value: String) {
        self.address = value;
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Configura gli elementi gratici
        configure_visuals(ctx);

        // Configura il pannello centrale
        central_panel(ctx, self);
    }
}
