extern crate i3ipc;
use std::hash::{Hash, Hasher};
use std::collections::HashMap;
use super::treewalker::*;

#[derive(Debug)]
pub struct WorkSpaceList {
    pub workspace_list: Vec<i64>,
    pub workspaces: HashMap<i64, WorkSpace>,
}

impl WorkSpaceList {
    pub fn build() -> WorkSpaceList {
        let mut wsl = WorkSpaceList {
            workspace_list: Vec::new(),
            workspaces: HashMap::new(),
        };
        build_lists(&mut wsl);
        // debug!("{:?}", wsl);

        wsl.window_on_focus(resolve_focused().unwrap());
        return wsl;
    }

    pub fn last_container(&self) {
        let current_ws = self.workspaces.get(&self.workspace_list[0]).unwrap();
        // println!("{:?}", current_ws);
        if current_ws.window_list.len() > 1 {
            let window_id = current_ws.window_list[1];
            send_command(window_id);
        } else {
            self.last_workspace();
        }
    }

    pub fn last_workspace(&self) {
        println!("{:?}", self.workspace_list);
        if self.workspace_list.len() > 2 {
            let current_ws = self.workspaces.get(&self.workspace_list[1]).unwrap();
            let window_id = current_ws.window_list[0];
            send_command(window_id);
        }
    }

    pub fn workspace_on_focus(&mut self, current_id: i64) {
        self.workspace_list.retain(|&x| x != current_id);
        self.workspace_list.insert(0, current_id);

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
        self.workspace_list.retain(|&x| x != workspace_id);
    }

    pub fn workspace_on_init(&mut self, workspace_id: i64) {
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
        match self.find_window(&window_id) {
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
        match self.find_window(&window_id) {
            Some((ws_id, index)) => {
                match self.workspaces.get_mut(&ws_id) {
                    Some(workspace) => {
                        workspace.window_list.remove(index);
                        workspace.window_list.insert(0, window_id);
                    }
                    None => {
                        println!("Window not found in list: inner");
                    }
                };
            }
            None => {
                println!("Window not found in list: outer"); //then find_it!
                self.window_on_init(window_id, None);
            }
        }
    }

    pub fn window_on_init(&mut self, window_id: i64, workspace_id: Option<i64>) {
        let find_it = || find_window_workspace_from_i3(window_id);
        match self.workspaces
            .get_mut(&workspace_id.unwrap_or_else(find_it))
        {
            Some(workspace) => {
                workspace.window_list.insert(0, window_id);
            }
            None => {
                println!("window init fail");
            }
        }
    }

    //need to move _all_ the windows of a container
    pub fn container_on_move(&mut self, container_id: i64) {
        self.window_on_close(container_id);
        match self.workspaces
            .get_mut(&find_window_workspace_from_i3(container_id))
        {
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

    fn find_window(&mut self, window_id: &i64) -> Option<(i64, usize)> {
        for (ws_id, ws) in self.workspaces.iter() {
            let mut count: usize = 0;
            for w in &ws.window_list {
                if w == window_id {
                    return Some((*ws_id, count));
                }
                count += 1;
            }
        }
        return None;
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

fn send_command(window_id: i64) {
    let ref command = format!("[con_id=\"{}\"] focus", window_id);

    i3ipc::I3Connection::connect()
        .unwrap()
        .run_command(command)
        .unwrap();
}
