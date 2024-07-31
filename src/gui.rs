use druid::{AppLauncher, WindowDesc, Data, Lens, Widget, WidgetExt, ImageBuf, Selector, Target, Command, Env};
use druid::widget::{Button, Flex, Label, TextBox, SizedBox};
use druid::piet::{ImageFormat, InterpolationMode};
use druid::{AppDelegate, DelegateCtx, Handled, RenderContext};
use crate::capture;
use crate::network;
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::Arc;
use tokio::sync::mpsc as tokio_mpsc;

#[derive(Clone, Lens)]
struct AppState {
    mode: String,
    address: String,
    #[lens(ignore)]
    image: Option<ImageData>,
}

#[derive(Clone)]
struct ImageData(Arc<ImageBuf>);

impl Data for ImageData {
    fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Data for AppState {
    fn same(&self, other: &Self) -> bool {
        self.mode == other.mode && self.address == other.address && self.image.same(&other.image)
    }
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            mode: "caster".into(),
            address: "".into(),
            image: None,
        }
    }
}

pub async fn run() {
    let main_window = WindowDesc::new(build_root_widget)
        .title("Screencast App")
        .window_size((800.0, 600.0));
    let initial_state = AppState::new();
    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn build_root_widget() -> impl Widget<AppState> {
    let mode_label = Label::new("Mode:");
    let mode_input = TextBox::new().lens(AppState::mode);
    
    let address_label = Label::new("Address:");
    let address_input = TextBox::new().lens(AppState::address);
    
    let start_button = Button::new("Start")
        .on_click(|ctx, data: &mut AppState, _env| {
            if data.mode == "caster" {
                // Start casting
                tokio::spawn(async {
                    capture::start_casting().await;
                });
            } else {
                // Start receiving
                let addr = data.address.clone();
                let (tx, rx): (Sender<ImageBuf>, Receiver<ImageBuf>) = mpsc::channel();
                let handle = ctx.get_external_handle();
                tokio::spawn(async move {
                    let _ = network::start_receiving(addr, tx).await;
                });

                // Avvia un thread separato per ricevere le immagini e inviare comandi all'interfaccia utente
                std::thread::spawn(move || {
                    while let Ok(image) = rx.recv() {
                        let image_data = ImageData(Arc::new(image));
                        handle.submit_command(UPDATE_IMAGE, image_data, Target::Auto).unwrap();
                    }
                });
            }
        });

    Flex::column()
        .with_child(mode_label)
        .with_child(mode_input)
        .with_child(address_label)
        .with_child(address_input)
        .with_child(start_button)
        .with_flex_child(
            SizedBox::new(
                ImageWidget::new().lens(AppStateLens).fix_size(1920.0, 1080.0)
            ),
            1.0
        )
}

// Comando personalizzato per aggiornare l'immagine
const UPDATE_IMAGE: Selector<ImageData> = Selector::new("update_image");

// Implementa un delegato per gestire i comandi personalizzati
struct Delegate;

impl AppDelegate<AppState> for Delegate {
    fn command(&mut self, ctx: &mut DelegateCtx, target: Target, cmd: &Command, data: &mut AppState, env: &Env) -> Handled {
        if let Some(image) = cmd.get(UPDATE_IMAGE) {
            data.image = Some(image.clone());
            return Handled::Yes;
        }
        Handled::No
    }
}

// Implementazione di un widget Image personalizzato
struct ImageWidget;

impl ImageWidget {
    pub fn new() -> Self {
        ImageWidget
    }
}

impl Widget<Option<ImageData>> for ImageWidget {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut Option<ImageData>, env: &druid::Env) {
        // Gestisci eventi, se necessario
    }

    fn lifecycle(&mut self, ctx: &mut druid::LifeCycleCtx, event: &druid::LifeCycle, data: &Option<ImageData>, env: &druid::Env) {
        // Gestisci ciclo di vita, se necessario
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &Option<ImageData>, data: &Option<ImageData>, env: &druid::Env) {
        if !old_data.same(data) {
            ctx.request_paint();
        }
    }

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, data: &Option<ImageData>, env: &druid::Env) -> druid::Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &Option<ImageData>, env: &druid::Env) {
        if let Some(image_data) = data {
            let image = image_data.0.to_owned();
            let size = ctx.size();
            if let Ok(cg_image) = ctx.make_image(image.width(), image.height(), &image.raw_pixels(), ImageFormat::RgbaSeparate) {
                ctx.draw_image(&cg_image, size.to_rect(), InterpolationMode::Bilinear);
            }
        }
    }
}

// Definizione della lente personalizzata per accedere al campo `image` di `AppState`
struct AppStateLens;

impl Lens<AppState, Option<ImageData>> for AppStateLens {
    fn with<V, F: FnOnce(&Option<ImageData>) -> V>(&self, data: &AppState, f: F) -> V {
        f(&data.image)
    }

    fn with_mut<V, F: FnOnce(&mut Option<ImageData>) -> V>(&self, data: &mut AppState, f: F) -> V {
        f(&mut data.image)
    }
}
