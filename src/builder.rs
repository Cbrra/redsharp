use crate::{
    minecraft::structures::{MinecraftStructureNbt, MinecraftStructureSignNbt, PALETTE_AIR_NAME, PALETTE_SIGN_NAME},
    nodes::{Node, NodeType},
    vectors::Vector3,
};
use std::collections::{HashMap, HashSet};

/// Takes the program nodes and get the corresponding structures. Each structures will be placed in a grid patterns.
pub fn build_nodes(nodes: Vec<Node>) -> (Vec<String>, HashMap<String, Vec<Vector3>>, HashSet<Vector3>) {
    // Grid
    let grid_width = 50;
    let origin_y = 150; // For testing purposes. Will be customizable later
    let margin = 10;

    // Current node position
    let mut current_row_start_x = 0;
    let mut current_row_start_z = 0;
    let mut current_row_z = 0;

    let mut instructions = Vec::new();
    let mut ports_data = HashMap::new();
    let mut obstacles = HashSet::new();

    for node in nodes.clone() {
        let structure_nbt = NodeType::get_nbt(node.node.clone());
        let [x, _, z] = structure_nbt.size;

        if current_row_z < z {
            current_row_z = z;
        }

        // Grid cell coords
        let (cell_x, cell_y, cell_z) = (current_row_start_x, origin_y, current_row_start_z);

        register_structure(
            &node,
            &structure_nbt,
            Vector3(cell_x, cell_y, cell_z),
            &mut obstacles,
            &mut ports_data,
        );

        // Place the structure
        let name = NodeType::get_name(node.clone().node);
        instructions.push(format!(
            "place template redsharp:{name} {cell_x} {cell_y} {cell_z}"
        ));

        // Move to the next cell
        current_row_start_x += (x + margin) as i32;

        // Go to the next row if too far
        if current_row_start_x >= grid_width {
            current_row_start_x = 0;
            current_row_start_z += (current_row_z + margin) as i32;
            current_row_z = 0;
        }
    }

    // Removes the ports positions from the obstacles
    // TODO: is this still needed this register_structure ignore them?
    for p in ports_data.values() {
        for pos in p {
            obstacles.remove(pos);
        }
    }

    (instructions, ports_data, obstacles)
}

/// Use the structure NBT to get the ports positions, and register all other blocks as obstacles
fn register_structure(
    node: &Node,
    structure_nbt: &MinecraftStructureNbt,
    origin: Vector3,
    obstacles: &mut HashSet<Vector3>,
    ports_data: &mut HashMap<String, Vec<Vector3>>,
) {
    // Get the signs palette indexes
    let sign_states: Vec<usize> = structure_nbt
        .palette
        .iter()
        .enumerate()
        .filter_map(|(index, &ref value)| {
            if value.name == PALETTE_SIGN_NAME {
                Some(index)
            } else {
                None
            }
        })
        .collect();

    // TODO: Here I use .0 since there should be only one type of air block. But is this true?
    let air_state = structure_nbt
        .palette
        .iter()
        .enumerate()
        .find(|(_, &ref value)| value.name == PALETTE_AIR_NAME)
        .unwrap()
        .0;

    for block in &structure_nbt.blocks {
        if block.state as usize == air_state {
            continue;
        }

        let block_pos = Vector3(
            origin.0 + block.pos[0],
            origin.1 + block.pos[1],
            origin.2 + block.pos[2],
        );

        if sign_states.contains(&(block.state as usize)) {
            if let Some(block_nbt) = &block.nbt {
                let sign_nbt: MinecraftStructureSignNbt =
                    serde_json::from_value(block_nbt.clone()).unwrap();
                let sign_data = sign_nbt.front_text.messages[0].clone();

                // The sign should follow the format "(i|o)-[0-9]{0,}-[0-9]{0,}"
                let parts: Vec<&str> = sign_data.trim_matches('"').split('-').collect();

                if let [type_, n, id] = &parts[..] {
                    let type_ = *type_;
                    let port_n: usize = n.parse().unwrap();
                    let id: usize = id.parse().unwrap();

                    // At this point we have the position of an input/output. So we store the position for later use
                    let port = if type_ == "i" {
                        node.inputs[port_n].clone()
                    } else {
                        node.outputs[port_n].clone()
                    };

                    let val = ports_data
                        .entry(port.id)
                        .or_insert_with(|| vec![Vector3(0, 0, 0); port.size as usize]);
                    val[id] = block_pos;
                } else {
                    panic!("A sign string is malformed. Got {sign_data:?} at {block_pos:?} in {:?}", node.node);
                }
            } else {
                obstacles.insert(block_pos);
            }
        } else {
            obstacles.insert(block_pos);
        }
    }
}
