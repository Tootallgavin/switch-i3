use super::structures::*;
use std::collections::hash_map::Iter;
extern crate i3ipc;

pub fn find_window(iter: Iter<i32, WorkSpace>, window_id: &i32) -> Option<(i32, usize)> {
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

pub fn build_lists(wsl: &mut WorkSpaceList) {
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
