use crate::model::Node;

pub struct ViewItem<'a> {
    pub index: Option<usize>,
    pub node: &'a Node,
    pub depth: usize,
}

pub fn build_visible_list(root: &Node) -> Vec<ViewItem> {
    let mut results = Vec::new();
    let mut counter = 0; // 只统计目录
    scan_recursive(root, 0, &mut 0, &mut results);
    results
}

fn scan_recursive<'a>(
    node: &'a Node,
    depth: usize,
    counter: &mut usize,
    out: &mut Vec<ViewItem<'a>>,
) {
    let index = if node.is_dir() {
        let idx = *counter;
        *counter += 1;
        Some(idx)
    } else {
        None
    };

    out.push(ViewItem { index, node, depth });

    // 只有 目录 且 已展开 才继续扫描子节点
    if node.is_dir() && node.is_expanded() {
        if let Some(children) = node.children() {
            for child in children {
                scan_recursive(child, depth + 1, counter, out);
            }
        }
    }
}

pub fn toggle_by_index(root: &mut Node, target_index: usize) -> bool {
    let mut current_index = 0;
    find_and_toggle(root, &mut current_index, target_index)
}

fn find_and_toggle(node: &mut Node, counter: &mut usize, target: usize) -> bool {
    if node.is_dir() {
        if *counter == target {
            node.toggle();
            return true;
        }
        *counter += 1;
    } else {
        return false;
    }

    if node.is_expanded() {
        if let Some(children) = node.children_mut() {
            for child in children {
                if find_and_toggle(child, counter, target) {
                    return true;
                }
            }
        }
    }

    false
}
