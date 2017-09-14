use std::hash::{Hash, Hasher};
use std::collections::HashMap;
extern crate i3ipc;

use super::treewalker::*;


#[derive(Debug)]
pub struct WorkSpaceList {
    workspace_list: Vec<i32>,
    workspaces: HashMap<i32, WorkSpace>,
}

impl WorkSpaceList {
    pub fn build() -> WorkSpaceList {
        let mut wsl = WorkSpaceList {
            workspace_list: Vec::new(),
            workspaces: HashMap::new(),
        };
        build_lists(&mut wsl);
        return wsl;
    }

    pub fn last_container(&self) {
        let current_ws = self.workspaces.get(&self.workspace_list[0]).unwrap();

        if current_ws.window_list.len() > 2 {
          let window_id = current_ws.window_list[1];
          send_command(window_id);
        } else {
          self.last_workspace();
      }
    }

    pub fn last_workspace(&self) {
        let current_ws = self.workspaces.get(&self.workspace_list[1]).unwrap();
        let window_id = current_ws.window_list[0];
        send_command(window_id);
    }

    pub fn workspace_on_focus(&mut self, current_id: i32) {
        self.workspace_list.retain(|&x| x != current_id);
        self.workspace_list.insert(0, current_id);

        if !self.workspaces.contains_key(&current_id) {
            self.workspaces
                .insert(current_id,
                        WorkSpace {
                            id: current_id,
                            window_list: Vec::new(),
                        });
        }
    }

    pub fn workspace_on_empty(&mut self, workspace_id: i32) {
        self.workspace_list.retain(|&x| x != workspace_id);
    }

    pub fn workspace_on_init(&mut self, workspace_id: i32) {
        self.workspaces
            .insert(workspace_id,
                    WorkSpace {
                        id: workspace_id,
                        window_list: Vec::new(),
                    });
        self.workspace_list.insert(0, workspace_id)
    }

    pub fn window_on_close(&mut self, window_id: i32) {
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

    pub fn window_on_focus(&mut self, window_id: i32) {
        match find_window(self.workspaces.iter(), &window_id) {
            Some((ws_id, index)) => {
                match self.workspaces.get_mut(&ws_id) {
                    Some(workspace) => {
                        workspace.window_list.remove(index);
                        workspace.window_list.insert(0, window_id);
                    }
                    None => {
                        println!("Ha");
                    }
                };
            }
            None => {
                println!("Ha2");
            }
        }
    }

    pub fn window_on_init(&mut self, window_id: i32, workspace_id: Option<i32>) {
        let find_it = || find_window_workspace_from_i3(window_id);

        match self.workspaces
                  .get_mut(&workspace_id.unwrap_or_else(find_it)) {
            Some(workspace) => {
                workspace.window_list.insert(0, window_id);
            }
            None => {
                println!("init fail");
            }
        }
    }

    pub fn container_on_move(&mut self, container_id: i32){
          self.window_on_close(container_id);
          match self.workspaces
                    .get_mut(&find_window_workspace_from_i3(container_id)) {
              Some(workspace) => {
                  workspace.window_list.insert(0, container_id);
                  let index = self.workspace_list.iter().position(|&r| r == workspace.id).unwrap();
                  self.workspace_list.remove(index);
                  self.workspace_list.insert(1, workspace.id)
              }
              None => {
                  println!("init fail");
              }
          }
        }
}

#[derive(Debug)]
pub struct WorkSpace {
    id: i32,
    pub window_list: Vec<i32>,
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

fn send_command(window_id: i32) {
    let ref command = format!("[con_id=\"{}\"] focus", window_id);
    let result = i3ipc::I3Connection::connect()
        .unwrap()
        .command(command)
        .unwrap();
}
