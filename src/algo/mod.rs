use petgraph::algo::Measure;
use petgraph::visit::IntoNodeIdentifiers;
use petgraph::visit::{EdgeRef, IntoEdges, Visitable};

use std::collections::hash_map::Entry::Occupied;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

mod utils;
use utils::{MinScored, PathTracker};

pub fn dijkstra<G, F, K>(
    graph: G,
    start: G::NodeId,
    goals: Vec<G::NodeId>,
    edge_cost: F,
) -> HashMap<G::NodeId, Option<(K, Vec<G::NodeId>)>>
where
    G: IntoEdges + Visitable,
    G::NodeId: Eq + Hash,
    F: Fn(G::EdgeRef) -> K,
    K: Measure + Copy,
{
    // consider implementing a fibonacci heap for even faster times
    let mut next_nodes = BinaryHeap::new();
    let mut scores = HashMap::new();
    let mut predecessors = HashMap::new();
    let mut remaining_goals: HashSet<_> = goals.iter().cloned().collect();

    next_nodes.push(MinScored(start, K::default()));
    scores.insert(start, K::default());

    while let Some(MinScored(current, score_when_queued)) = next_nodes.pop() {
        if score_when_queued > scores[&current] {
            continue; // already visited
        }

        remaining_goals.remove(&current);
        if remaining_goals.is_empty() {
            break;
        }

        for edge in graph.edges(current) {
            let target = edge.target();
            let score_from_current = scores[&current] + edge_cost(edge);
            match scores.entry(target) {
                Occupied(score_entry) if score_from_current >= *score_entry.get() => {}
                score_entry => {
                    *score_entry.or_default() = score_from_current;
                    predecessors.insert(target, current);
                    next_nodes.push(MinScored(target, score_from_current))
                }
            }
        }
    }

    let make_path = |goal: G::NodeId| {
        let mut path: Vec<_> = [goal].into();
        while let Some(&pred) = predecessors.get(path.last().unwrap()) {
            path.push(pred);
        }
        path.reverse();
        return path;
    };

    goals
        .into_iter()
        .map(|g| {
            (
                g,
                scores
                    .get(&g)
                    .and_then(|score| Some((*score, make_path(g)))),
            )
        })
        .collect()
}

pub fn shortest_hamiltonian_path<G, F, K>(
    graph: G,
    start: G::NodeId,
    edge_cost: F,
) -> Option<(Vec<G::NodeId>, K)>
where
    G: IntoEdges + IntoNodeIdentifiers + Visitable,
    G::NodeId: Eq + Hash + Debug,
    F: Fn(G::EdgeRef) -> K,
    K: Measure + Copy,
{
    let node_count = graph.node_identifiers().count();
    let mut next_nodes = BinaryHeap::new();
    let mut path_tracker = PathTracker::new();

    next_nodes.push(MinScored(path_tracker.push_root(start), K::default()));

    while let Some(MinScored(path_node, score_when_queued)) = next_nodes.pop() {
        if path_node.depth + 1 == node_count {
            return Some((path_tracker.recreate_path(&path_node), score_when_queued));
        }

        for edge in graph.edges(path_node.id) {
            let target = edge.target();
            let score_from_current = score_when_queued + edge_cost(edge);
            if !path_tracker.path_includes(&path_node, target) {
                next_nodes.push(MinScored(
                    path_tracker.push(&path_node, target),
                    score_from_current,
                ));
            }
        }
    }

    None
}
