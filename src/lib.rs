#[macro_use] extern crate bitflags;

use std::collections::HashMap;
use std::collections::VecDeque;
use std::io;
use std::iter::Iterator;

use petgraph::graphmap;
use petgraph::visit::EdgeRef;

mod algo;

mod utils;
use utils::ItWithFallback;

bitflags! {
    pub struct Directions : u16 {
        const UP = 0x0001;
        const LEFT = 0x0002;
        const RIGHT = 0x0004;
        const DOWN = 0x0008;

        const UP_LEFT = 0x0010;
        const UP_RIGHT = 0x0020;
        const DOWN_LEFT = 0x0040;
        const DOWN_RIGHT = 0x0080;

        const UP_DOWN = 0x0100;
        const LEFT_RIGHT = 0x0200;
        const UP_LEFT_RIGHT = 0x0400;
        const UP_LEFT_DOWN = 0x0800;
        const UP_RIGHT_DOWN = 0x1000;
        const LEFT_RIGHT_DOWN = 0x2000;

        const ALL_AT_ONCE = 0x4000;

        const ANY_STRAIGHT = Directions::UP.bits | Directions::LEFT.bits
                           | Directions::RIGHT.bits | Directions::DOWN.bits;
        const ANY_DIAGONAL = Directions::UP_LEFT.bits | Directions::UP_RIGHT.bits
                           | Directions::DOWN_LEFT.bits | Directions::DOWN_RIGHT.bits;
        const ANY_REAL_TIME_IMPOSSIBLE = Directions::UP_DOWN.bits
                                       | Directions::LEFT_RIGHT.bits
                                       | Directions::UP_LEFT_RIGHT.bits
                                       | Directions::UP_LEFT_DOWN.bits
                                       | Directions::UP_RIGHT_DOWN.bits
                                       | Directions::LEFT_RIGHT_DOWN.bits
                                       | Directions::ALL_AT_ONCE.bits;
    }
}

impl Directions {
    fn has_straight(&self) -> bool {
        self.intersects(Directions::ANY_STRAIGHT)
    }

    fn has_diagonal(&self) -> bool {
        self.intersects(Directions::ANY_DIAGONAL)
    }

    // fn has_real_time_impossible(&self) -> bool {
    //     self.intersects(Directions::ANY_REAL_TIME_IMPOSSIBLE)
    // }
}

impl std::fmt::Display for Directions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let pairs = [
            (Directions::UP, "U"),
            (Directions::LEFT, "L"),
            (Directions::RIGHT, "R"),
            (Directions::DOWN, "D"),
            (Directions::UP_LEFT, "UL"),
            (Directions::UP_RIGHT, "UR"),
            (Directions::DOWN_LEFT, "DL"),
            (Directions::DOWN_RIGHT, "DR"),
            (Directions::UP_DOWN, "UD"),
            (Directions::LEFT_RIGHT, "LR"),
            (Directions::UP_LEFT_RIGHT, "ULR"),
            (Directions::UP_LEFT_DOWN, "ULD"),
            (Directions::UP_RIGHT_DOWN, "URD"),
            (Directions::LEFT_RIGHT_DOWN, "LRD"),
            (Directions::ALL_AT_ONCE, "ALL"),
        ];

        write!(
            f,
            "{}",
            pairs
                .iter()
                .take(4)
                .filter(|t| self.contains(t.0))
                .or(pairs.iter().skip(4).take(4).filter(|t| self.contains(t.0)))
                .or(pairs.iter().skip(8).take(2).filter(|t| self.contains(t.0)))
                .or(pairs.iter().skip(10).take(4).filter(|t| self.contains(t.0)))
                .or(pairs.iter().skip(14).filter(|t| self.contains(t.0)))
                .map(|t| t.1)
                .collect::<Vec<&str>>()
                .join("|")
        )
    }
}

const RAM_MOVE_BEHAVIOUR_ROOT_ADDRESS: i64 = 0x0BD5DA;
const RAM_MOVE_BEHAVIOUR_INDEX_OFFSET: i64 = 0x10; // 16 bytes

