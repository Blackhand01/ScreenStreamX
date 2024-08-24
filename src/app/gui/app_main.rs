// src/app/gui/app_main.rs
use eframe::{egui, App, CreationContext};
use local_ip_address::local_ip;
use crate::app::capture::CaptureArea;

use super::visuals::{configure_visuals, central_panel, capture_area_panel};
use std::sync::mpsc;

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
    capture_area: Option<CaptureArea>,
    stop_tx: Option<mpsc::Sender<()>>,  // Campo privato
    selecting_area: bool, // Nuovo campo per gestire la modalità di selezione dell'area
    show_confirmation_dialog: bool,  // Nuovo campo per gestire la finestra di conferma

}

impl MyApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        let ip_address = match local_ip() {
            Ok(ip) => ip.to_string(),
            Err(_) => String::from("Unable to get IP"),
        };

        Self {
            is_caster: false,
            address: ip_address,
            is_annotation_tools_active: false,
            is_recording: false,
            capture_area: Some(CaptureArea::default()), // Inizializza con un'area di cattura vuota
            stop_tx: None,  // Inizialmente nessun canale di stop
            selecting_area: false, // Inizialmente la selezione non è attiva
            show_confirmation_dialog: false, // Inizialmente la finestra di conferma non è visibile
        }
    }

    // Getter e setter per i vari stati dell'applicazione
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

    pub fn get_stop_tx(&self) -> Option<mpsc::Sender<()>> {
        self.stop_tx.clone()
    }

    pub fn set_stop_tx(&mut self, tx: Option<mpsc::Sender<()>>) {
        self.stop_tx = tx;
    }

    pub fn is_selecting_area(&self) -> bool {
        self.selecting_area
    }

    pub fn set_selecting_area(&mut self, value: bool) {
        self.selecting_area = value;
    }

    pub fn get_capture_area(&self) -> Option<&CaptureArea> {
        self.capture_area.as_ref()
    }

    pub fn get_capture_area_mut(&mut self) -> Option<&mut CaptureArea> {
        self.capture_area.as_mut()
    }

    pub fn set_capture_area(&mut self, area: Option<CaptureArea>) {
        self.capture_area = area;
    }
    pub fn show_confirmation_dialog(&self) -> bool {
        self.show_confirmation_dialog
    }

    pub fn set_show_confirmation_dialog(&mut self, value: bool) {
        self.show_confirmation_dialog = value;
    }
}



impl App for MyApp {
    /// Metodo principale di aggiornamento dell'interfaccia utente (ciclo di eventi).
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.is_selecting_area() {
            capture_area_panel(ctx, self);
        } else {
            // UI normale
            configure_visuals(ctx);
            central_panel(ctx, self);
        }
    }
    
}
