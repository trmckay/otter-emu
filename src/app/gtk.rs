extern crate gio;
extern crate glib;
extern crate gtk;
extern crate webbrowser;
use super::super::otter;
use super::super::util;
use gtk::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const GUI_REFRESH_PERIOD: usize = 1;
const IR_PERIOD_US: u64 = 2000;

struct GUIMessage {
    console_msg: String,
    update_leds: bool,
    update_pc: bool,
    new_leds: Vec<bool>,
    update_sseg: bool,
    new_sseg: u16,
    update_rf: bool,
    new_rf: Vec<u32>,
    new_pc: u32,
    update_ir: bool,
    new_ir: otter::Instruction,
}

// contains all the data needed to update the GUI
impl GUIMessage {
    fn gui_update(
        print: Option<&str>,
        leds: Option<Vec<bool>>,
        sseg: Option<u16>,
        rf: Option<Vec<u32>>,
        pc: Option<u32>,
        ir: Option<otter::Instruction>,
    ) -> GUIMessage {
        let mut msg = GUIMessage {
            console_msg: String::from(""),
            new_sseg: 0,
            new_pc: 0,
            new_leds: vec![false; 16],
            new_rf: vec![0; 32],
            new_ir: otter::Instruction {
                op: otter::Operation::Invalid,
                rs1: 0,
                rs2: 0,
                rd: 0,
                imm: 0,
            },
            update_leds: false,
            update_pc: false,
            update_sseg: false,
            update_rf: false,
            update_ir: false,
        };

        if let Some(s) = print {
            msg.console_msg = String::from(s);
        }

        if let Some(l) = leds {
            msg.update_leds = true;
            msg.new_leds = l;
        };

        if let Some(s) = sseg {
            msg.update_sseg = true;
            msg.new_sseg = s;
        };

        if let Some(p) = pc {
            msg.update_pc = true;
            msg.new_pc = p;
        }

        if let Some(r) = rf {
            msg.update_rf = true;
            msg.new_rf = r;
        };

        if let Some(i) = ir {
            msg.update_ir = true;
            msg.new_ir = i;
        }

        msg
    }

    fn log_console(tx: &glib::Sender<GUIMessage>, message: &str) {
        tx.send(GUIMessage::gui_update(
            Some(message),
            None,
            None,
            None,
            None,
            None,
        ))
        .unwrap();
    }
}

