use std::ptr::null_mut;

use procon_lg::lg_recur;

#[derive(Debug)]
struct Node {
    key: i32,
    left: *mut Self,
    right: *mut Self,
}

#[lg_recur]
fn traversal(
    #[no_name]
    #[fmt(root.key)]
    root: &Node,
    #[fmt(format!("{:b}", code))] code: u32,
) {
    unsafe {
        if let Some(left) = root.left.as_ref() {
            traversal(left, code << 1);
        }
        eprintln!("Hi!");
        if let Some(right) = root.right.as_ref() {
            traversal(right, code << 1 | 1);
        }
    }
}

fn main() {
    let mut nodes = (0..6)
        .map(|key| Node {
            key,
            left: null_mut(),
            right: null_mut(),
        })
        .collect::<Vec<_>>();
    nodes[2].left = &mut nodes[0];
    nodes[0].right = &mut nodes[1];
    nodes[2].right = &mut nodes[3];
    nodes[3].right = &mut nodes[5];
    nodes[5].left = &mut nodes[4];
    traversal(&nodes[2], 1);
}
