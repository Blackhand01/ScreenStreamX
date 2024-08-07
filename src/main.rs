mod app;
mod ai;
mod utils;

use app::gui::app_main::initialize;
use ai::transcription::Transcriber;
use utils::hotkeys::HotkeyManager;

/// Punto di ingresso principale dell'applicazione ScreenCaster.
/// Questa funzione inizializza l'applicazione, configura i gestori di hotkey
/// e il trascrittore, quindi avvia il ciclo degli eventi.
fn main() {
    // Inizializza l'applicazione e ottieni l'evento loop, il display e la GUI.
    if let Err(e) = initialize() {
        eprintln!("Errore durante l'inizializzazione dell'interfaccia: {}", e);
        return;
    }

    // Configura e avvia il gestore di hotkey.
    let hotkey_manager = HotkeyManager::new();
    hotkey_manager.register_hotkeys();

    // Inizializza il trascrittore.
    let _transcriber = Transcriber::new();

    // Nota: Aggiungi il ciclo degli eventi solo se è separato dalla GUI.
    // In questo esempio, `initialize` gestisce tutto, quindi `run` non è necessario qui.
    // run(event_loop, display, gui, transcriber, hotkey_manager);
}
