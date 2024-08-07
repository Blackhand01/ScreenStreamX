use eframe::{egui, App, CreationContext};
use local_ip_address::local_ip;

pub fn initialize() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "ScreenStreamX",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

struct MyApp {
    is_caster: bool,
    address: String,
}

impl MyApp {
    fn new(_cc: &CreationContext<'_>) -> Self {
        // Ottenere l'indirizzo IP locale
        let ip_address = match local_ip() {
            Ok(ip) => ip.to_string(),
            Err(_) => String::from("Unable to get IP"),
        };

        Self {
            is_caster: false,
            address: ip_address,
        }
    }

    fn configure_visuals(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.window_shadow.blur = 10.0;
        visuals.window_shadow.offset = egui::Vec2::new(2.0, 2.0);
        visuals.window_rounding = egui::Rounding::same(10.0);
        visuals.override_text_color = Some(egui::Color32::from_gray(220));
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 30, 30);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(50, 50, 50);
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(60, 60, 60);
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(80, 80, 80);
        visuals.widgets.open.bg_fill = egui::Color32::from_rgb(70, 70, 70);
        ctx.set_visuals(visuals);
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Configura i visuals
        MyApp::configure_visuals(ctx);

        // Centrare il contenuto
        egui::CentralPanel::default().show(ctx, |ui| {
            // Centrare verticalmente e orizzontalmente il contenuto
            ui.vertical_centered(|ui| {
                // Aggiungi un'intestazione all'interfaccia utente
                ui.heading(egui::RichText::new("ScreenStreamX").strong());
                
                // Aggiungi uno spazio tra l'intestazione e i pulsanti
                ui.add_space(20.0);

                // Crea un layout orizzontale per i pulsanti Caster e Receiver
                ui.horizontal(|ui| {
                    // Crea un pulsante "Caster" e verifica se è stato cliccato
                    if ui.add_sized(
                        [150.0, 50.0],
                        egui::Button::new(
                            egui::RichText::new("Caster")
                                .color(egui::Color32::WHITE)
                                .strong()
                        ).fill(egui::Color32::from_rgb(0, 102, 204))
                    ).clicked() {
                        // Se il pulsante è stato cliccato, imposta is_caster su true
                        self.is_caster = true;
                    }
                    // Crea un pulsante "Receiver" e verifica se è stato cliccato
                    if ui.add_sized(
                        [150.0, 50.0],
                        egui::Button::new(
                            egui::RichText::new("Receiver")
                                .color(egui::Color32::WHITE)
                                .strong()
                        ).fill(egui::Color32::from_rgb(0, 204, 102))
                    ).clicked() {
                        // Se il pulsante è stato cliccato, imposta is_caster su false
                        self.is_caster = false;
                    }
                });

                // Aggiungi uno spazio tra i pulsanti e la sezione successiva
                ui.add_space(20.0);

                // Verifica se la modalità caster è attiva
                if self.is_caster {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Caster mode selected").strong());
                        ui.add_space(10.0);
                        // Pulsante per la selezione dell'area di cattura
                        if ui.add_sized(
                            [200.0, 40.0],
                            egui::Button::new(
                                egui::RichText::new("Select Capture Area")
                                    .color(egui::Color32::WHITE)
                                    .strong()
                            ).fill(egui::Color32::from_rgb(255, 153, 102)) // Arancione meno intenso
                        ).clicked() {
                            // Implementa la logica di selezione dell'area di cattura
                        }
                        ui.add_space(10.0);
                        // Pulsante per attivare/disattivare gli strumenti di annotazione
                        if ui.add_sized(
                            [200.0, 40.0],
                            egui::Button::new(
                                egui::RichText::new("Toggle Annotation Tools")
                                    .color(egui::Color32::WHITE)
                                    .strong()
                            ).fill(egui::Color32::from_rgb(153, 0, 153)) // Viola meno intenso
                        ).clicked() {
                            // Implementa la logica per attivare/disattivare gli strumenti di annotazione
                        }
                        ui.add_space(10.0);
                        // Pulsante per supportare più monitor
                        if ui.add_sized(
                            [200.0, 40.0],
                            egui::Button::new(
                                egui::RichText::new("Multi-Monitor Support")
                                    .color(egui::Color32::WHITE)
                                    .strong()
                            ).fill(egui::Color32::from_rgb(102, 204, 255)) // Azzurro meno intenso
                        ).clicked() {
                            // Implementa la logica per supportare più monitor
                        }
                    });
                } else {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Receiver mode selected").strong());
                        ui.add_space(10.0);
                        ui.vertical_centered(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Caster address:").strong());
                                ui.text_edit_singleline(&mut self.address);
                            });
                        });
                        ui.add_space(10.0);
                        // Pulsante per avviare la registrazione del contenuto ricevuto
                        if ui.add_sized(
                            [200.0, 40.0],
                            egui::Button::new(
                                egui::RichText::new("Start Recording")
                                    .color(egui::Color32::WHITE)
                                    .strong()
                            ).fill(egui::Color32::from_rgb(204, 51, 51)) // Rosso meno intenso
                        ).clicked() {
                            // Implementa la logica per avviare la registrazione
                        }
                        ui.add_space(10.0);
                        // Pulsante per terminare la registrazione del contenuto ricevuto
                        if ui.add_sized(
                            [200.0, 40.0],
                            egui::Button::new(
                                egui::RichText::new("Stop Recording")
                                    .color(egui::Color32::WHITE)
                                    .strong()
                            ).fill(egui::Color32::from_rgb(102, 0, 0)) // Rosso scuro meno intenso
                        ).clicked() {
                            // Implementa la logica per terminare la registrazione
                        }
                    });
                }
            });
        });
    }
}