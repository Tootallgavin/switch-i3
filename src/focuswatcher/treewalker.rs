use super::structures::*;
use std::collections::hash_map::Iter;
extern crate i3ipc;

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
    return i3ipc::I3Connection::connect().unwrap().get_tree().unwrap();
}

pub fn build_lists(wsl: &mut WorkSpaceList) {
    let rootnode = get_tree();
    walk_tree_bl(wsl, rootnode, 0);
}

pub fn resolve_name(id: i64) -> Option<String> {
    let rootnode = get_tree();
    return walk_tree_rn(rootnode, id);
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
    return walk_to_resolve_windows_workspace(rootnode, window_id, 0);
}

fn walk_to_resolve_windows_workspace(
    rootnode: i3ipc::reply::Node,
    window_id: i64,
    current_workspace_id: i64,
) -> i64 {
    let mut found: i64 = 0;
    for node in rootnode.nodes {
        if found != 0 {
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

#[cfg(test)]
mod test {
    use super::*;

    // for the following tests send a request and get the reponse.
    // response types are specific so often getting them at all indicates success.
    // can't do much better without mocking an i3 installation.
    extern crate x11_dl;

    use std::ffi::CString;
    use std::mem;
    use std::os::raw::*;
    use std::ptr;

    use self::x11_dl::xlib;

    struct WindowHelper {
      display:*mut x11_dl::xlib::_XDisplay,
      xlib: xlib::Xlib
    }

    impl WindowHelper {
      fn close_window(&self){
        unsafe {

                    // Shut down.
                    (self.xlib.XCloseDisplay)(self.display);
          }
      }
      fn build()-> WindowHelper{
        unsafe {
      //       // Load Xlib library.
            let xlib = xlib::Xlib::open().unwrap();
      //
      //       // Open display connection.
            let display = (xlib.XOpenDisplay)(ptr::null());

            return WindowHelper{ display: display, xlib:xlib}
          }
      }
      fn open_window_with_name(&mut self,name: &str){
        unsafe {
      //
            if self.display.is_null() {
                panic!("XOpenDisplay failed");
            }
      //
      //       // Create window.
            let screen = (self.xlib.XDefaultScreen)(self.display);
            let root = (self.xlib.XRootWindow)(self.display, screen);
      //
            let mut attributes: self::xlib::XSetWindowAttributes = mem::uninitialized();
            attributes.background_pixel = (self.xlib.XWhitePixel)(self.display, screen);
      //
            let window = (self.xlib.XCreateWindow)(
                self.display,
                root,
                0,
                0,
                400,
                300,
                0,
                0,
                xlib::InputOutput as c_uint,
                ptr::null_mut(),
                xlib::CWBackPixel,
                &mut attributes,
            );
      //
      //       // Set window title.
            let title_str = CString::new(name).unwrap();
            (self.xlib.XStoreName)(self.display, window, title_str.as_ptr() as *mut c_char);

            // Hook close requests.
            let wm_protocols_str = CString::new("WM_PROTOCOLS").unwrap();
            let wm_delete_window_str = CString::new("WM_DELETE_WINDOW").unwrap();
            //
            let wm_protocols = (self.xlib.XInternAtom)(self.display, wm_protocols_str.as_ptr(), xlib::False);
            let wm_delete_window =
                (self.xlib.XInternAtom)(self.display, wm_delete_window_str.as_ptr(), xlib::False);

            let mut protocols = [wm_delete_window];
            //
            (self.xlib.XSetWMProtocols)(
                self.display,
                window,
                protocols.as_mut_ptr(),
                protocols.len() as c_int,
            );
            //
            // // Show window.
            (self.xlib.XMapWindow)(self.display, window);

            // Main loop.
            let mut event: xlib::XEvent = mem::uninitialized();
            (self.xlib.XCheckIfEvent)(self.display, &mut event, None, ptr::null_mut());
            println!("gigoaisdv");
        }
      }
    }

    #[test]
    fn ws_on_init() {

        let mut wh = WindowHelper::build();
        const NAME :& 'static str = "test124";
        wh.open_window_with_name(NAME);
        let ws = resolve_focused();
        assert_eq!(NAME, resolve_name(ws.unwrap()).unwrap());
        wh.close_window();
    }
}
