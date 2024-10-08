pub mod app;
pub mod command_dispatcher;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    let mut app = app::App::new();
    eframe::run_simple_native("Rift", native_options, move |ctx, _frame| {
        app.draw(ctx);
    })
}
