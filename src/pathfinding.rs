use crate::{nodes::Node, vectors::Vector3};
use pathfinding::directed::astar::astar;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Eq)]
/// Since only the "position" value must be checked by the A* algorithm, some traits are implemented manually to exclude the others values
pub struct PathNode {
    pub position: Vector3,
    pub steps_from_start: u32,
    pub previous: Option<Vector3>,
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl std::hash::Hash for PathNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.position.hash(state);
    }
}

pub fn is_obstacle_nearby(obstacles: &HashSet<Vector3>, pos: Vector3) -> bool {
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                if obstacles.contains(&Vector3(pos.0 + x, pos.1 + y, pos.2 + z)) {
                    return true;
                }
            }
        }
    }

    false
}

pub struct Pathfinding {}

impl Pathfinding {
    fn find_path(
        &self,
        start: Vector3,
        goal: Vector3,
        obstacles: &HashSet<Vector3>,
        entries: &Vec<Vector3>,
    ) -> Option<Vec<Vector3>> {
        astar(
            &PathNode {
                position: start,
                steps_from_start: 0,
                previous: None,
            },
            |node| {
                node.position
                    .neighbors(&start, &goal, obstacles, entries, &node)
                    .into_iter()
                    .map(|(pos, cost)| {
                        (
                            PathNode {
                                position: pos,
                                steps_from_start: node.steps_from_start + 1,
                                previous: Some(node.position.clone()),
                            },
                            cost,
                        )
                    })
                    .collect::<Vec<_>>()
            },
            |node| node.position.distance(&goal),
            |node| node.position == goal,
        )
        .map(|(path, _cost)| path.into_iter().map(|node| node.position).collect())
    }

    /// Find and build all the paths
    pub fn resolve(
        &self,
        instructions: &mut Vec<String>,
        nodes: Vec<Node>,
        edges: Vec<(String, String, String, String)>,
        ports: &HashMap<String, Vec<Vector3>>,
        obstacles: &mut HashSet<Vector3>,
    ) {
        let mut entries: Vec<Vector3> = vec![];
        for v in ports.values() {
            entries.extend(v);
        }

        let mut edge_i = 0;

        // Loop over the edges and find the paths.
        // An edge connects two nodes. An edge can be multiple path e.g. An int (8 bits) corresponds 8 paths
        for edge in edges.clone() {
            let (node_a_id, port_a_id, _node_b_id, port_b_id) = edge;

            let node_a = nodes.iter().find(|n: &&Node| n.id == node_a_id).unwrap();
            let port_a = node_a.inputs.iter().find(|p| p.id == port_a_id);
            let port_a = port_a.or(node_a.outputs.iter().find(|p| p.id == port_a_id));
            let port_a = port_a.unwrap();
            let size = port_a.size as usize;

            let pos_a = ports.get(&port_a_id).unwrap();
            let pos_b = ports.get(&port_b_id).unwrap();

            for i in 0..size {
                if let Some(path) = self.find_path(pos_a[i], pos_b[i], obstacles, &entries) {
                    let mut j = 0;
                    let mut last_pos = Vector3(0, 0, 0);

                    for pos in path.clone() {
                        let dir = Vector3(
                            pos.0 - last_pos.0,
                            pos.1 - last_pos.1,
                            pos.2 - last_pos.2,
                        );
                        last_pos = pos;
                        let redstone_step = j % 14;

                        // Build the path and the wires
                        instructions.push(format!(
                            "setblock {} {} {} minecraft:green_wool",
                            pos.0, pos.1, pos.2
                        ));
                        instructions.push(format!(
                            "setblock {} {} {} minecraft:{}",
                            pos.0,
                            pos.1 + 1,
                            pos.2,
                            if redstone_step == 1 {
                                let orientation = match dir {
                                    Vector3(1, _, 0) => "west",
                                    Vector3(0, _, 1) => "north",
                                    Vector3(-1, _, 0) => "east",
                                    Vector3(0, _, -1) => "south",
                                    _ => unreachable!(),
                                };

                                format!("repeater[facing={orientation}]")
                            } else {
                                "redstone_wire".to_string()
                            }
                        ));

                        obstacles.insert(pos);
                        j += 1;
                    }
                    println!("[EDGE {}/{}] [{}/{size}] Path found!", edge_i + 1, edges.len(), i + 1);
                } else {
                    println!("[EDGE {}/{}] [{}/{size}] No path found.", edge_i + 1, edges.len(), i + 1);
                }
            }

            edge_i += 1;
        }
    }
}
