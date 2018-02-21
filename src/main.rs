extern crate clap;
extern crate futures;
extern crate i3ipc;
extern crate tokio_core;
extern crate tokio_uds;
pub mod focuswatcher;
mod sockethandler;
use clap::{App, SubCommand};
use std::thread;
use std::sync::{Arc, Mutex};

fn set_up_watch() {
    let workspace_list = Arc::new(Mutex::new(focuswatcher::structures::WorkSpaceList::build()));
    let c = workspace_list.clone();
    let d = workspace_list.clone();

    let watch_handler = thread::spawn(move || sockethandler::watch(c.as_ref()));
    let reciver_handler = thread::spawn(move || sockethandler::receiver(d.as_ref()));

    watch_handler.join().unwrap();
    reciver_handler.join().unwrap();
}

fn main() {
    let matches = App::new("switch-it")
        .version("1.0")
        .author("Gavin Stringfellow")
        .about("focus on different windows")
        .args_from_usage(
            "-w 'Switches to last focused window in last focused workspace'
                          -c 'Switches to last focused window in current container'",
        )
        .subcommand(SubCommand::with_name("watch"))
        .get_matches();

    if matches.is_present("watch") {
        set_up_watch();
    } else if matches.is_present("w") {
        sockethandler::send(String::from("w"));
    } else {
        sockethandler::send(String::from("c"));
    }
}
