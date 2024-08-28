mod app;
mod utils;

use app::gui::app_main::initialize;

/// Punto di ingresso principale dell'applicazione ScreenCaster.
/// Questa funzione inizializza l'applicazione, configura i gestori di hotkey
/// e il trascrittore, quindi avvia il ciclo degli eventi.
fn main() {
    println!("Main function started.");

    // Inizializza l'applicazione e gestisci eventuali errori di inizializzazione.
    if let Err(e) = initialize() {
        eprintln!("Errore durante l'inizializzazione dell'interfaccia: {}", e);
        return;
    }
    println!("Applicazione inizializzata con successo.");

}
