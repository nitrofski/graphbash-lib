use std::fs::File;

use graphbash::*;

fn print_path(destination: i32, path: &Vec<Directions>) {
    println!("Path to {} found with {} steps.", destination, path.len());
    println!(
        "{:?}",
        path.iter().map(|d| d.to_string()).collect::<Vec<String>>()
    );
}

fn main() {
    let ram_dump_filename = "resources/RAM.bin";
    let mut file = File::open(ram_dump_filename).expect("Could not open RAM dump file.");

    let depth = 50;
    let graph = generate(&mut file, depth).expect(&format!(
        "Could not generate graph of depth {} from RAM dump file.",
        depth
    ));

    println!(
        "Graph contains {} nodes and {} edges.",
        graph.node_count(),
        graph.edge_count()
    );

    const ROOT_INDEX: i32 = 34;
    const PANIC_DASH_NODE: i32 = -1190;
    const TIME_CODE_NODE: i32 = -1608;
    const INSTABOSS_NODE: i32 = -1399;
    const INSTAWIN_NODE: i32 = -1569;

    if let Some(path) = get_shortest_path(&graph, ROOT_INDEX, PANIC_DASH_NODE) {
        print_path(PANIC_DASH_NODE, &path);
    } else {
        println!("No path found.");
    }

    if let Some(path) = get_shortest_path(&graph, PANIC_DASH_NODE, TIME_CODE_NODE) {
        print_path(TIME_CODE_NODE, &path);
    } else {
        println!("No path found.");
    }

    if let Some(path) = get_shortest_path(&graph, TIME_CODE_NODE, INSTABOSS_NODE) {
        print_path(INSTABOSS_NODE, &path);
    } else {
        println!("No path found.");
    }

    if let Some(path) = get_shortest_path(&graph, INSTABOSS_NODE, INSTAWIN_NODE) {
        print_path(INSTAWIN_NODE, &path);
    } else {
        println!("No path found.");
    }

    const SOME_RANDOM_INDEX: i32 = -3683;
    if let Some(path) = get_shortest_path(&graph, ROOT_INDEX, SOME_RANDOM_INDEX) {
        print_path(SOME_RANDOM_INDEX, &path);
    } else {
        println!("No path found.");
    }
}
