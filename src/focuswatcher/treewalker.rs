//! Abstraction and functions for walking the i3-node tree.

extern crate i3ipc;
use super::structures::*;

struct TreeWalker<T> {
    // rootnode: i3ipc::reply::Node,
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
            // rootnode: tree.clone(),
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
    mut tree_walker: TreeWalker<T>,
    on_node: &mut FnMut(TreeWalker<T>) -> TreeWalker<T>,
) -> TreeWalker<T> {
    let node = tree_walker.nextnode.unwrap();
    tree_walker.nextnode = None;

    for node in node.nodes {
        tree_walker.nextnode = Some(node.clone());
        if tree_walker.result.is_some() {
            return tree_walker;
        }

        match node.nodetype {
            i3ipc::reply::NodeType::Output => {
                tree_walker.output = Some(node);
                tree_walker = walk_tree(tree_walker, on_node);
                tree_walker.output = None;
            }
            i3ipc::reply::NodeType::Workspace => {
                tree_walker.workspace = Some(node);
                tree_walker = (on_node)(tree_walker);
                tree_walker = walk_tree(tree_walker, on_node);
                tree_walker.workspace = None;
            }
            i3ipc::reply::NodeType::Con => {
                match node.window {
                    Some(_) => {
                        tree_walker.window = Some(node);
                        tree_walker = (on_node)(tree_walker);
                        tree_walker.window = None;
                    }
                    None => {
                        tree_walker = (on_node)(tree_walker);
                        tree_walker.parent_containers.push(node);
                        tree_walker = walk_tree(tree_walker, on_node);
                        // tree_walker.output = None; //should this be here?
                        tree_walker.parent_containers.pop();
                    }
                }
            }
            i3ipc::reply::NodeType::FloatingCon => {
                println!("F");
            }
            _ => {}
        }
    }

    tree_walker
}

pub fn build_lists(wsl: &mut WorkSpaceList) {
    let tree_walker = TreeWalker::new();

    let logic = &mut |tree_walker: TreeWalker<i64>| -> TreeWalker<i64> {
        if tree_walker.workspace.is_some() && tree_walker.window.is_none() {
            let workspace = tree_walker.workspace.clone().unwrap();
 
            wsl.workspace_on_init(workspace.id);
        }

        if tree_walker.window.is_some() {
            let window = tree_walker.window.clone().unwrap();
            let workspace = tree_walker.workspace.clone().unwrap();
            wsl.window_on_init(window.id, Some(workspace.id));
        }

        return tree_walker;
    };

    walk_tree(tree_walker, logic);
}

pub fn resolve_name(id: i64) -> Option<String> {
    let tree_walker = TreeWalker::new();

    let logic = &mut |mut tree_walker: TreeWalker<String>| -> TreeWalker<String> {
        if tree_walker.workspace.is_some() && tree_walker.window.is_none() {
            let workspace = tree_walker.workspace.clone().unwrap();
            if workspace.id == id {
                tree_walker.result = workspace.name;
            }
        }

        if tree_walker.window.is_some() {
            let window = tree_walker.window.clone().unwrap();
            if window.id == id {
                tree_walker.result = window.name;
            }
        }

        return tree_walker;
    };

    walk_tree(tree_walker, logic).result
}

pub fn resolve_focused() -> Option<i64> {
    let tree_walker = TreeWalker::new();

    let logic = &mut |mut tree_walker: TreeWalker<i64>| -> TreeWalker<i64> {
        if tree_walker.window.is_some() {
            let window = tree_walker.window.clone().unwrap();
            if window.focused {
                tree_walker.result = Some(window.id);
            }
        }

        return tree_walker;
    };

    walk_tree(tree_walker, logic).result
}

pub fn find_window_workspace_from_i3(window_id: i64) -> i64 {
    let tree_walker = TreeWalker::new();

    let logic = &mut |mut tree_walker: TreeWalker<i64>| -> TreeWalker<i64> {
        if tree_walker.window.is_some() {
            let window = tree_walker.window.clone().unwrap();
            let workspace = tree_walker.workspace.clone().unwrap();
            if window.id == window_id {
                tree_walker.result = Some(workspace.id);
            }
        }

        return tree_walker;
    };

    walk_tree(tree_walker, logic).result.unwrap()
}

fn get_tree() -> i3ipc::reply::Node {
    return i3ipc::I3Connection::connect().unwrap().get_tree().unwrap();
}