pub fn generate(
    ram_dump: &mut (impl io::Read + io::Seek),
    max_depth: i32,
) -> std::result::Result<graphmap::DiGraphMap<i32, Directions>, std::io::Error> {
    let mut graph = graphmap::DiGraphMap::<i32, Directions>::new();

    struct NodeDepth {
        panel_idx: i32,
        depth: i32,
    }

    let mut queue = VecDeque::<NodeDepth>::new();

    queue.push_back(NodeDepth {
        panel_idx: graph.add_node(0),
        depth: max_depth,
    });

    // The directional behavior table in RAM is shaped as sets of 4 bytes, separated from each
    // adjacent sets by 12 bytes. The 4 bytes represent, respectively, the left, right, up then down
    // behaviors.
    //
    // When composing multiple directions, the game simply adds the current index's horizontal and
    // vertical behaviors for the current panel index. This is what causes the menu OoB.

    while let Some(cur_node) = queue.pop_front() {
        ram_dump.seek(io::SeekFrom::Start(
            (RAM_MOVE_BEHAVIOUR_ROOT_ADDRESS
                + cur_node.panel_idx as i64 * RAM_MOVE_BEHAVIOUR_INDEX_OFFSET) as u64,
        ))?;

        let mut bytes = [0u8; 4];
        ram_dump.read_exact(&mut bytes)?;

        let l = bytes[0] as i8 as i32;
        let r = bytes[1] as i8 as i32;
        let u = bytes[2] as i8 as i32;
        let d = bytes[3] as i8 as i32;

        let mut add_node_edge = |directions: Directions, to_panel_idx: i32| {
            if !graph.contains_node(to_panel_idx) {
                graph.add_node(to_panel_idx);
                if cur_node.depth > 0 {
                    queue.push_back(NodeDepth {
                        panel_idx: to_panel_idx,
                        depth: cur_node.depth - 1,
                    });
                }
            }

            if let Some(edge_direction) = graph.edge_weight_mut(cur_node.panel_idx, to_panel_idx) {
                *edge_direction |= directions;
            } else {
                graph.add_edge(cur_node.panel_idx, to_panel_idx, directions.into());
            }
        };

        add_node_edge(Directions::UP, cur_node.panel_idx + u);
        add_node_edge(Directions::LEFT, cur_node.panel_idx + l);
        add_node_edge(Directions::RIGHT, cur_node.panel_idx + r);
        add_node_edge(Directions::DOWN, cur_node.panel_idx + d);

        add_node_edge(Directions::UP_LEFT, cur_node.panel_idx + u + l);
        add_node_edge(Directions::UP_RIGHT, cur_node.panel_idx + u + r);
        add_node_edge(Directions::DOWN_LEFT, cur_node.panel_idx + d + l);
        add_node_edge(Directions::DOWN_RIGHT, cur_node.panel_idx + d + r);

        add_node_edge(Directions::UP_DOWN, cur_node.panel_idx + u + d);
        add_node_edge(Directions::LEFT_RIGHT, cur_node.panel_idx + l + r);
        add_node_edge(Directions::UP_LEFT_RIGHT, cur_node.panel_idx + u + l + r);
        add_node_edge(Directions::UP_LEFT_DOWN, cur_node.panel_idx + u + l + d);
        add_node_edge(Directions::UP_RIGHT_DOWN, cur_node.panel_idx + u + r + d);
        add_node_edge(Directions::LEFT_RIGHT_DOWN, cur_node.panel_idx + l + r + d);
        add_node_edge(Directions::ALL_AT_ONCE, cur_node.panel_idx + u + l + r + d);

        // Debug
        #[cfg(debug_assertions)]
        {
            let node_edges = graph
                .edges(cur_node.panel_idx)
                .collect::<Vec<(i32, i32, &Directions)>>();
            println!(
                "({:2}) Added node {:5} with edges: {:?}",
                max_depth - cur_node.depth,
                cur_node.panel_idx,
                node_edges
                    .iter()
                    .map(|e| { format!("{}:{}", e.2, e.1) })
                    .collect::<Vec<String>>()
            );
        }
    }

    Ok(graph)
}

fn edge_cost((_, to, w): (i32, i32, &Directions)) -> f32 {
    if to == -896
        || to == -945
        || to == -979
        || to == -1014
        || to == -1018
        || to == -1025
        || to == -1039
        || to == -1064
        || to == -1207
        || to == -1313
        || to == -1314
        || to == -1317
        || to == -1347
        || to == -1353
        || to == -1373
        || to == -1375
        || to == -1381
        || to == -1383
        || to == -1388
        || to == -1397
        || to == -1400
        || to == -1409
        || to == -1420
        || to == -1424
        || to == -1664
        || to == -1954
        || to == -2205
        || to == -2271
        || to == -2275
        || to == -2277
        || to == -2280
        || to == -2282
    {
        return f32::INFINITY;
    }

    // Because diagonals are risky to input RTA in game, we'll give them a bigger cost.
    // Also, because other directioans are impossible RTA, they have infinite weight.
    // This should probably be configurable.
    if w.has_straight() {
        1.0
    } else if w.has_diagonal() {
        5.5
    } else {
        15.5
    }
}

pub fn get_shortest_code(
    graph: &graphmap::DiGraphMap<i32, Directions>,
    from_node: i32,
    goal_nodes: &[i32],
) -> Option<(Vec<(i32, Vec<Directions>)>, f32)> {
    let shortest_path_descriptions: HashMap<_, _> =
        std::iter::once((from_node, goal_nodes.iter().cloned().collect::<Vec<_>>()))
            .chain(goal_nodes.iter().map(|&from| {
                let to = goal_nodes
                    .iter()
                    .filter(|&to| *to != from)
                    .cloned()
                    .collect::<Vec<_>>();
                (from, to)
            }))
            .flat_map(|(from, goals)| {
                algo::dijkstra(&graph, from, goals, edge_cost)
                    .into_iter()
                    .filter_map(move |(to, opt)| {
                        opt.and_then(|(cost, path)| Some(((from, to), (cost, path))))
                    })
            })
            .collect();

    // shortest_path_descriptions
    //     .iter()
    //     .for_each(|desc| println!("{:?}", desc));

    // we build a goal graph, where nodes are the goals and
    // edge weights are total path cost in source graph
    let goal_graph = graphmap::DiGraphMap::<i32, f32>::from_edges(
        shortest_path_descriptions
            .iter()
            .map(|(&(from, to), (w, _))| (from, to, w)),
    );

    return algo::shortest_hamiltonian_path(&goal_graph, from_node, |e| *e.weight()).and_then(
        |(shortest_goal_path, total_cost)| {
            if total_cost.is_infinite() {
                None
            } else {
                Some((
                    shortest_goal_path
                        .iter()
                        .zip(shortest_goal_path.iter().skip(1))
                        .map(|(&from_goal, &to_goal)| {
                            let (_, sub_path) = shortest_path_descriptions
                                .get(&(from_goal, to_goal))
                                .unwrap();
                            (
                                to_goal,
                                sub_path
                                    .iter()
                                    .zip(sub_path.iter().skip(1))
                                    .map(|(&from, &to)| *graph.edge_weight(from, to).unwrap())
                                    .collect(),
                            )
                        })
                        .collect(),
                    total_cost,
                ))
            }
        },
    );
}
