use crate::{
    minecraft::structures::{read_minecraft_structure_file, MinecraftStructureNbt},
    parser::ast::Operator,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Port {
    pub id: String,
    pub size: u8,
}

impl Port {
    pub fn new(size: u8) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            size,
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeType {
    // Secondary nodes
    Operator(Operator),
    VarInt { name: String, value: u32 },
    Int(u32),
    IntRef,

    // Primary nodes
    Start,
    End,
    If,
    Loop,
    Wait,
    Set,
}

impl NodeType {
    /// Based on the structure files
    pub fn get_inputs(t: NodeType) -> Vec<Port> {
        match t {
            Self::Start => {
                vec![]
            }
            Self::Set => {
                vec![Port::new(1), Port::new(8)]
            }
            Self::Operator { .. } => {
                vec![Port::new(8), Port::new(8)]
            }
            Self::VarInt { .. } => {
                vec![Port::new(8), Port::new(1)]
            }
            Self::Int(_) => {
                vec![Port::new(8), Port::new(1)]
            }
            Self::IntRef => {
                vec![Port::new(8)]
            }
            _ => unimplemented!("inputs list of {t:?}"),
        }
    }

    pub fn get_nbt(t: NodeType) -> MinecraftStructureNbt {
        let name = NodeType::get_name(t);
        read_minecraft_structure_file(&format!("redsharp\\data\\redsharp\\structure\\{name}.nbt"))
    }

    pub fn get_outputs(t: NodeType) -> Vec<Port> {
        match t {
            Self::Start => {
                vec![Port::new(1)]
            }
            Self::Set => {
                vec![Port::new(1), Port::new(8)]
            }
            Self::Operator(_) => {
                vec![Port::new(8)]
            }
            Self::VarInt { .. } => {
                vec![Port::new(8)]
            }
            Self::Int(_) => {
                vec![Port::new(8)]
            }
            Self::IntRef => {
                vec![Port::new(8)]
            }
            _ => unimplemented!("outputs list of {t:?}"),
        }
    }

    pub fn get_name(t: NodeType) -> &'static str {
        match t {
            Self::Start => "start",
            Self::Set => "int",
            Self::Operator(op) => match op {
                Operator::Add => "adder",
                _ => unimplemented!("operator {op:?}"),
            },
            Self::VarInt { .. } => "int",
            Self::Int(_) => "int",
            _ => unimplemented!("node get name"),
        }
    }
}

#[derive(Debug, Clone)]
/// A node is the representation of a Minecraft structure.
/// A node will be connected to other nodes via its inputs and outputs.
pub struct Node {
    pub id: String,
    pub node: NodeType,
    pub is_primary: bool,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
}

impl Node {
    pub fn from(node: NodeType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            node: node.clone(),
            is_primary: matches!(
                node,
                NodeType::If | NodeType::Loop | NodeType::Set | NodeType::Wait
            ),
            inputs: NodeType::get_inputs(node.clone()),
            outputs: NodeType::get_outputs(node),
        }
    }

    pub fn get_input_id(&self, size: u8, index: usize) -> String {
        self.inputs
            .iter()
            .filter(|x| x.size == size)
            .collect::<Vec<&Port>>()[index]
            .id
            .clone()
    }

    pub fn get_output_id(&self, size: u8, index: usize) -> String {
        self.outputs
            .iter()
            .filter(|x| x.size == size)
            .collect::<Vec<&Port>>()[index]
            .id
            .clone()
    }
}
