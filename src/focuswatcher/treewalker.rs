use super::structures::*;
use std::collections::hash_map::Iter;
extern crate i3ipc;
use std::cell::RefCell;

pub fn find_window(iter: Iter<i64, WorkSpace>, window_id: &i64) -> Option<(i64, usize)> {
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

fn get_tree() -> i3ipc::reply::Node {
    if cfg!(test) {
        return i3ipc::I3Connection::connect().unwrap().get_tree().unwrap();
    } else {
        return i3ipc::I3Connection::connect().unwrap().get_tree().unwrap();
    }
}

pub fn build_lists(wsl: &mut WorkSpaceList) {
    let rootnode = get_tree();
    // println!("{:?}", rootnode);
    walk_tree_bl(wsl, rootnode, 0);
}

pub fn resolve_name(id: i64) -> Option<String> {
    let rootnode = get_tree();
    return walk_tree_rn(rootnode, id);
}

type Bon = Box<Option<i3ipc::reply::Node>>;

#[derive(Clone, Copy)]
struct TreeWalkerHelper<'a> {
    output:&'a i3ipc::reply::Node,
    workspace: &'a i3ipc::reply::Node,
    container: &'a  i3ipc::reply::Node
}
use focuswatcher::core::clone::Clone;
fn walk_tree<'a>(node: &'a i3ipc::reply::Node,twh:&'a mut  TreeWalkerHelper, onNode: &mut Fn(i3ipc::reply::Node, TreeWalkerHelper)) {
    // let next =  |mut node| walk_tree(node, twh, onNode);
    let &'a mut s =  TreeWalkerHelper {output: None, workspace: None, container: None};
    // let bn  = *node;
    for node in node.nodes {
        match node.nodetype {
            i3ipc::reply::NodeType::Output => {
                twh.output = &node;
                walk_tree(&node, twh, onNode);
                // (next)(twh.output);
            }
            i3ipc::reply::NodeType::Workspace => {
                twh.workspace =  &node;
                onNode(node,*twh);
                // next(RefCell::from(node));
            }
            i3ipc::reply::NodeType::Con => {
                match node.window {
                    Some(_) => {
                        // onNode(node,&twh);
                        // walk_tree(node);
                    }
                    None => {
                        twh.container =  &node;
                        // onNode(node);
                        // next(RefCell::from(node));
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

fn walk_tree_rn(node: i3ipc::reply::Node, id: i64) -> Option<String> {
    let mut name: Option<String> = None;
    for node in node.nodes {
        if name.is_some() {
            return name;
        }
        match node.nodetype {
            i3ipc::reply::NodeType::Output => {
                name = walk_tree_rn(node, id);
            }
            i3ipc::reply::NodeType::Workspace => {
                if id == node.id {
                    return node.name;
                }
                name = walk_tree_rn(node, id);
            }
            i3ipc::reply::NodeType::Con => {
                match node.window {
                    Some(_) => {
                        if id == node.id {
                            return node.name;
                        }
                    }
                    None => {
                        return walk_tree_rn(node, id);
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

pub fn resolve_focused() -> Option<i64> {
    let rootnode = get_tree();
    return walk_tree_f(rootnode);
}

fn walk_tree_f(node: i3ipc::reply::Node) -> Option<i64> {
    let mut id: Option<i64> = None;
    for node in node.nodes {
        if id.is_some() {
            return id;
        }
        match node.nodetype {
            i3ipc::reply::NodeType::Output => {
                id = walk_tree_f(node);
            }
            i3ipc::reply::NodeType::Workspace => {
                id = walk_tree_f(node);
            }
            i3ipc::reply::NodeType::Con => {
                match node.window {
                    Some(_) => {
                        if node.focused {
                            return Some(node.id);
                        }
                    }
                    None => {
                        return walk_tree_f(node);
                    }
                }
            }
            i3ipc::reply::NodeType::FloatingCon => {
                println!("F");
            }
            _ => {}
        }
    }
    return id;
}

fn walk_tree_bl(wsl: &mut WorkSpaceList, rootnode: i3ipc::reply::Node, workspace_id: i64) {
    for node in rootnode.nodes {
        match node.nodetype {
            i3ipc::reply::NodeType::Output => {
                walk_tree_bl(wsl, node, workspace_id);
            }
            i3ipc::reply::NodeType::Workspace => {
                let c = node.id;
                wsl.workspace_on_init(node.id);
                walk_tree_bl(wsl, node, c);
            }
            i3ipc::reply::NodeType::Con => {
                match node.window {
                    Some(_) => {
                        wsl.window_on_init(node.id, Some(workspace_id));
                    }
                    None => {
                        walk_tree_bl(wsl, node, workspace_id);
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

pub fn find_window_workspace_from_i3(window_id: i64) -> i64 {
    let rootnode = get_tree();
    // println!("{:?}", rootnode);
    return walk_to_resolve_windows_workspace(rootnode, window_id, 0);
}

fn walk_to_resolve_windows_workspace(
    rootnode: i3ipc::reply::Node,
    window_id: i64,
    current_workspace_id: i64,
) -> i64 {
    let mut found :i64 = 0;
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
                        return walk_to_resolve_windows_workspace(
                            node,
                            window_id,
                            current_workspace_id,
                        );
                    }
                }
            }
            _ => {}
        }
    }
    return found;
}
