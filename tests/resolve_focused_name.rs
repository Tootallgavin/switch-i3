#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]
extern crate switch_it;
mod window_helper;
use switch_it::focuswatcher::structures::*;
use switch_it::focuswatcher::treewalker::*;
use window_helper::WindowHelper;
use std::thread;

describe! top_level {
        it "checks resolve focused and name" {
            const NAME: &'static str = "test124";
            let mut wh = WindowHelper::create_window_with_name(NAME);
            thread::sleep_ms(2);
            let ws = resolve_focused();

            assert_eq!(NAME, resolve_name(ws.unwrap()).unwrap());
            
            wh.close_window();
        }
}

//     fn new_workspace_window() {
//         const WINDOW_NAME: &'static str = "test120";
//         const WORKSPACE_NAME: &'static str = "test120233";
//
//         let ref command = format!("workspace {:?}",WORKSPACE_NAME);
//         let workspace_list = Arc::new(Mutex::new(WorkSpaceList::build()));
//         let d = workspace_list.clone();
//
//         let watch_handler = thread::spawn(move || sockethandler::watch(d.as_ref()));
//         thread::sleep_ms(1); //
//
//         i3ipc::I3Connection::connect()
//             .unwrap()
//             .command(command)
//             .unwrap();
//         let mut wh = WindowHelper::build();
//
//
//         wh.open_window_with_name(WINDOW_NAME);
//
//         let ws = resolve_focused().unwrap();
//         assert_eq!(WINDOW_NAME, resolve_name(ws).unwrap());
//         assert_eq!(WORKSPACE_NAME, resolve_name(find_window_workspace_from_i3(ws)).unwrap());
//         let mut wsl = workspace_list.as_ref().lock().unwrap();
//
//         match find_window(wsl.workspaces.iter(), &ws) {
//             Some((ws_id, index)) => {
//                 match wsl.workspaces.get_mut(&ws_id) {
//                     Some(workspace) => {
//                         assert_eq!(
//                             WINDOW_NAME,
//                             resolve_name(*workspace.window_list.get_mut(index).unwrap()).unwrap()
//                         );
//                     }
//                     None => {
//                         assert!(false);
//                     }
//                 };
//             }
//             None => {
//               assert!(false,"workspace not found");
//
//             }
//         }
//         wh.close_window();
//         i3ipc::I3Connection::connect()
//             .unwrap()
//             .command("workspace back_and_forth")
//             .unwrap();
//     }
