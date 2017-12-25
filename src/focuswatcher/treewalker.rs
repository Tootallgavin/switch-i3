extern crate i3ipc;
use super::structures::*;
use std::collections::hash_map::Iter;
use std::boxed::Box;
use std::borrow::BorrowMut;
use std::mem;

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

pub fn build_lists(wsl: &mut WorkSpaceList) {
    let mut treeWalker = TreeWalker::new();

    let mut logic = &mut |mut treeWalker: TreeWalker<i64>| -> TreeWalker<i64> {
        if treeWalker.workspace.is_some() && treeWalker.window.is_none() {
            let workspace = treeWalker.workspace.clone().unwrap();
            wsl.workspace_on_init(workspace.id);
        }

        if treeWalker.window.is_some() {
            let window = treeWalker.window.clone().unwrap();
            let workspace = treeWalker.workspace.clone().unwrap();
            wsl.window_on_init(window.id, Some(workspace.id));
        }

        return treeWalker;
    };

    walk_tree(treeWalker, logic);
}

pub fn resolve_name(id: i64) -> Option<String> {
    let mut treeWalker = TreeWalker::new();

    let mut logic = &mut |mut treeWalker: TreeWalker<String>| -> TreeWalker<String> {
        if treeWalker.workspace.is_some() && treeWalker.window.is_none() {
            let workspace = treeWalker.workspace.clone().unwrap();
            if workspace.id == id {
                treeWalker.result = workspace.name;
            }
        }

        if treeWalker.window.is_some() {
            let window = treeWalker.window.clone().unwrap();
            if window.id == id {
                treeWalker.result = window.name;
            }
        }

        return treeWalker;
    };

    walk_tree(treeWalker, logic).result
}

pub fn resolve_focused() -> Option<i64> {
    let mut treeWalker = TreeWalker::new();

    let mut logic = &mut |mut treeWalker: TreeWalker<i64>| -> TreeWalker<i64> {
        if treeWalker.window.is_some() {
            let window = treeWalker.window.clone().unwrap();
            if window.focused {
                treeWalker.result = Some(window.id);
            }
        }

        return treeWalker;
    };

    walk_tree(treeWalker, logic).result
}

pub fn find_window_workspace_from_i3(window_id: i64) -> i64 {
    let mut treeWalker = TreeWalker::new();

    let mut logic = &mut |mut treeWalker: TreeWalker<i64>| -> TreeWalker<i64> {
        if treeWalker.window.is_some() {
            let window = treeWalker.window.clone().unwrap();
            let workspace = treeWalker.workspace.clone().unwrap();
            if window.id == window_id {
                treeWalker.result = Some(workspace.id);
            }
        }

        return treeWalker;
    };

    walk_tree(treeWalker, logic).result.unwrap()
}

fn get_tree() -> i3ipc::reply::Node {
    return i3ipc::I3Connection::connect().unwrap().get_tree().unwrap();
}

// #[derive(Default)]
struct TreeWalker<T> {
    rootnode: i3ipc::reply::Node,
    nextnode: Option<i3ipc::reply::Node>,
    output: Option<i3ipc::reply::Node>,
    workspace: Option<i3ipc::reply::Node>,
    parent_containers: Vec<i3ipc::reply::Node>, // the outer-most (i.e. the first) parent is 0
    window: Option<i3ipc::reply::Node>,
    result: Option<T>,
}

impl<T> TreeWalker<T> {
    fn new() -> TreeWalker<T> {
        let tree = get_tree();

        TreeWalker::<T> {
            rootnode: tree.clone(),
            nextnode: Some(tree),
            output: None,
            workspace: None,
            parent_containers: Vec::new(),
            window: None,
            result: None,
        }
    }
}

fn walk_tree<T>(
    mut treeWalker: TreeWalker<T>,
    onNode: &mut FnMut(TreeWalker<T>) -> TreeWalker<T>,
) -> TreeWalker<T> {
    let mut node = treeWalker.nextnode.clone().unwrap();

    for node in node.nodes {
        treeWalker.nextnode = Some(node.clone());
        // println!("g");
        if treeWalker.result.is_some() {
            return treeWalker;
        }

        match node.nodetype {
            i3ipc::reply::NodeType::Output => {
                // println!("Output");
                treeWalker.output = Some(node);
                treeWalker = walk_tree(treeWalker, onNode);
                treeWalker.output = None;
            }
            i3ipc::reply::NodeType::Workspace => {
                // println!("Workspace");
                treeWalker.workspace = Some(node.clone());
                treeWalker = (onNode)(treeWalker);
                treeWalker = walk_tree(treeWalker, onNode);
                treeWalker.workspace = None;

            }
            i3ipc::reply::NodeType::Con => {
                match node.window {
                    Some(_) => {
                        // println!("Window");
                        treeWalker.window = Some(node.clone());
                        treeWalker = (onNode)(treeWalker);
                        treeWalker.window = None;

                    }
                    None => {
                        // println!("Con");
                        // if()
                        treeWalker.parent_containers.push(node.clone());
                        treeWalker = (onNode)(treeWalker);
                        treeWalker = walk_tree(treeWalker, onNode);
                        treeWalker.output = None;
                        treeWalker.parent_containers.pop();

                    }
                }
            }
            i3ipc::reply::NodeType::FloatingCon => {
                println!("F");
            }
            _ => {}
        }
    }

    treeWalker
}

