extern crate gio;
extern crate glib;
extern crate gtk;
use clap::App;
use gio::prelude::*;
use std::env::args;

mod app;
mod otter;
mod util;

fn main() {
    App::new("oemu")
        .version("0.1.0")
        .author("Trevor McKay <trmckay@calpoly.edu>")
        .about("Emulator for the RV32I multi-cycle Otter")
        .get_matches();

    let application = gtk::Application::new(Some("com.trmckay.oemu"), Default::default())
        .expect("Initialization failed...");

    application.connect_activate(|app| {
        app::gtk::build_gui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
