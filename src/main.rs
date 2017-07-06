extern crate nanomsg;
extern crate clap;
use clap::{App, SubCommand};
mod focuswatcher;
mod sockethandler;
use std::thread;
use std::sync::{Arc, Mutex};


fn set_up_watch() {
    println!("setup");
    let workspace_list = Arc::new(Mutex::new(focuswatcher::WorkSpaceList::build()));
    let c = workspace_list.clone();
    let d = workspace_list.clone();

    let fhandler = thread::spawn(move || focuswatcher::watch(c.as_ref()));
    let rhandler = thread::spawn(move || sockethandler::receiver(d.as_ref()));

    fhandler.join().unwrap();
    rhandler.join().unwrap();
}

fn main() {
    let matches = App::new("switch-it")
        .version("1.0")
        .author("Gavin Stringfellow")
        .about("focus on different windows")
        .args_from_usage("-w 'Switches to last focused window in last focused workspace'
                          -c 'Switches to last focused window in current container'")
        .subcommand(SubCommand::with_name("watch"))
        .get_matches();
    // focuswatcher::find_window_workspace_from_i3(33105776);
    // set_up_watch();
    if matches.is_present("watch") {
        set_up_watch();
    } else if matches.is_present("w") {
        sockethandler::send(String::from("w"));
    } else {
        sockethandler::send(String::from("c"));
    }
}
