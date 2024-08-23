// src/app/capture.rs
use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;
use image::{ImageBuffer, Rgba};
use serde::{Serialize, Deserialize};

// Definizione di ScreenCapture per la serializzazione
#[derive(Serialize, Deserialize, Debug)]
pub struct ScreenCapture {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // I dati dell'immagine memorizzati come un vettore di byte
}

impl ScreenCapture {
    pub fn from_image_buffer(buffer: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Self {
        let (width, height) = buffer.dimensions();
        let data = buffer.clone().into_raw();
        ScreenCapture { width, height, data }
    }

    pub fn into_image_buffer(self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        ImageBuffer::from_raw(self.width, self.height, self.data).expect("Errore nella conversione in ImageBuffer")
    }
}

#[derive(Clone, Debug, Default)]
pub struct DragState {
    pub start_pos: Option<egui::Pos2>,
    pub end_pos: Option<egui::Pos2>,
}

impl DragState {
    pub fn new() -> Self {
        DragState {
            start_pos: None,
            end_pos: None,
        }
    }

    pub fn reset(&mut self) {
        self.start_pos = None;
        self.end_pos = None;
    }
}


/// Struttura che rappresenta l'area di cattura selezionata.
#[derive(Clone, Debug, Default)]
pub struct CaptureArea {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub drag_state: DragState,  // Stato di drag incorporato
}

impl CaptureArea {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        CaptureArea { 
            x, 
            y, 
            width, 
            height,
            drag_state: DragState::new(),
        }
    }

    /// Verifica se l'area di cattura è valida (non zero).
    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }

    /// Converte l'area di cattura in un rettangolo.
    pub fn as_rect(&self) -> (usize, usize, usize, usize) {
        (self.x, self.y, self.width, self.height)
    }
}


/// Struttura responsabile della cattura dello schermo.
pub struct ScreenCapturer {
    capturer: Capturer,
    width: usize,
    height: usize,
    capture_area: Option<CaptureArea>,
}

impl ScreenCapturer {
    /// Crea un nuovo `ScreenCapturer` per catturare l'intero schermo o un'area specifica.
    pub fn new(capture_area: Option<CaptureArea>) -> Self {
        let display = Display::primary().expect("Failed to find primary display.");
        let capturer = Capturer::new(display).expect("Failed to begin capture.");
        let (width, height) = (capturer.width(), capturer.height());

        // Verifica che l'area di cattura sia valida rispetto alla dimensione dello schermo.
        if let Some(area) = &capture_area {
            assert!(area.x + area.width <= width && area.y + area.height <= height, "Invalid capture area.");
        }

        Self {
            capturer,
            width,
            height,
            capture_area,
        }
    }

    /// Cattura un singolo frame dello schermo o dell'area selezionata.
    pub fn capture_frame(&mut self) -> Option<ScreenCapture> {
        loop {
            match self.capturer.frame() {
                Ok(frame) => {
                    let frame = frame.to_vec();
                    let mut buffer = ImageBuffer::new(self.width as u32, self.height as u32);

                    for (x, y, pixel) in buffer.enumerate_pixels_mut() {
                        let idx = (y as usize * self.width + x as usize) * 4;
                        *pixel = Rgba([frame[idx + 2], frame[idx + 1], frame[idx], 255]);
                    }

                    let image_buffer = match &self.capture_area {
                        Some(area) => self.crop_frame(&buffer, area),
                        None => buffer,
                    };

                    return Some(ScreenCapture::from_image_buffer(&image_buffer));
                }
                Err(ref e) if e.kind() == WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(_) => return None,
            }
        }
    }

    /// Taglia l'immagine catturata all'area specificata.
    fn crop_frame(&self, buffer: &ImageBuffer<Rgba<u8>, Vec<u8>>, area: &CaptureArea) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut cropped = ImageBuffer::new(area.width as u32, area.height as u32);
        for y in 0..area.height {
            for x in 0..area.width {
                let pixel = buffer.get_pixel((area.x + x) as u32, (area.y + y) as u32);
                cropped.put_pixel(x as u32, y as u32, *pixel);
            }
        }
        cropped
    }
}
