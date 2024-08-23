// src/app/network.rs

use crate::app::capture::{ScreenCapturer, ScreenCapture, CaptureArea};  // Importa correttamente ScreenCapturer e CaptureArea
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use bincode;

/// Funzione per avviare il server che trasmette i frame catturati.
pub fn start_server(capture_area: Option<CaptureArea>, recording_flag: Arc<Mutex<bool>>) {
    let listener = TcpListener::bind("0.0.0.0:8080").expect("Impossibile avviare il server");

    thread::spawn(move || {
        println!("Broadcast server started");

        if let Ok((mut stream, _addr)) = listener.accept() {
            println!("Connection established with receiver");

            let mut screen_capturer = ScreenCapturer::new(capture_area);

            while *recording_flag.lock().unwrap() {
                if let Some(frame) = screen_capturer.capture_frame() {
                    let serialized_frame = bincode::serialize(&frame).expect("Failed to serialize frame");
                    stream.write_all(&serialized_frame).expect("Failed to send frame");
                }
            }
        }

        println!("Broadcast server exiting");
    });
}

/// Funzione per avviare il client che riceve i frame e li processa.
pub fn start_client(
    receiver_address: &str,
    recording_flag: Arc<Mutex<bool>>,
    on_frame_received: impl Fn(ScreenCapture) + Send + 'static,
) {
    match TcpStream::connect(format!("{}:8080", receiver_address)) {
        Ok(mut stream) => {
            println!("Receiver client started");

            thread::spawn(move || {
                while *recording_flag.lock().unwrap() {
                    // Leggi la lunghezza del frame
                    let mut length_buffer = [0u8; 4]; // Un buffer per leggere la lunghezza del frame
                    if stream.read_exact(&mut length_buffer).is_err() {
                        println!("Failed to read frame length.");
                        break;
                    }

                    let length = u32::from_be_bytes(length_buffer) as usize;

                    let mut buffer = vec![0; length];

                    match stream.read_exact(&mut buffer) {
                        Ok(_) => {
                            match bincode::deserialize::<ScreenCapture>(&buffer) {
                                Ok(frame) => on_frame_received(frame),
                                Err(e) => println!("Failed to deserialize frame: {:?}", e),
                            }
                        }
                        Err(e) => {
                            println!("Error reading from stream: {:?}", e);
                            break;
                        }
                    }
                }

                println!("Receiver client exiting");
            });
        }
        Err(e) => {
            println!("Failed to connect to caster: {:?}", e);
        }
    }
}

