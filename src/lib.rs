#[macro_use]
extern crate bitflags;

use std::collections::VecDeque;
use std::io;
use std::iter::Iterator;

use petgraph::graphmap;

mod utils;
use utils::ItWithFallback;

bitflags! {
    pub struct Directions : u8 {
        const UP = 0x01;
        const RIGHT = 0x02;
        const DOWN = 0x04;
        const LEFT = 0x08;
        const UP_RIGHT = 0x10;
        const DOWN_RIGHT = 0x20;
        const DOWN_LEFT = 0x40;
        const UP_LEFT = 0x80;

        const NONE = 0x00;
        const ALL_STRAIGHTS = Self::UP.bits | Self::RIGHT.bits | Self::DOWN.bits | Self::LEFT.bits;
        const ALL_DIAGONALS = !Self::ALL_STRAIGHTS.bits;
        const ALL = 0xFF;
    }
}

impl std::fmt::Display for Directions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let pairs = [
            (Directions::UP, "U"),
            (Directions::RIGHT, "R"),
            (Directions::DOWN, "D"),
            (Directions::LEFT, "L"),
            (Directions::UP_RIGHT, "UR"),
            (Directions::DOWN_RIGHT, "DR"),
            (Directions::DOWN_LEFT, "DL"),
            (Directions::UP_LEFT, "UL"),
        ];

        write!(
            f,
            "{}",
            pairs
                .iter()
                .take(4)
                .filter(|t| self.contains(t.0))
                .or(pairs.iter().skip(4).filter(|t| self.contains(t.0)))
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
        panel_index: i32,
        depth: i32,
    }

    let mut queue = VecDeque::<NodeDepth>::new();

    queue.push_back(NodeDepth {
        panel_index: graph.add_node(0),
        depth: max_depth,
    });

    // The directional behavior table in RAM is shaped as sets of 4 bytes, separated from each
    // adjacent sets by 12 bytes. The 4 bytes represent, respectively, the left, right, up then down
    // behaviors.
    //
    // When composing multiple directions, the game simply adds the current index's horizontal and
    // vertical behaviors for the current panel index. This is what causes the menu OoB.

    while let Some(node_depth) = queue.pop_front() {
        ram_dump.seek(io::SeekFrom::Start(
            (RAM_MOVE_BEHAVIOUR_ROOT_ADDRESS
                + node_depth.panel_index as i64 * RAM_MOVE_BEHAVIOUR_INDEX_OFFSET)
                as u64,
        ))?;

        let mut bytes = [0u8; 4];
        ram_dump.read_exact(&mut bytes)?;

        let l = bytes[0] as i8 as i32;
        let r = bytes[1] as i8 as i32;
        let u = bytes[2] as i8 as i32;
        let d = bytes[3] as i8 as i32;

        let mut add_node_and_edge = |directions: Directions, to_panel_index: i32| {
            if !graph.contains_node(to_panel_index) {
                graph.add_node(to_panel_index);
                if node_depth.depth > 0 {
                    queue.push_back(NodeDepth {
                        panel_index: to_panel_index,
                        depth: node_depth.depth - 1,
                    });
                }
            }

            if let Some(edge_direction) =
                graph.edge_weight_mut(node_depth.panel_index, to_panel_index)
            {
                *edge_direction |= directions;
            } else {
                graph.add_edge(node_depth.panel_index, to_panel_index, directions.into());
            }
        };

        add_node_and_edge(Directions::UP, node_depth.panel_index + u);
        add_node_and_edge(Directions::RIGHT, node_depth.panel_index + r);
        add_node_and_edge(Directions::DOWN, node_depth.panel_index + d);
        add_node_and_edge(Directions::LEFT, node_depth.panel_index + l);
        add_node_and_edge(Directions::UP_RIGHT, node_depth.panel_index + u + r);
        add_node_and_edge(Directions::DOWN_RIGHT, node_depth.panel_index + d + r);
        add_node_and_edge(Directions::DOWN_LEFT, node_depth.panel_index + d + l);
        add_node_and_edge(Directions::UP_LEFT, node_depth.panel_index + u + l);

        // Debug
        #[cfg(debug_assertions)]
        {
            let node_edges = graph
                .edges(node_depth.panel_index)
                .collect::<Vec<(i32, i32, &Directions)>>();
            println!(
                "({:2}) Added node {:5} with edges: {:?}",
                max_depth - node_depth.depth,
                node_depth.panel_index,
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
            if e.2.intersects(Directions::ALL_STRAIGHTS) {
                1
            } else {
                3
            }
        },
        /*estimate_cost:*/
        |i| {
            // Every straight direction can move at most 128 indices toward the desired index. We
            // can use that as a heuristic, as that gives us a mean to quickly calculate the minimum
            // number of steps to get to the desired index.
            // Note: Because of how diagonal composition works, this is only valid if a diagonal
            //       input is set to cost at least twice as much as a straight input. Otherwise,
            //       this estimate should be halved (e.g. for TAS). An underestimate won't affect
            //       the result, only the performance.
            let estimate = (to_node - i).abs() / 128 + 1;
            estimate
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
