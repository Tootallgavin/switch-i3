#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]
extern crate switch_it;
extern crate i3ipc;
mod window_helper;
use switch_it::focuswatcher::structures::*;
use switch_it::focuswatcher::treewalker::*;
use switch_it::sockethandler::*;
use std::thread;
use window_helper::WindowHelper;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

const BASE_WINDOW_NAME: &'static str = "SItest-window";
const BASE_WORKSPACE_NAME: &'static str = "SItest-window";

fn send_command(command: &str) {
    i3ipc::I3Connection::connect()
        .unwrap()
        .command(command)
        .unwrap();
}

describe! find_window {
    before_each {
        let workspace_list = Arc::new(Mutex::new(WorkSpaceList::build()));
        let d = workspace_list.clone();

        let watch_handler = thread::spawn(move || watch(d.as_ref()));
        thread::sleep_ms(1);
    }

        ignore "checks create and find window" {
            let ref command = format!("workspace {}",BASE_WORKSPACE_NAME);

            send_command(command);

            let mut wh = WindowHelper::create_window_with_name(BASE_WINDOW_NAME);

            let ws = resolve_focused().unwrap();
            assert_eq!(BASE_WINDOW_NAME, resolve_name(ws).unwrap());
            assert_eq!(BASE_WORKSPACE_NAME, resolve_name(find_window_workspace_from_i3(ws))
            .unwrap());
            let mut wsl = workspace_list.as_ref().lock().unwrap();

            match find_window(wsl.workspaces.iter(), &ws) {
                Some((ws_id, index)) => {
                    match wsl.workspaces.get_mut(&ws_id) {
                        Some(workspace) => {
                            assert_eq!(
                                BASE_WINDOW_NAME,
                                resolve_name(*workspace.window_list
                                    .get_mut(index).unwrap()).unwrap()
                            );
                        }
                        None => {
                            assert!(false,"window not found");
                        }
                    };
                }
                None => {
                  assert!(false,"workspace not found");
                }
            }
            wh.close_window();
            send_command("workspace back_and_forth");
        }

        //create n number of windows in
        //n number of workspaces randomly closing
        ignore "checks close window and workspace" {
            let mut count = 0;
            // workspace_list: Vec::new(),
            // workspaces: HashMap::new(),
            let mut wsw :HashMap<i64, Vec<i64>> = HashMap::new();
            loop{
                let ws_name = format!("{}-{}",BASE_WORKSPACE_NAME,count);
                let win_name = format!("{}-{}",BASE_WINDOW_NAME,count);
                send_command(&format!("workspace {}",ws_name));

                let mut wh = WindowHelper::create_window_with_name(&win_name);
                let win_id = resolve_focused().unwrap();
                assert_eq!(win_name, resolve_name(win_id).unwrap());
                let ws_id = find_window_workspace_from_i3(win_id);
                assert_eq!(ws_name, resolve_name(ws_id).unwrap());

                wsw.insert(ws_id, Vec::new());
                wsw.get_mut(&ws_id).unwrap().insert(0,win_id);


                count += 1;
                if count == 5{
                    break;
                }
            }

        }
}
