#[macro_use]
extern crate bitflags;

use std::collections::VecDeque;
use std::io;
use std::iter::Iterator;

use petgraph::graphmap;

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

pub fn get_shortest_path(
    graph: &graphmap::DiGraphMap<i32, Directions>,
    from_node: i32,
    to_node: i32,
) -> Option<Vec<Directions>> {
    let mut counter = 0;
    let (_, path) = petgraph::algo::astar(
        &graph,
        /*start:*/ from_node,
        /*is_goal:*/
        |i| {
            counter += 1;
            i == to_node
        },
        /*edge_cost:*/
        |e| {
            if e.1 == -1025
                || e.1 == -1317
                || e.1 == -1353
                || e.1 == -1381
                || e.1 == -1383
                || e.1 == -1400
                || e.1 == -1409
                || e.1 == -1424
            {
                return std::f32::INFINITY;
            }

            // Because diagonals are risky to input RTA in game, we'll give them a bigger cost.
            // Also, because other directioans are impossible RTA, they have infinite weight.
            // This should probably be configurable.
            if e.2.has_straight() {
                1.0
            } else if e.2.has_diagonal() {
                1.1
            } else {
                1.2
            }
        },
        /*estimate_cost:*/
        |i| {
            // Every straight direction can move at most 128 units toward the desired index. We
            // can use that as a heuristic, as that gives us a mean to quickly calculate the minimum
            // number of steps to get to the desired index.
            // Note: Because of how diagonal composition works, this is only valid if a diagonal
            //       input is set to cost at least twice as much as a straight input. Otherwise,
            //       this estimate should be halved (e.g. for TAS). An underestimate won't affect
            //       the result, only the performance.
            let estimate = (to_node - i).abs() / 128 + 1;
            estimate as f32 / 4.0
        },
    )?;

    println!("Visited {} nodes before done.", counter);

    Some(
        path.iter()
            .zip(path.iter().skip(1))
            .map(|p| *graph.edge_weight(*p.0, *p.1).unwrap())
            .collect(),
    )
}
