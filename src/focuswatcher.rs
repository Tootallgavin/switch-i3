extern crate i3ipc;
use self::i3ipc::I3EventListener;
use self::i3ipc::Subscription;
use self::i3ipc::event::Event;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::ops::Deref;
use std::collections::hash_map::Iter;

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
        let window_id = current_ws.window_list[1];
        send_command(window_id);
    }

    pub fn last_workspace(&self) {
        let current_ws = self.workspaces.get(&self.workspace_list[1]).unwrap();
        let window_id = current_ws.window_list[0];
        send_command(window_id);
    }

    fn workspace_on_focus(&mut self, current_id: i32) {
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

    fn workspace_on_empty(&mut self, workspace_id: i32) {
        self.workspace_list.retain(|&x| x != workspace_id);
    }

    fn workspace_on_init(&mut self, workspace_id: i32) {
        self.workspaces
            .insert(workspace_id,
                    WorkSpace {
                        id: workspace_id,
                        window_list: Vec::new(),
                    });
        self.workspace_list.insert(0, workspace_id)
    }

    fn window_on_close(&mut self, window_id: i32) {
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

    fn window_on_focus(&mut self, window_id: i32) {
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

    fn window_on_init(&mut self, window_id: i32, workspace_id: Option<i32>) {
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

    fn container_on_move(&mut self, container_id: i32){

    }
}

fn send_command(window_id: i32) {
    let ref command = format!("[con_id=\"{}\"] focus", window_id);
    let result = i3ipc::I3Connection::connect()
        .unwrap()
        .command(command)
        .unwrap();
}

fn find_window(iter: Iter<i32, WorkSpace>, window_id: &i32) -> Option<(i32, usize)> {
    for (ws_id, ws) in iter {
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

fn build_lists(wsl: &mut WorkSpaceList) {
    let rootnode = i3ipc::I3Connection::connect().unwrap().get_tree().unwrap();
    walk_tree(wsl, rootnode, 0);
}

fn resolve_name(id: i32) -> Option<String> {
    let rootnode = i3ipc::I3Connection::connect().unwrap().get_tree().unwrap();
    return wt(rootnode, id);
}

fn wt(node: i3ipc::reply::Node, id: i32) -> Option<String> {
    let mut name: Option<String> = None;
    for node in node.nodes {
        if name.is_some() {
            return name;
        }
        match node.nodetype {
            i3ipc::reply::NodeType::Output => {
                name = wt(node, id);
            }
            i3ipc::reply::NodeType::Workspace => {
                if id == node.id {
                    return node.name;
                }
                name = wt(node, id);
            }
            i3ipc::reply::NodeType::Con => {
                match node.window {
                    Some(_) => {
                        if id == node.id {
                            return node.name;
                        }
                    }
                    None => {
                        return wt(node, id);
                    }
                }
            }
            i3ipc::reply::NodeType::FloatingCon => {
                println!("F");
            }
            _ => {}
        }
    }
    return name;
}

fn walk_tree(wsl: &mut WorkSpaceList, rootnode: i3ipc::reply::Node, workspace_id: i32) {
    for node in rootnode.nodes {
        match node.nodetype {
            i3ipc::reply::NodeType::Output => {
                walk_tree(wsl, node, workspace_id);
            }
            i3ipc::reply::NodeType::Workspace => {
                let c = node.id;
                wsl.workspace_on_init(node.id);
                walk_tree(wsl, node, c);
            }
            i3ipc::reply::NodeType::Con => {
                match node.window {
                    Some(_) => {
                        wsl.window_on_init(node.id, Some(workspace_id));
                    }
                    None => {
                        walk_tree(wsl, node, workspace_id);
                    }
                }
            }
            i3ipc::reply::NodeType::FloatingCon => {
                println!("F");
            }
            _ => {}
        }
    }
}

pub fn find_window_workspace_from_i3(window_id: i32) -> i32 {
    let rootnode = i3ipc::I3Connection::connect().unwrap().get_tree().unwrap();
    // println!("{:?}", rootnode);
    return walk_to_resolve_windows_workspace(rootnode, window_id, 0);
}

fn walk_to_resolve_windows_workspace(rootnode: i3ipc::reply::Node,
                                     window_id: i32,
                                     current_workspace_id: i32)
                                     -> i32 {
    let mut found = 0;
    for node in rootnode.nodes {
        if found != 0 {
            println!("exit");
            return found;
        }
        match node.nodetype {
            i3ipc::reply::NodeType::Output => {
                found = walk_to_resolve_windows_workspace(node, window_id, current_workspace_id);
            }
            i3ipc::reply::NodeType::Workspace => {
                let c = node.id;
                found = walk_to_resolve_windows_workspace(node, window_id, c);
            }
            i3ipc::reply::NodeType::Con => {
                match node.window {
                    Some(_) => {
                        if node.id == window_id {
                            return current_workspace_id;
                        }
                    }
                    None => {
                        return walk_to_resolve_windows_workspace(node,
                                                                 window_id,
                                                                 current_workspace_id);
                    }
                }
            }
            _ => {}
        }
    }
    return found;
}


#[derive(Debug)]
struct WorkSpace {
    id: i32,
    window_list: Vec<i32>,
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

pub fn watch(workspace_list: &Mutex<WorkSpaceList>) {
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
