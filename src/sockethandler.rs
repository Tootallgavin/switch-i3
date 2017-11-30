extern crate futures;
extern crate tokio_core;
extern crate tokio_uds;
extern crate i3ipc;
use std::io::{Read, Write,Error};
use std::sync::Mutex;
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
use std::time::Instant;

static SOCKET_FILE: &str = "/tmp/switch-it.ipc";

fn on_command(command: &String, workspace_list: &mut WorkSpaceList) {
    if command.contains("w") {
        // println!("switching windows");
        workspace_list.last_workspace()
    } else {
        // println!("switching container");
        workspace_list.last_container()
    }
}

pub fn watch(workspace_list: &Mutex<WorkSpaceList>) {
    let mut core = Core::new().unwrap();
    let mut listener = I3EventListener::connect().unwrap();

    // subscribe to window and workspace event.
    let subs = [Subscription::Workspace, Subscription::Window];
    listener.subscribe(&subs).unwrap();
    let l = &mut listener.listen();

    let stream = iter_ok::<_, Error>(l);

    let server = stream.for_each(|event| {
        let now = Instant::now();
        // println!("start {:?}", now);
        let mut wsl = workspace_list.lock().unwrap();
        on_i3_event(&mut wsl, event.unwrap());
        println!("end {:?}", now.elapsed());

        Ok(())
    });
    core.run(server).unwrap();
}


pub fn receiver(workspace_list: &Mutex<WorkSpaceList>) {

    let listener = UnixListener::bind(&SOCKET_FILE).unwrap_or_else(|_| {
        fs::remove_file(&SOCKET_FILE)
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
