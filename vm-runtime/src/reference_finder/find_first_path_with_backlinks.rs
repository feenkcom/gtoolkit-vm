use crate::reference_finder::GraphNode;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
use vm_object_model::AnyObjectRef;

/// Frame with backlink to previous node
#[derive(Clone, Debug)]
struct SearchFrame<T: GraphNode> {
    node: T,
    parent: Option<Rc<SearchFrame<T>>>,
}

/// BFS with backlinks, using plain objects (no Arc)
pub fn find_paths_with_backlinks(start: AnyObjectRef, target: AnyObjectRef, search_all: bool) -> Vec<Vec<AnyObjectRef>>
{
    let mut visited: HashSet<AnyObjectRef> = HashSet::new();

    let root = SearchFrame {
        node: start,
        parent: None,
    };

    let mut buffer = vec![];
    let mut objects_left = vec![];
    objects_left.push(Rc::new(root));
    visited.insert(start);

    let mut paths = vec![];

    loop {
        buffer.clear();
        let objects = std::mem::replace(&mut objects_left, buffer);
        for frame in &objects {
            for neighbor in frame.node.neighbors(target) {
                if neighbor == target {
                    let new_frame = SearchFrame {
                        node: neighbor,
                        parent: Some(frame.clone()),
                    };
                    paths.push(reconstruct_path(&new_frame));
                    if !search_all {
                        return paths;
                    }
                }

                if visited.insert(neighbor) {
                    let new_frame = SearchFrame {
                        node: neighbor,
                        parent: Some(frame.clone()),
                    };
                    objects_left.push(Rc::new(new_frame));
                }
            }
        }

        buffer = objects;

        if objects_left.is_empty() {
            break;
        }
    }

    paths
}

/// Reconstruct path from backlinks
fn reconstruct_path<T: GraphNode>(mut frame: &SearchFrame<T>) -> Vec<T> {
    let mut path = Vec::new();
    loop {
        path.push(frame.node.clone());
        match &frame.parent {
            Some(parent) => frame = parent,
            None => break,
        }
    }
    path.reverse();
    path
}
