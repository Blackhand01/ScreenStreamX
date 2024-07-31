use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use std::error::Error;
use image::{ImageBuffer, Rgba};
use druid::{ImageBuf, piet::ImageFormat};
use std::sync::{mpsc::Sender, Arc};

pub async fn start_receiving(address: String, tx: Sender<ImageBuf>) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(address).await?;
    
    loop {
        let (mut socket, _) = listener.accept().await?;
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut buffer = vec![0; 1920 * 1080 * 4]; // Assumendo una risoluzione di 1080p e RGBA
            loop {
                match socket.read_exact(&mut buffer).await {
                    Ok(_) => {
                        // Converti il buffer in un'immagine e invialo tramite il canale
                        if let Some(image) = ImageBuffer::<Rgba<u8>, _>::from_raw(1920, 1080, buffer.clone()) {
                            let image_raw: Arc<[u8]> = Arc::from(buffer.clone().into_boxed_slice());
                            match ImageBuf::from_raw(image_raw, ImageFormat::RgbaSeparate, 1920, 1080) {
                                image_buf => {
                                    if tx.send(image_buf).is_err() {
                                        eprintln!("Error sending image");
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from socket: {:?}", e);
                        break;
                    }
                }
            }
        });
    }
    Ok(())
}

pub async fn start_casting() {
    let listener = TcpListener::bind("0.0.0.0:12345").await.unwrap();
    
    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            // Send the captured frames to this socket
            println!("Accepted connection");
        });
    }
}
