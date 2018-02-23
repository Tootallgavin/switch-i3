extern crate futures;
extern crate i3ipc;
extern crate tokio_core;
extern crate tokio_uds;
use std::io::{Error, Read, Write};
use std::fs;
use std::str;
use std::process;
use std::os::unix::net::UnixStream as US;
use std::os::unix::net::UnixListener;
use i3ipc::I3EventListener;
use i3ipc::Subscription;
use self::futures::Stream;
use self::futures::stream::iter_ok;
use self::tokio_core::reactor::Core;
use focuswatcher::on_i3_event;
use focuswatcher::structures::WorkSpaceList;
use std::thread;
use std::sync::{Arc, Mutex};

//used to communicate between switch-it processes
static SOCKET_FILE: &str = "/tmp/switch-it.ipc";

///creates two threads: one responds to events sent by i3, the other to respond to events from the socket_file
//TODO: want to make this single threaded with tokio 
pub fn set_up_watch() {
    let workspace_list = Arc::new(Mutex::new(WorkSpaceList::build()));
    let c = workspace_list.clone();
    let d = workspace_list.clone();

    let watch_handler = thread::spawn(move || watch(c.as_ref()));
    let reciver_handler = thread::spawn(move || receiver(d.as_ref()));

    watch_handler.join().unwrap();
    reciver_handler.join().unwrap();
}

///performs the command sent from the socket_file
fn on_command(command: &String, workspace_list: &mut WorkSpaceList) {
    if command.contains("w") {
        // println!("switching windows");
        workspace_list.last_workspace()
    } else {
        // println!("switching container");
        workspace_list.last_container()
    }
}

///watches i3 for workspace and window events 
fn watch(workspace_list: &Mutex<WorkSpaceList>) {
    let mut core = Core::new().unwrap();
    let mut listener = I3EventListener::connect().unwrap();

    // subscribe to window and workspace events.
    let subs = [Subscription::Workspace, Subscription::Window];
    listener.subscribe(&subs).unwrap();
    let l = &mut listener.listen();

    let stream = iter_ok::<_, Error>(l);

    let server = stream.for_each(|event| {
        let mut wsl = workspace_list.lock().unwrap();
        on_i3_event(&mut wsl, event.unwrap());
        Ok(())
    });
    core.run(server).unwrap();
}

///recieves commands from the socket_file 
fn receiver(workspace_list: &Mutex<WorkSpaceList>) {
    let listener = UnixListener::bind(&SOCKET_FILE).unwrap_or_else(|_| {
        fs::remove_file(&SOCKET_FILE) // stale socket file, delete it
            .and(UnixListener::bind(&SOCKET_FILE)) 
            .unwrap()
    });

    // accept connections and process them
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                /* connection succeeded */
                let mut command: String = String::new();
                stream.read_to_string(&mut command).unwrap();

                let mut wsl = workspace_list.lock().unwrap();
                on_command(&command, &mut wsl)
            }
            Err(_) => {
                /* connection failed */
                break;
            }
        }
    }
}

///send a command to the reciever thread
pub fn send(msg: String) {
    match US::connect(&SOCKET_FILE).and_then(|mut writer| {
        print!("{:?}", msg);
        writer.write(msg.as_bytes())
    }) {
        Ok(_) => {}
        Err(_) => {
            println!("Daemon not running");
            process::exit(0x0001);
        }
    }
}
