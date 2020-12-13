extern crate gio;
extern crate glib;
extern crate gtk;
use gio::prelude::*;
use std::env::args;
#[path = "./app.rs"]
mod app;

fn main() {
    let application = gtk::Application::new(Some("com.trmckay.oemu"), Default::default())
        .expect("Initialization failed...");

    application.connect_activate(|app| {
        app::build_gui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
