extern crate i3ipc;
use i3ipc::event::{self, Event};
pub mod structures;
pub mod treewalker;
use focuswatcher;

pub fn on_i3_event(
    workspace_list: &mut structures::WorkSpaceList,
    event: self::focuswatcher::i3ipc::event::Event,
) {
    match event {
        Event::WorkspaceEvent(e) => {
            match e.change {
                i3ipc::event::inner::WorkspaceChange::Empty => {
                    workspace_list.workspace_on_empty(e.current.unwrap().id)
                }
                i3ipc::event::inner::WorkspaceChange::Focus => {
                    workspace_list.workspace_on_focus(e.current.unwrap().id)
                }
                i3ipc::event::inner::WorkspaceChange::Init => {
                    workspace_list.workspace_on_init(e.current.unwrap().id)
                }
                _ => println!("Not Implemented"),
            }
        }
        Event::WindowEvent(e) => {
            match e.change {
                i3ipc::event::inner::WindowChange::Close => {
                    workspace_list.window_on_close(e.container.id)
                }
                i3ipc::event::inner::WindowChange::Focus => {
                    workspace_list.window_on_focus(e.container.id)
                }
                i3ipc::event::inner::WindowChange::New => {
                    workspace_list.window_on_init(e.container.id, None)
                }

                i3ipc::event::inner::WindowChange::Move => {
                    workspace_list.container_on_move(e.container.id);
                }
                _ => println!("Urgent"),
            }
        }
        _ => unreachable!(),
    }
}
