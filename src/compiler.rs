use super::nodes::{Node, NodeType};
use crate::parser::ast::{Expr, Operator, Statement};

/// Compile the given AST into nodes
pub struct Compiler {
    pub nodes: Vec<Node>,
    /// (node1_id, port1_id, node2_id, port2_id)
    pub edges: Vec<(String, String, String, String)>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    fn get_variable_structure(&self, var_name: String) -> Option<&Node> {
        for s in &self.nodes {
            if let NodeType::VarInt { name, .. } = &s.node {
                if name.eq(&var_name) {
                    return Some(s);
                }
            }
        }
        None
    }

    pub fn compile(&mut self, statements: Vec<Statement>) {
        for statement in statements {
            self.compile_statement(statement);
        }
    }

    fn compile_statement(&mut self, statement: Statement) {
        match statement {
            Statement::Expression(expr) => {
                let node = self.compile_expression(expr);
                self.nodes.push(node);
            }
            Statement::Let(name, expr) => {
                self.compile_let(name, expr);
            }
            Statement::Block(block) => {
                self.compile(block);
            }
            _ => unimplemented!("Statement type ({statement:?})"),
        };
    }

    fn compile_let(&mut self, name: String, expr: Expr) {
        if let Expr::Int { value } = expr {
            let s = Node::from(NodeType::VarInt {
                name: name.clone(),
                value: value.clone(),
            });

            self.nodes.push(s);
        } else {
            panic!("Invalid declare value");
        }
    }

    fn parse_identifier(&self, expr: Expr) -> Option<&Node> {
        if let Expr::Identifier(name) = expr {
            if let Some(s) = self.get_variable_structure(name.into()) {
                return Some(s);
            }
        }

        None
    }

    pub fn compile_expression(&mut self, node: Expr) -> Node {
        match node {
            Expr::Identifier(name) => {
                if let Some(_) = self.get_variable_structure(name.clone().into()) {
                    return Node::from(NodeType::IntRef).into();
                } else {
                    panic!("Invalid variable ({name})");
                }
            }
            Expr::Int { value } => {
                let s = Node::from(NodeType::Int(value.clone()));
                return s.into();
            }
            Expr::Infix {
                left,
                operator,
                right,
            } => match operator {
                Operator::Add => {
                    let op_node = Node::from(NodeType::Operator(operator.clone()));

                    if let Some(lhs_node) = self.parse_identifier(*left.clone()) {
                        self.edges.push((
                            lhs_node.id.clone(),
                            lhs_node.get_output_id(8, 0),
                            op_node.id.clone(),
                            op_node.get_input_id(8, 0),
                        ));
                    } else {
                        let lhs_node = self.compile_expression(*left);
                        self.edges.push((
                            lhs_node.id.clone(),
                            lhs_node.get_output_id(8, 0),
                            op_node.id.clone(),
                            op_node.get_input_id(8, 0),
                        ));
                        self.nodes.push(lhs_node);
                    }

                    if let Some(rhs_node) = self.parse_identifier(*right.clone()) {
                        self.edges.push((
                            rhs_node.id.clone(),
                            rhs_node.get_output_id(8, 0),
                            op_node.id.clone(),
                            op_node.get_input_id(8, 1),
                        ));
                    } else {
                        let rhs_node = self.compile_expression(*right);
                        self.edges.push((
                            rhs_node.id.clone(),
                            rhs_node.get_output_id(8, 0),
                            op_node.id.clone(),
                            op_node.get_input_id(8, 1),
                        ));
                        self.nodes.push(rhs_node);
                    }

                    op_node
                }
                _ => unimplemented!("Infix operator ({operator:?})"),
            },
            Expr::Assignment { left, right } => {
                let set_node = Node::from(NodeType::Set);

                if let Some(rhs_node) = self.parse_identifier(*right.clone()) {
                    self.edges.push((
                        rhs_node.id.clone(),
                        rhs_node.get_output_id(8, 0),
                        set_node.id.clone(),
                        set_node.get_input_id(8, 0),
                    ));
                } else {
                    let rhs_node = self.compile_expression(*right);
                    self.edges.push((
                        rhs_node.id.clone(),
                        rhs_node.get_output_id(8, 0),
                        set_node.id.clone(),
                        set_node.get_input_id(8, 0),
                    ));
                    self.nodes.push(rhs_node);
                }

                if let Some(lhs_node) = self.parse_identifier(*left.clone()) {
                    self.edges.push((
                        set_node.id.clone(),
                        set_node.get_output_id(8, 0),
                        lhs_node.id.clone(),
                        lhs_node.get_input_id(8, 0),
                    ));
                } else {
                    let lhs_node = self.compile_expression(*left);
                    self.edges.push((
                        set_node.id.clone(),
                        set_node.get_output_id(8, 0),
                        lhs_node.id.clone(),
                        lhs_node.get_input_id(8, 0),
                    ));
                    self.nodes.push(lhs_node);
                }

                set_node
            }
            _ => unimplemented!("Node type ({node:?})"),
        }
    }
}
