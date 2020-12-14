extern crate gio;
extern crate glib;
extern crate gtk;
use gio::prelude::*;
use std::env::args;

mod app;
mod otter;
mod util;

fn main() {
    let application = gtk::Application::new(Some("com.trmckay.oemu"), Default::default())
        .expect("Initialization failed...");

    application.connect_activate(|app| {
        app::gtk::build_gui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
