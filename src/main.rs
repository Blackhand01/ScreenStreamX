mod gui;
mod capture;
mod network;

fn main() {
    println!("Welcome to ScreenStreamX!");

    // Esempio di chiamate alle funzioni dei moduli
    gui::initialize();
    capture::start_capture();
    network::start_streaming();
}
