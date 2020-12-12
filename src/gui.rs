extern crate gtk;
extern crate glib;
extern crate gio;
use std::sync::{Arc, Mutex};
use std::thread;
use gtk::prelude::*;
#[path = "./otter.rs"] mod otter;

struct GUIMessage {
    console_msg: String,
    update_leds: bool,
    new_leds: Vec<bool>,
    update_sseg: bool,
    new_sseg: Vec<bool>,
}

impl GUIMessage {
    fn console_msg(msg: &str) -> GUIMessage {
        GUIMessage {
            console_msg: String::from(msg),
            update_leds: false,
            update_sseg: false,
            new_leds: vec![],
            new_sseg: vec![]
        }
    }
}

pub fn run() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let mcu_mutex = Arc::from(Mutex::from(otter::MCU::from_bin("./res/programs/test/all/bin")));
    let running_mutex = Arc::from(Mutex::from(false));

    // load in glade source
    let glade_src = include_str!("../res/gui/gtk_main.ui");
    let builder = gtk::Builder::from_string(glade_src);

    // main window
    let window: gtk::Window = builder.get_object("main_window").unwrap();

    // buttons
    let run_btn: gtk::Button = builder.get_object("run_btn").unwrap();
    let step_btn: gtk::Button = builder.get_object("step_btn").unwrap();
    let pause_btn: gtk::Button = builder.get_object("pause_btn").unwrap();

    // menu items
    let open_bin_btn: gtk::MenuItem = builder.get_object("open_binary").unwrap();

    let (tx_main, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let console: gtk::TextBuffer = builder.get_object("console_buffer").unwrap();

    // set receive channel to update GUI
    rx.attach(None, move |message: GUIMessage| {
        console.insert_at_cursor(&format!("\n{}", message.console_msg));
        glib::Continue(true)
    });

    // FILE DIALOG
    // clone window handle (for opening a file dialogue)
    let window_clone = window.clone();
    let mcu = mcu_mutex.clone();
    let tx = tx_main.clone();
    open_bin_btn.connect_activate(move |_| {
        let mut mcu = mcu.lock().unwrap();
        // open a dialogue
        let dialog = gtk::FileChooserDialog::with_buttons(
            Some("Choose a file"),
            Some(&window_clone),
            gtk::FileChooserAction::Open,
            &[("Cancel", gtk::ResponseType::Cancel), ("Open", gtk::ResponseType::Ok)],
        );
        // get the uri
        let result_uri = if dialog.run() == gtk::ResponseType::Ok {
            dialog.get_uri()
        } else {
            None
        };
        // parse into path
        let path = String::from(result_uri.unwrap().trim_start_matches("file://"));
        dialog.close();
        // program the MCU
        mcu.load_bin(&path);
        tx.send(GUIMessage::console_msg(&format!("Programmed with {}.", path))).unwrap();
    });

    // RUN
    let mcu = mcu_mutex.clone();
    let running = running_mutex.clone();
    let tx = tx_main.clone();
    run_btn.connect_clicked(move |_| {
        tx.send(GUIMessage::console_msg("State set to running.")).unwrap();
        let mcu = mcu.clone();
        let running = running.clone();
        // create a new thread so the CPU runs in the background
        thread::spawn(move || {
            // TODO: update GUI here
            let mut local_running = true;
            while local_running {
                {
                    let mut mcu = mcu.lock().unwrap();
                    mcu.step();
                }
                local_running = *running.lock().unwrap();
            }
        });
    });

    // STEP
    let tx = tx_main.clone();
    step_btn.connect_clicked(move |_| {
        // TODO: implement
        tx.send(GUIMessage::console_msg("Step.")).unwrap();
    });

    // PAUSE
    let tx = tx_main.clone();
    let running = running_mutex.clone();
    pause_btn.connect_clicked(move |_| {
        tx.send(GUIMessage::console_msg("State set to paused.")).unwrap();
        // set running state to false
        let mut running = running.lock().unwrap();
        *running = false;
    });

    window.show_all();
    gtk::main();
}
