use std::hash::{Hash, Hasher};
use std::collections::HashMap;
extern crate i3ipc;
use super::treewalker::*;

///A representation of the i3 tree of nodes
#[derive(Debug)]
pub struct WorkSpaceList {
    ///a list of all workspace
    pub workspace_list: Vec<i64>,
    ///a hashmap whose key is a workspace id
    ///and value is a list of all the windows
    ///contained in the workspace 
    pub workspaces: HashMap<i64, WorkSpace>,
}

impl WorkSpaceList {
    pub fn build() -> WorkSpaceList {
        let mut wsl = WorkSpaceList {
            workspace_list: Vec::new(),
            workspaces: HashMap::new(),
        };
        build_lists(&mut wsl);
        // seth the current focused window to be at the head of the list
        // wsl.window_on_focus(resolve_focused().unwrap());
        return wsl;
    }

    ///attempts to switch to the last focused window in the current workspace
    ///if there are no other windows in the current workspace
    ///it will call last_workspace
    pub fn last_container(&self) {
        
        let current_ws = self.workspaces.get(&self.workspace_list[0]).unwrap();

        if current_ws.window_list.len() > 2 {
            let window_id = current_ws.window_list[1];
            send_command(window_id);
            debug!("last_container: {:?}", window_id);
        } else {
            self.last_workspace();
        }
    }

    ///switches to the last focused window of the last focused workspace
    pub fn last_workspace(&self) {
        if self.workspace_list.len() > 2 {
            let current_ws = self.workspaces.get(&self.workspace_list[1]).unwrap();
            let window_id = current_ws.window_list[0];
            send_command(window_id);
            debug!("last_workspace: {:?} container: {:?}",current_ws, window_id);
        }
    }

    pub fn workspace_on_focus(&mut self, current_id: i64) {
        debug!("workspace_on_focus: {:?}", current_id);
        //delete the just focused workspace from the list
        self.workspace_list.retain(|&x| x != current_id);
        //and move it to the front
        self.workspace_list.insert(0, current_id);

        //if the workspace does not have a key in the hashmap
        //create one and instantiate a workspace
        if !self.workspaces.contains_key(&current_id) {
            self.workspaces.insert(
                current_id,
                WorkSpace {
                    id: current_id,
                    window_list: Vec::new(),
                },
            );
        }
    }

    pub fn workspace_on_empty(&mut self, workspace_id: i64) {
        debug!("workspace_on_empty: {:?}", workspace_id);
        //remove workspace from our data
        self.workspace_list.retain(|&x| x != workspace_id);
        self.workspaces.remove(workspace_id);
    }


    pub fn workspace_on_init(&mut self, workspace_id: i64) {
        debug!("workspace_on_init: {:?}", workspace_id);
        self.workspaces.insert(
            workspace_id,
            WorkSpace {
                id: workspace_id,
                window_list: Vec::new(),
            },
        );
        self.workspace_list.insert(0, workspace_id)
    }

    pub fn window_on_close(&mut self, window_id: i64) {
        debug!("window_on_close: {:?}", window_id);
        //we only get the window ID, so we need to find the 
        //the parent workspace so we can delete the window 
        //from our data
        match find_window(self.workspaces.iter(), &window_id) {
            Some((ws_id, index)) => {
                match self.workspaces.get_mut(&ws_id) {
                    Some(workspace) => {
                        workspace.window_list.remove(index);
                    }
                    None => {}
                };
            }
            None => {}
        }
    }

    pub fn window_on_focus(&mut self, window_id: i64) {
        debug!("window_on_focus: {:?}", window_id);
        match find_window(self.workspaces.iter(), &window_id) {
            Some((ws_id, index)) => {
                match self.workspaces.get_mut(&ws_id) {
                    Some(workspace) => {
                        workspace.window_list.remove(index);
                        workspace.window_list.insert(0, window_id);
                    }
                    None => {
                        //then find_it!... maybe
                        debug!("Window not found in list");
                    }
                };
            }
            None => {
                debug!("Window not found in list, creating"); //then find_it!
                self.window_on_init(window_id,None);
            }
        }
    }

    pub fn window_on_init(&mut self, window_id: i64, workspace_id: Option<i64>) {
        debug!("window_on_init: container:{:?} window:", workspace_id, window_id);
        let find_it = || find_window_workspace_from_i3(window_id);
        match self.workspaces.get_mut(
            &workspace_id.unwrap_or_else(find_it),
        ) {
            Some(workspace) => {
                workspace.window_list.insert(0, window_id);
            }
            None => {
                //this should be unreachable.... what happend to find_it
                debug!("window init fail");
            }
        }
    }

    //need to move _all_ the windows of a container
    pub fn container_on_move(&mut self, container_id: i64) {
        debug!("container_on_move: {:?}", container_id);
        self.window_on_close(container_id);
        match self.workspaces.get_mut(
            &find_window_workspace_from_i3(container_id),
        ) {
            Some(workspace) => {
                workspace.window_list.insert(0, container_id);
                let index = self.workspace_list
                    .iter()
                    .position(|&r| r == workspace.id)
                    .unwrap();
                self.workspace_list.remove(index);
                self.workspace_list.insert(1, workspace.id)
            }
            None => {
                println!("container move fail");
            }
        }
    }
}

#[derive(Debug)]
pub struct WorkSpace {
    pub id: i64,
    pub window_list: Vec<i64>,
}

impl PartialEq for WorkSpace {
    fn eq(&self, other: &WorkSpace) -> bool {
        self.id == other.id
    }
}
impl Hash for WorkSpace {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.window_list.hash(state);
    }
}
impl Eq for WorkSpace {}

///sends a command to i3 
///particularly what to focus on
fn send_command(window_id: i64) {
    let ref command = format!("[con_id=\"{}\"] focus", window_id);

    i3ipc::I3Connection::connect()
        .unwrap()
        .command(command)
        .unwrap();
}