pub fn build_gui(application: &gtk::Application) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    // mutexs for shared memory
    let mcu_mutex = Arc::from(Mutex::from(otter::MCU::new()));
    let running_mutex = Arc::from(Mutex::from(false));
    let programmed_mutex = Arc::from(Mutex::from(false));
    let bps_mutex: Arc<Mutex<Vec<u32>>> = Arc::from(Mutex::from(Vec::new()));

    // load in glade source
    let glade_src = include_str!("../../res/gui/gtk.ui");
    let builder = gtk::Builder::from_string(glade_src);

    // main window
    let window: gtk::ApplicationWindow = builder.get_object("main_window").unwrap();
    window.set_application(Some(application));
    window.set_resizable(false);

    // control buttons
    let run_btn: gtk::Button = builder.get_object("run_btn").unwrap();
    let step_btn: gtk::Button = builder.get_object("step_btn").unwrap();
    let pause_btn: gtk::Button = builder.get_object("pause_btn").unwrap();
    let reset_btn: gtk::Button = builder.get_object("reset_btn").unwrap();
    let mem_rd_btn: gtk::Button = builder.get_object("read_mem_btn").unwrap();

    // menu items
    let load_bin_btn: gtk::Button = builder.get_object("load_binary_btn").unwrap();
    let about_btn: gtk::Button = builder.get_object("about_btn").unwrap();
    let dump_btn: gtk::Button = builder.get_object("dump_btn").unwrap();

    // switches
    for i in 0..16 {
        let mcu = mcu_mutex.clone();
        let sw: gtk::CheckButton = builder.get_object(&format!("switch{}", i)).unwrap();
        sw.connect_clicked(move |_| {
            let mut mcu = mcu.lock().unwrap();
            mcu.toggle_sw(i);
        });
    }

    // I/O
    let sseg: gtk::TextBuffer = builder.get_object("sseg_buffer").unwrap();

    // console
    let console_buffer: gtk::TextBuffer = builder.get_object("console_buffer").unwrap();
    let console_container: gtk::ScrolledWindow = builder.get_object("console_container").unwrap();

    // pc
    let pc_buffer: gtk::TextBuffer = builder.get_object("pc_buffer").unwrap();

    // ir buffers
    let ir_type_buffer: gtk::TextBuffer = builder.get_object("ir_type_buffer").unwrap();
    let ir_rd_buffer: gtk::TextBuffer = builder.get_object("ir_rd_buffer").unwrap();
    let ir_rs1_buffer: gtk::TextBuffer = builder.get_object("ir_rs1_buffer").unwrap();
    let ir_rs2_buffer: gtk::TextBuffer = builder.get_object("ir_rs2_buffer").unwrap();
    let ir_imm_buffer: gtk::TextBuffer = builder.get_object("ir_imm_buffer").unwrap();

    // channel for listening to GUI updates
    let (tx_main, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    // clone builder so the GUI updated closure has access
    let builder_clone = builder.clone();
    // attach to the rcv channel
    rx.attach(None, move |message: GUIMessage| {
        // update the console
        if message.console_msg != "" {
            console_buffer.insert_at_cursor(&format!("\n{}", message.console_msg));
            // auto-scroll
            let adj = console_container.get_vadjustment().unwrap();
            adj.set_value(adj.get_upper() - adj.get_page_size());
        }
        // udpate the SSEG
        if message.update_sseg {
            sseg.set_text(&format!("{:#06X}", message.new_sseg));
        }
        // update the LEDs
        if message.update_leds {
            for i in 0..16 {
                let curr_led: gtk::Image = builder_clone.get_object(&format!("led{}", i)).unwrap();
                if message.new_leds[i] {
                    curr_led.set_opacity(100.0);
                } else {
                    curr_led.set_opacity(0.0);
                }
            }
        }
        // update the registers
        if message.update_rf {
            for i in 0..32 {
                let curr_rf_buffer: gtk::TextBuffer = builder_clone
                    .get_object(&format!("rf_buffer{}", i))
                    .unwrap();
                curr_rf_buffer.set_text(&format!(" {:#010X} ", message.new_rf[i]));
            }
        }
        // update the PC
        if message.update_pc {
            pc_buffer.set_text(&format!(" {:#010X} ", message.new_pc));
        }
        // update the current instruction
        if message.update_ir {
            let rd = message.new_ir.rd;
            let rs1 = message.new_ir.rd;
            let rs2 = message.new_ir.rd;
            ir_type_buffer.set_text(&format!(" {:?} ", message.new_ir.op));
            ir_rd_buffer.set_text(&format!(" x{} ({}) ", rd, otter::reg_name(rd)));
            ir_rs1_buffer.set_text(&format!(" x{} ({}) ", rs1, otter::reg_name(rs1)));
            ir_rs2_buffer.set_text(&format!(" x{} ({}) ", rs2, otter::reg_name(rs2)));
            ir_imm_buffer.set_text(&format!(" {:#010X} ", message.new_ir.imm));
        };
        // continue
        glib::Continue(true)
    });

    // FILE DIALOG
    let window_clone = window.clone();
    let mcu = mcu_mutex.clone();
    let tx = tx_main.clone();
    let programmed = programmed_mutex.clone();
    load_bin_btn.connect_clicked(move |_| {
        let mut mcu = mcu.lock().unwrap();
        // open a dialogue
        let dialog = gtk::FileChooserDialog::with_buttons(
            Some("Choose a file"),
            Some(&window_clone),
            gtk::FileChooserAction::Open,
            &[
                ("Cancel", gtk::ResponseType::Cancel),
                ("Open", gtk::ResponseType::Ok),
            ],
        );
        // get the uri
        let result_uri = if dialog.run() == gtk::ResponseType::Ok {
            dialog.get_uri()
        } else {
            dialog.close();
            return;
        };
        // parse into path
        let path = String::from(result_uri.unwrap().trim_start_matches("file://"));
        dialog.close();
        // program the MCU
        mcu.load_bin(&path);
        *programmed.lock().unwrap() = true;
        GUIMessage::log_console(&tx, &format!("Programmed with {}.", path));
    });

    // CONSOLE BUTTON
    let console_btn: gtk::Button = builder.get_object("console_btn").unwrap();
    let builder_clone = builder.clone();
    console_btn.connect_clicked(move |_| {
        let console: gtk::Window = builder_clone.get_object("console_window").unwrap();
        console.set_default_size(600, 600);
        console.show_all();
        console.grab_focus();
    });

    // BREAKPOINTS BTNs
    let bp_btn: gtk::Button = builder.get_object("bp_btn").unwrap();
    let builder_clone = builder.clone();
    bp_btn.connect_clicked(move |_| {
        let bps: gtk::Window = builder_clone.get_object("bp_window").unwrap();
        bps.set_default_size(400, 200);
        bps.show_all();
        bps.grab_focus();
    });

    // BREAKPOINTS ENTRY
    let builder_clone = builder.clone();
    let add_bp_btn: gtk::Button = builder.get_object("add_bp_btn").unwrap();
    let clear_bp_btn: gtk::Button = builder.get_object("clear_bp_btn").unwrap();
    let bps_clone = bps_mutex.clone();
    add_bp_btn.connect_clicked(move |_| {
        let input: gtk::Entry = builder_clone.get_object("bp_entry").unwrap();
        let list: gtk::ListBox = builder_clone.get_object("bp_list").unwrap();
        let addr = match util::parse::parse_int(&input.get_text()) {
            Ok(n) => n,
            _ => return,
        };
        bps_clone.lock().unwrap().push(addr);
        let row = gtk::ListBoxRow::new();
        let label = gtk::Label::new(Some(&format!("{:#10X}", addr)));
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 20);
        container.add(&label);
        container.pack_start(&label, true, true, 10);
        row.add(&container);
        list.add(&row);
        list.show_all();
    });
    let bps_clone = bps_mutex.clone();
    let builder_clone = builder.clone();
    clear_bp_btn.connect_clicked(move |_| {
        bps_clone.lock().unwrap().clear();
        let list: gtk::ListBox = builder_clone.get_object("bp_list").unwrap();
        list.foreach(|w| {list.remove(w)});
    });

    // ABOUT BUTTON
    about_btn.connect_clicked(move |_| {
        webbrowser::open("https://github.com/trmckay/otter-emu").unwrap();
    });

    // DUMP STATE BUTTON
    let mcu = mcu_mutex.clone();
    let tx_logger = tx_main.clone();
    dump_btn.connect_clicked(move |_| {
        mcu.lock().unwrap().dump("oemu.dump", |s| {GUIMessage::log_console(&tx_logger, s)});
    });

    // READ MEMORY
    let builder_clone = builder.clone();
    let mcu = mcu_mutex.clone();
    mem_rd_btn.connect_clicked(move |_| {
        let input: gtk::Entry = builder_clone.get_object("mem_addr_entry").unwrap();
        let addr = match util::parse::parse_int(&input.get_text()) {
            Ok(n) => n,
            _ => return,
        };
        let mcu = mcu.lock().unwrap();
        let res = mcu.mem_rd(addr, otter::Size::Word);
        let buffer: gtk::TextBuffer = builder.get_object("mem_buffer").unwrap();
        buffer.set_text(&format!("{:#010X}", res));
    });

    // RESET
    let mcu = mcu_mutex.clone();
    let tx = tx_main.clone();
    // set MCU PC to zero and log to console
    reset_btn.connect_clicked(move |_| {
        let mut mcu = mcu.lock().unwrap();
        mcu.reset();
        tx.send(GUIMessage::gui_update(
            Some("Reset MCU."),
            Some(mcu.leds()),
            Some(mcu.sseg()),
            Some(mcu.rf()),
            Some(mcu.pc),
            Some(mcu.fetch(|_s| {}).0),
        ))
        .unwrap();
    });

    // RUN
    let mcu = mcu_mutex.clone();
    let running = running_mutex.clone();
    let programmed = programmed_mutex.clone();
    let tx = tx_main.clone();
    run_btn.connect_clicked(move |_| {
        // do not spawn another thread if it's already running
        if *running.lock().unwrap() {
            return;
        }
        if !*programmed.lock().unwrap() {
            GUIMessage::log_console(&tx, "Error: MCU must be programmed first.");
            return;
        }
        // clone mutexs
        let mcu = mcu.clone();
        let running = running.clone();
        let tx = tx.clone();
        let bps = bps_mutex.clone();
        // create a new thread so the CPU runs in the background
        thread::spawn(move || {
            let mut c: usize = 0;
            *running.lock().unwrap() = true;
            let mut local_running = true;
            // do while still running (wait for pause btn)
            while local_running {
                let mut mcu = mcu.lock().unwrap();
                let tx_logger = tx.clone();
                mcu.step(move |s| GUIMessage::log_console(&tx_logger, s));
                if bps.lock().unwrap().contains(&mcu.pc) {
                    *running.lock().unwrap() = false;
                    local_running = false;
                    tx.send(GUIMessage::gui_update(
                        Some(&format!("Encountered breakpoint at {:#010X}.", mcu.pc)),
                        Some(mcu.leds()),
                        Some(mcu.sseg()),
                        Some(mcu.rf()),
                        Some(mcu.pc),
                        Some(mcu.fetch(|_s| {}).0),
                    ))
                    .unwrap();
                } else {
                    local_running = *running.lock().unwrap();
                }
                // refresh GUI
                thread::sleep(Duration::from_micros(IR_PERIOD_US));
                c += 1;
                if c == GUI_REFRESH_PERIOD {
                    // reset count, lock mcu, read, send message
                    c = 0;
                    tx.send(GUIMessage::gui_update(
                        None,
                        Some(mcu.leds()),
                        Some(mcu.sseg()),
                        Some(mcu.rf()),
                        Some(mcu.pc),
                        None,
                    ))
                    .unwrap();
                }
            }
        });
    });

    // STEP
    let tx = tx_main.clone();
    let mcu = mcu_mutex.clone();
    let running = running_mutex.clone();
    let programmed = programmed_mutex.clone();
    step_btn.connect_clicked(move |_| {
        if !*programmed.lock().unwrap() {
            GUIMessage::log_console(&tx, "Error: MCU must be programmed first.");
            return;
        }
        if *running.lock().unwrap() {
            GUIMessage::log_console(&tx, "Error: Cannot step while running.");
            return;
        }
        let mut mcu = mcu.lock().unwrap();
        let tx_logger = tx.clone();
        mcu.step(move |s| GUIMessage::log_console(&tx_logger, s));
        tx.send(GUIMessage::gui_update(
            None,
            Some(mcu.leds()),
            Some(mcu.sseg()),
            Some(mcu.rf()),
            Some(mcu.pc),
            Some(mcu.fetch(|_s| {}).0),
        ))
        .unwrap();
    });

    // PAUSE
    let tx = tx_main.clone();
    let mcu = mcu_mutex.clone();
    let running = running_mutex.clone();
    pause_btn.connect_clicked(move |_| {
        // set running state to false
        {
            let mut running = running.lock().unwrap();
            *running = false;
            thread::sleep(Duration::from_micros(5000));
        }
        let mcu = mcu.lock().unwrap();
        tx.send(GUIMessage::gui_update(
            Some(&format!("Paused at {:#010X}.", mcu.pc)),
            Some(mcu.leds()),
            Some(mcu.sseg()),
            Some(mcu.rf()),
            Some(mcu.pc),
            Some(mcu.fetch(|_s| {}).0),
        ))
        .unwrap();
    });

    window.show_all();
}
