use crate::pathfinding::{is_obstacle_nearby, PathNode};
use std::{collections::HashSet, ops::Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vector3(pub i32, pub i32, pub i32);

impl Sub for Vector3 {
    type Output = Vector3;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

// Implementation of the pathfinding methods
impl Vector3 {
    pub fn neighbors(
        &self,
        start: &Vector3,
        goal: &Vector3,
        obstacles: &HashSet<Vector3>,
        entries: &Vec<Vector3>,
        path_node: &PathNode,
    ) -> Vec<(Vector3, u32)> {
        let &Vector3(x, y, z) = self;
        let mut neighbors = Vec::new();

        // TODO: Move the bounds values and make them customizable
        let (start_x, end_x, start_y, end_y, start_z, end_z) = (-10, 50, 140, 180, -30, 30);

        let redstone_step: i32 = (path_node.steps_from_start as i32 - 1) % 14;
        let previous_direction = path_node
            .previous
            .map(|prev| Vector3(x - prev.0, y - prev.1, z - prev.2));

        let directions = [
            (1, 0, 0, false),  // Right
            (-1, 0, 0, false), // Left
            (1, 1, 0, true),   // Right up
            (1, -1, 0, true),  // Right down
            (-1, 1, 0, true),  // Left up
            (-1, -1, 0, true), // Left down
            (0, 1, 1, true),   // Forward up
            (0, -1, 1, true),  // Forward down
            (0, 1, -1, true),  // Backward up
            (0, -1, -1, true), // Backward down
            (0, 0, 1, false),  // Forward
            (0, 0, -1, false), // Backward
        ];

        for &(dx, dy, dz, is_diagonal) in &directions {
            let next_pos = Vector3(x + dx, y + dy, z + dz);
            let dist_to_goal = next_pos.distance(goal);
            let dist_to_start = next_pos.distance(start);

            // Out of bounds check
            if next_pos.0 < start_x
                || next_pos.0 > end_x
                || next_pos.1 < start_y
                || next_pos.1 > end_y
                || next_pos.2 < start_z
                || next_pos.2 > end_z
            {
                continue;
            }

            if is_diagonal {
                // The 8 first blocks cannot be in diagonal
                if dist_to_goal < 8 || dist_to_start < 8 {
                    continue;
                }
            }

            if let Some(prev_dir) = previous_direction {
                if is_diagonal {
                    // If diagonal, it must go in the same direction (and not backward which is not possible in redstone)
                    if (dx, dz) != (prev_dir.0, prev_dir.2) {
                        continue;
                    }
                }

                // Cannot go in the opposite direction (X)
                if (dx, dy, dz) == (-prev_dir.0, 0, 0) {
                    continue;
                }
                // Cannot go in the opposite direction (Z)
                if (dx, dy, dz) == (0, 0, -prev_dir.2) {
                    continue;
                }
            }

            // If it is next to the start or goal, allow it
            if next_pos == *start || next_pos == *goal {
                neighbors.push((next_pos, 1));
                continue;
            }

            // If next to the start or goal, allow it
            if dist_to_start == 1 || dist_to_goal == 1 {
                neighbors.push((next_pos, 1));
                continue;
            }

            // If step == 0, this is a repeater
            if redstone_step == 0 {
                // Cannot change direction
                if let Some(prev_dir) = previous_direction {
                    if (dx, dy, dz) != (prev_dir.0, prev_dir.1, prev_dir.2) {
                        continue;
                    }
                }
                // Cannot go in diagonal
                if is_diagonal {
                    continue;
                }
            }

            // If the next will be repeater, cannot go in diagonal down
            if redstone_step == 13 && is_diagonal && dy < 0 {
                continue;
            }

            if !is_obstacle_nearby(obstacles, next_pos) {
                // TODO: The cost values and algorithm need to be adjusted. Currently there are some issues in the final paths
                let mut cost = 1.0;

                for v in entries {
                    let dist = next_pos.distance(v);
                    cost += 1.0 / (dist as f32) * 100.0;
                }

                if is_diagonal {
                    cost += 50.0
                }

                neighbors.push((next_pos, cost as u32));
            }
        }

        neighbors
    }

    /// Manhattan distance
    pub fn distance(&self, other: &Vector3) -> u32 {
        ((self.0 - other.0).abs() + (self.1 - other.1).abs() + (self.2 - other.2).abs())
            .try_into()
            .unwrap()
    }
}
