extern crate nanomsg;
use nanomsg::{Socket, Protocol, PollRequest, PollInOut};
use std::io::{Read, Write};
use focuswatcher::structures::WorkSpaceList;
use std::sync::Mutex;

static SOCKET_FILE: &str = "ipc:///tmp/switch-it.ipc";

fn on_command(command: &String, workspace_list_mute: &Mutex<WorkSpaceList>) {
    let ref wsl = workspace_list_mute.lock().unwrap();
    if command.contains("w") {
        // or c
        wsl.last_workspace()
    } else {
        wsl.last_container()
    }
}


pub fn receiver(workspace_list: &Mutex<WorkSpaceList>) {
    let mut socket = Socket::new(Protocol::Pull).unwrap();
    let _ = socket.bind(&SOCKET_FILE); // let _ = means we don't need any return value stored somewere
    let mut request = String::new();

    loop {
        match socket.read_to_string(&mut request) {
            Ok(_) => on_command(&request, workspace_list),
            Err(err) => {
                println!("failed '{}'", err);
                break;
            }
        }
        request.clear();
    }
}

fn can_write_to_pipe(socket: &Socket) -> bool {
    let mut pollfd = [socket.new_pollfd(PollInOut::Out)];
    let mut poll_req = PollRequest::new(&mut pollfd);
    let _ = Socket::poll(&mut poll_req, 10);
    return poll_req.get_fds()[0].can_write();
}

pub fn send(msg: String) {
    let mut socket = Socket::new(Protocol::Push).unwrap();
    socket.connect(&SOCKET_FILE).unwrap();

    if can_write_to_pipe(&socket) {
        match socket.write_all(&msg.as_bytes()) {
            Ok(_) => println!("SENDING '{}'", &msg),
            Err(err) => {
                println!("failed '{}'", err);
            }
        }
    } else {
        println!("nope");
    }
}
