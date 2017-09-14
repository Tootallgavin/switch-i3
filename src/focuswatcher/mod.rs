extern crate i3ipc;
use self::i3ipc::I3EventListener;
use self::i3ipc::Subscription;
use self::i3ipc::event::Event;
use std::sync::Mutex;
use std::ops::Deref;

pub mod structures;
mod treewalker;

pub fn watch(workspace_list: &Mutex<structures::WorkSpaceList>) {
    let mut listener = I3EventListener::connect().unwrap();


    // subscribe to a couple events.
    let subs = [Subscription::Workspace, Subscription::Window];
    listener.subscribe(&subs).unwrap();

    // handle them
    for event in listener.listen() {
        let mut wsl = workspace_list.lock().unwrap();
        match event.unwrap() {
            Event::WorkspaceEvent(e) => {
                match e.change {
                    i3ipc::event::inner::WorkspaceChange::Empty => {
                        wsl.workspace_on_empty(e.current.unwrap().id)
                    }
                    i3ipc::event::inner::WorkspaceChange::Focus => {
                        wsl.workspace_on_focus(e.current.unwrap().id)
                    }
                    i3ipc::event::inner::WorkspaceChange::Init => {
                        wsl.workspace_on_init(e.current.unwrap().id)
                    }
                    _ => println!("Urgent"),
                }
            }
            Event::WindowEvent(e) => {
                match e.change {
                    i3ipc::event::inner::WindowChange::Close => wsl.window_on_close(e.container.id),
                    i3ipc::event::inner::WindowChange::Focus => wsl.window_on_focus(e.container.id),
                    i3ipc::event::inner::WindowChange::New => {
                        wsl.window_on_init(e.container.id, None)
                    }

                    i3ipc::event::inner::WindowChange::Move => {
                        wsl.container_on_move(e.container.id);
                    }
                    _ => println!("Urgent"),
                }
            }
            _ => unreachable!(),
        }
    }
}
