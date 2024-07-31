use scrap::{Capturer, Display};
use tokio::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::io::ErrorKind;

pub async fn start_casting() {
    let (tx, mut rx) = mpsc::channel(32);
    thread::spawn(move || {
        let display = Display::primary().expect("Couldn't find primary display.");
        let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
        loop {
            match capturer.frame() {
                Ok(frame) => {
                    let buffer = frame.to_vec();
                    tx.blocking_send(buffer).expect("Couldn't send frame.");
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    // Retry until the frame is available
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => {
                    eprintln!("Error capturing frame: {:?}", e);
                    break;
                }
            }
        }
    });
    
    while let Some(buffer) = rx.recv().await {
        // Send buffer to peers
        println!("Buffer length: {}", buffer.len());
    }
}
