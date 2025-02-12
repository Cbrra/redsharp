mod builder;
mod compiler;
mod file;
mod minecraft;
mod nbt_reader;
mod nodes;
mod parser;
mod pathfinding;
mod vectors;

use builder::build_nodes;
use compiler::Compiler;
use file::read_file_code;
use minecraft::datapack::Datapack;
use parser::{ast::Statement, parser::parse};
use pathfinding::Pathfinding;
use std::time::Instant;

fn main() {
    println!("RedSharp - THIS IS A DEVELOPMENT VERSION.");
    let ast = parse_file();

    let mut compiler = Compiler::new();
    compiler.compile(ast);

    // Debug
    // println!("[[ NODES ]]");
    // println!("{:#?}", compiler.nodes);
    println!("Generated {} nodes", compiler.nodes.len());

    // println!("[[ EDGES ]]");
    // println!("{:#?}", compiler.edges);
    println!("Generated {} edges", compiler.edges.len());

    // Build the nodes and get their ports and obstacles positions
    let (nodes_instructions, ports_data, mut obstacles) = build_nodes(compiler.nodes.clone());

    // Find the edges paths
    println!("Starting the pathfinding...");
    let mut edges_instructions = Vec::new();
    let finder = Pathfinding {};
    finder.resolve(
        &mut edges_instructions,
        compiler.nodes.clone(),
        compiler.edges.clone(),
        &ports_data,
        &mut obstacles,
    );

    // Write the datapack
    println!("Writing the datapack...");
    let config_dir = dirs_next::config_dir().expect("Could not get the user config directory");
    let world_path = config_dir.join(".minecraft\\saves\\RedSharp");
    let datapack = Datapack::new(world_path.into());

    datapack.write_nodes(&nodes_instructions);
    datapack.write_edges(&edges_instructions);
    datapack.write_generate();
    datapack.write_datapack();
}

fn parse_file() -> Vec<Statement> {
    let program = read_file_code().unwrap();

    println!(">>> Running: '{}'", program);

    let ast_start_time = Instant::now();
    let ast = parse(&program).unwrap();
    println!(
        "AST parsed in {:.9}ms",
        ast_start_time.elapsed().as_nanos() as f64 / 1_000_000.0
    );

    // println!("{:#?}", ast);
    ast
}
