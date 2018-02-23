extern crate clap;
extern crate futures;
extern crate i3ipc;
extern crate tokio_core;
extern crate tokio_uds;
pub mod focuswatcher;
mod sockethandler;
use clap::{App, SubCommand};

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
        sockethandler::set_up_watch();
    } else if matches.is_present("w") {
        sockethandler::send(String::from("w"));
    } else {
        sockethandler::send(String::from("c"));
    }
}
