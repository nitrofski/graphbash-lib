use std::fmt::{Debug, Display};
use std::fs::File;

use graphbash::*;

fn print_code<D, T>(destination: D, path: &[T]) where D: Debug, T: Display {
    println!("path to {:?} in {} steps:", destination, path.len());
    println!(
        "{:?}",
        path.iter().map(|d| d.to_string()).collect::<Vec<String>>()
    );
}

fn main() {
    let ram_dump_filename = "resources/RAM.bin";
    let mut file = File::open(ram_dump_filename).expect("could not open RAM dump file");

    let depth = 50;
    let graph = generate(&mut file, depth).expect(&format!(
        "could not generate graph of depth {} from RAM dump file",
        depth
    ));

    println!(
        "graph contains {} nodes and {} edges",
        graph.node_count(),
        graph.edge_count()
    );

    const ROOT_INDEX: i32 = 34;

    const PANIC_DASH_NODE: i32 = -1190;
    const INSTABOSS_NODE: i32 = -1399;
    const EARLY_START_NODE: i32 = -1510;
    const INSTAWIN_NODE: i32 = -1569;
    // const TIME_SKIP_NODE: i32 = -1608;
    const TIME_CUT_NODE: i32 = -1615;
    const MELT_PANIC_ARENA_SIZE_NODE: i32 = -2024;

    // const VISUAL_NO_RULES_BOX_NODE: i32 = -974;
    const VISUAL_SPINNING_SKYBOX_NODE: i32 = -989;
    const VISUAL_CORRUPT_PAUSE_NODE: i32 = -1482;

    if let Some((code_segments, _)) = get_shortest_code(&graph, ROOT_INDEX, &[
        PANIC_DASH_NODE,
        INSTABOSS_NODE,
        EARLY_START_NODE,
        INSTAWIN_NODE,
        // TIME_SKIP_NODE,
        TIME_CUT_NODE,
        MELT_PANIC_ARENA_SIZE_NODE,
        // VISUAL_NO_RULES_BOX_NODE,
        VISUAL_SPINNING_SKYBOX_NODE,
        VISUAL_CORRUPT_PAUSE_NODE,
    ]) {
        code_segments.iter().for_each(|(goal, seg)| print_code(goal, seg));
        let (goal_order, sub_codes): (Vec<_>, Vec<_>) = code_segments.into_iter().unzip();
        let code: Vec<_> = sub_codes.into_iter().flatten().collect();

        println!("== all in one ==");
        print_code(&goal_order[..], &code[..]);
    } else {
        println!("no path found");
    }

    if let Some((code_segments, cost)) = get_shortest_code(&graph, ROOT_INDEX, &[-72]) {
        code_segments.iter().for_each(|(goal, seg)| print_code(goal, seg));
        let (goal_order, sub_codes): (Vec<_>, Vec<_>) = code_segments.into_iter().unzip();
        let code: Vec<_> = sub_codes.into_iter().flatten().collect();

        println!("== all in one (total cost: {}) ==", cost);
        print_code(&goal_order[..], &code[..]);
    } else {
        println!("no path found");
    }
}
