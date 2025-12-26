use petgraph::graph::{NodeIndex, UnGraph};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConnection {
    #[serde(rename = "SystemConnectionId")]
    pub system_connection_id: String,
    #[serde(rename = "ConnectingId")]
    pub connecting_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarSystem {
    #[serde(rename = "SystemId")]
    pub system_id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "NaturalId")]
    pub natural_id: String,
    #[serde(rename = "Type")]
    pub star_type: String,
    #[serde(rename = "PositionX")]
    pub position_x: f32,
    #[serde(rename = "PositionY")]
    pub position_y: f32,
    #[serde(rename = "PositionZ")]
    pub position_z: f32,
    #[serde(rename = "SectorId")]
    pub sector_id: String,
    #[serde(rename = "SubSectorId")]
    pub sub_sector_id: String,
    #[serde(rename = "Connections")]
    pub connections: Vec<SystemConnection>,
    #[serde(rename = "UserNameSubmitted")]
    pub user_name_submitted: String,
    #[serde(rename = "Timestamp")]
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct StarNode {
    pub name: String,
    pub natural_id: String,
    pub star_type: StarType,
    pub position: [f32; 3],
    pub sector_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StarType {
    O, // Blue
    B, // Blue-white
    A, // White
    F, // Yellow-white
    G, // Yellow (like our Sun)
    K, // Orange
    M, // Red
    Unknown,
}

impl StarType {
    pub fn from_str(s: &str) -> Self {
        match s.chars().next() {
            Some('O') => StarType::O,
            Some('B') => StarType::B,
            Some('A') => StarType::A,
            Some('F') => StarType::F,
            Some('G') => StarType::G,
            Some('K') => StarType::K,
            Some('M') => StarType::M,
            _ => StarType::Unknown,
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            StarType::O => egui::Color32::from_rgb(155, 176, 255),
            StarType::B => egui::Color32::from_rgb(170, 191, 255),
            StarType::A => egui::Color32::from_rgb(202, 215, 255),
            StarType::F => egui::Color32::from_rgb(248, 247, 255),
            StarType::G => egui::Color32::from_rgb(255, 244, 234),
            StarType::K => egui::Color32::from_rgb(255, 210, 161),
            StarType::M => egui::Color32::from_rgb(255, 204, 111),
            StarType::Unknown => egui::Color32::GRAY,
        }
    }
}

impl From<&StarSystem> for StarNode {
    fn from(sys: &StarSystem) -> Self {
        StarNode {
            name: sys.name.clone(),
            natural_id: sys.natural_id.clone(),
            star_type: StarType::from_str(&sys.star_type),
            position: [sys.position_x, sys.position_y, sys.position_z],
            sector_id: sys.sector_id.clone(),
        }
    }
}

pub struct StarMap {
    pub graph: UnGraph<StarNode, ()>,
    #[allow(dead_code)]
    id_to_index: HashMap<String, NodeIndex>,
}



impl StarMap {
    pub fn from_systems(systems: Vec<StarSystem>) -> Self {
        let mut graph = UnGraph::new_undirected();
        let mut id_to_index = HashMap::new();

        // First pass: add all nodes
        for sys in &systems {
            let node = StarNode::from(sys);
            let idx = graph.add_node(node);
            id_to_index.insert(sys.system_id.clone(), idx);
        }

        // Second pass: add edges
        for sys in &systems {
            if let Some(&from_idx) = id_to_index.get(&sys.system_id) {
                for conn in &sys.connections {
                    if let Some(&to_idx) = id_to_index.get(&conn.connecting_id) {
                        // Only add edge if it doesn't exist (undirected graph)
                        if !graph.contains_edge(from_idx, to_idx) {
                            graph.add_edge(from_idx, to_idx, ());
                        }
                    }
                }
            }
        }

        StarMap {
            graph,
            id_to_index,
        }
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}
