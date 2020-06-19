use std::ptr;
use std::pin::Pin;
use std::cmp::Ordering;
use std::fmt::Debug;

//
// MinScored
//

#[derive(Copy, Clone, Debug)]
pub struct MinScored<T, K>(pub T, pub K);

impl<T, K: PartialOrd> PartialEq for MinScored<T, K> {
    #[inline]
    fn eq(&self, other: &MinScored<T, K>) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<T, K: PartialOrd> Eq for MinScored<T, K> {}

impl<T, K: PartialOrd> PartialOrd for MinScored<T, K> {
    #[inline]
    fn partial_cmp(&self, other: &MinScored<T, K>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T, K: PartialOrd> Ord for MinScored<T, K> {
    #[inline]
    fn cmp(&self, other: &MinScored<T, K>) -> Ordering {
        let left = &self.1;
        let right = &other.1;
        match left.partial_cmp(&right) {
            Some(ordering) => ordering.reverse(),
            None if left.ne(right) && right.ne(left) => Ordering::Equal,
            None if left.ne(left) => Ordering::Less,
            None => Ordering::Greater,
        }
    }
}

//
// PathTracker
//

#[derive(Debug)]
struct PathTrackerNode<T> {
    id: T,
    predecessor: *const PathTrackerNode<T>,
    depth: usize,
}

pub struct PathTrackerNodeRef<T> {
    pub id: T,
    pub depth: usize,
    tracker_idx: usize,
}

pub struct PathTracker<T> {
    nodes: Vec<Pin<Box<PathTrackerNode<T>>>>,
}

impl<T> PathTracker<T>
where
    T: Copy + PartialEq + Debug,
{
    pub fn new() -> PathTracker<T> {
        PathTracker { nodes: Vec::new() }
    }

    fn push_node(&mut self, node: PathTrackerNode<T>) -> PathTrackerNodeRef<T> {
        let node_ref = PathTrackerNodeRef {
            id: node.id,
            depth: node.depth,
            tracker_idx: self.nodes.len(),
        };

        self.nodes.push(Box::pin(node));
        return node_ref;
    }

    pub fn push_root(&mut self, root: T) -> PathTrackerNodeRef<T> {
        self.push_node(PathTrackerNode {
            id: root,
            predecessor: ptr::null(),
            depth: 0,
        })
    }

    pub fn push(&mut self, predecessor: &PathTrackerNodeRef<T>, next: T) -> PathTrackerNodeRef<T> {
        self.push_node(PathTrackerNode {
            id: next,
            predecessor: &*self.nodes[predecessor.tracker_idx],
            depth: predecessor.depth + 1,
        })
    }

    pub fn recreate_path(&self, to: &PathTrackerNodeRef<T>) -> Vec<T> {
        let mut path = Vec::new();
        path.push(&*self.nodes[to.tracker_idx]);
        while let Some(predecessor) = unsafe { path[path.len() - 1].predecessor.as_ref() } {
            path.push(predecessor);
        }
        path.iter().rev().map(|&n| n.id).collect()
    }

    pub fn path_includes(&self, to: &PathTrackerNodeRef<T>, id: T) -> bool {
        let mut cursor = &*self.nodes[to.tracker_idx];
        while let Some(predecessor) = unsafe { cursor.predecessor.as_ref() } {
            if cursor.id == id {
                return true;
            }
            cursor = predecessor;
        }
        return cursor.id == id;
    }
}
