extern crate nanomsg;
extern crate std;
use nanomsg::{Socket, Protocol, PollRequest, PollInOut};
use std::io::{Read, Write};
use focuswatcher::structures::WorkSpaceList;
use std::sync::Mutex;
extern crate tokio_core;
extern crate tokio_uds;
extern crate i3ipc;
use self::i3ipc::I3EventListener;
use self::i3ipc::EventIterator;
use self::i3ipc::Subscription;
use self::i3ipc::event::Event;
use std::fs;
use std::str;
use std::io;
extern crate futures;
use self::futures::*;

use self::futures::{Future, Stream};
use self::tokio_core::io::read_to_end;
use self::tokio_core::reactor::Core;
use self::tokio_uds::{UnixListener, UnixStream};
use self::futures::future::Executor;
use std::os::unix::net::UnixStream as US;

use focuswatcher::structures;
use focuswatcher;
use std::rc::Rc;
use std::sync::{Arc};

use std::cell::RefCell;
static SOCKET_FILE: &str = "/tmp/switch-it.ipc";

fn on_command(command: &String, workspace_list: &mut structures::WorkSpaceList) {
    if command.contains("w") {
        println!("switching windows");
        workspace_list.last_workspace()
    } else {
        println!("switching container");
        workspace_list.last_container()
    }
}

pub fn watch(workspace_list: &Mutex<WorkSpaceList>) {
    let mut core = Core::new().unwrap();

    let handle = core.handle();

    let mut listener = I3EventListener::connect().unwrap();


    // subscribe to a couple events.
    let subs = [Subscription::Workspace, Subscription::Window];
    listener.subscribe(&subs).unwrap();
    let l = &mut listener.listen();

    let mut stream = stream::iter_ok::<_, std::io::Error>(l);

    let server = stream.for_each(|event| {
        let mut wsl = workspace_list.lock().unwrap();
        focuswatcher::on_i3_event(&mut wsl,event.unwrap());
        println!("yep");
        Ok(())
    });
    core.run(server);

}


pub fn receiver(workspace_list:  &Mutex<WorkSpaceList>) {
    let mut core = Core::new().unwrap();

    let handle = core.handle();
    // let data = Rc::new(RefCell::new(workspace_list));

    // let w = Box::new(workspace_list);
    let listener = match UnixListener::bind(&SOCKET_FILE, &handle) {
        Ok(m) => m,
        Err(_) => {
            fs::remove_file(&SOCKET_FILE).unwrap();
            UnixListener::bind(&SOCKET_FILE, &handle).unwrap()
        }
    };

    let task = listener.incoming().for_each(|(socket, _)|  {
        let buf = Vec::new();
        // let w = workspace_list.clone();
        // let c = Arc::new(w);
        // // let g = c.clone();
        let reader = read_to_end(socket, buf)
            .map( move|(_, _buf)| {
                let command = String::from_utf8(_buf).unwrap();
                // workspace_list;
                println!("incoming: {:?}", command);
                // Ok(())
            })
            .then(|_| Ok(()));
            // .then(|_| {
            //     // let mut wsl = workspace_list.clone().lock().unwrap();
            //     // w?\
            //     // on_command(&command, &mut wsl);
            //
            //     Ok(())});

        handle.spawn(reader);
        Ok(())
    });
    // .then(|_|{
    //
    // });

    core.run(task).unwrap();
}

pub fn send(msg: String) {
    print!("{:?}", msg);
    let mut writer = US::connect(&SOCKET_FILE).unwrap();

    writer.write(msg.as_bytes()).unwrap();
}
