use crate::app::gui::app_main::MyApp;

pub fn toggle_annotation_tools(app: &mut MyApp) {
    app.set_annotation_tools_active(!app.is_annotation_tools_active());
}
