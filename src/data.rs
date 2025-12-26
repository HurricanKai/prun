use petgraph::graph::{NodeIndex, UnGraph};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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

// Exchange station data from /exchange/station
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeStation {
    #[serde(rename = "StationId")]
    pub station_id: String,
    #[serde(rename = "NaturalId")]
    pub natural_id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "SystemId")]
    pub system_id: String,
    #[serde(rename = "SystemNaturalId")]
    pub system_natural_id: String,
    #[serde(rename = "SystemName")]
    pub system_name: String,
    #[serde(rename = "ComexCode")]
    pub comex_code: String,
    #[serde(rename = "ComexName")]
    pub comex_name: String,
}

// Ship data from /ship/ships/{username}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
    #[serde(rename = "ShipId")]
    pub ship_id: String,
    #[serde(rename = "StoreId")]
    pub store_id: Option<String>,
    #[serde(rename = "StlFuelStoreId")]
    pub stl_fuel_store_id: Option<String>,
    #[serde(rename = "FtlFuelStoreId")]
    pub ftl_fuel_store_id: Option<String>,
    #[serde(rename = "Registration")]
    pub registration: String,
    #[serde(rename = "Name")]
    pub name: Option<String>,
    #[serde(rename = "CommissioningTimeEpochMs")]
    pub commissioning_time_epoch_ms: Option<i64>,
    #[serde(rename = "BlueprintNaturalId")]
    pub blueprint_natural_id: Option<String>,
    #[serde(rename = "FlightId")]
    pub flight_id: Option<String>,
    #[serde(rename = "Acceleration")]
    pub acceleration: Option<f64>,
    #[serde(rename = "Thrust")]
    pub thrust: Option<f64>,
    #[serde(rename = "Mass")]
    pub mass: Option<f64>,
    #[serde(rename = "OperatingEmptyMass")]
    pub operating_empty_mass: Option<f64>,
    #[serde(rename = "ReactorPower")]
    pub reactor_power: Option<f64>,
    #[serde(rename = "EmitterPower")]
    pub emitter_power: Option<f64>,
    #[serde(rename = "Volume")]
    pub volume: Option<f64>,
    #[serde(rename = "Weight")]
    pub weight: Option<f64>,
    #[serde(rename = "StlFuelFlowRate")]
    pub stl_fuel_flow_rate: Option<f64>,
    #[serde(rename = "Condition")]
    pub condition: Option<f64>,
    #[serde(rename = "RepairMaterials")]
    pub repair_materials: Option<Vec<serde_json::Value>>,
    #[serde(rename = "LastRepairEpochMs")]
    pub last_repair_epoch_ms: Option<i64>,
    #[serde(rename = "Location")]
    pub location: Option<String>,
    #[serde(rename = "UserNameSubmitted")]
    pub user_name_submitted: Option<String>,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<String>,
}

// Site data from /sites/{username}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    #[serde(rename = "SiteId")]
    pub site_id: String,
    #[serde(rename = "PlanetId")]
    pub planet_id: String,
    #[serde(rename = "PlanetIdentifier")]
    pub planet_identifier: Option<String>,
    #[serde(rename = "PlanetName")]
    pub planet_name: Option<String>,
    #[serde(rename = "PlanetFoundedEpochMs")]
    pub planet_founded_epoch_ms: Option<i64>,
    #[serde(rename = "InvestedPermits")]
    pub invested_permits: Option<i32>,
    #[serde(rename = "MaximumPermits")]
    pub maximum_permits: Option<i32>,
    #[serde(rename = "UserNameSubmitted")]
    pub user_name_submitted: Option<String>,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<String>,
}

// Auth response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    #[serde(rename = "AuthToken")]
    pub auth_token: String,
    #[serde(rename = "Expiry")]
    pub expiry: Option<String>,
}

// Flight line (part of origin/destination address)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlightLine {
    #[serde(rename = "Type", default)]
    pub line_type: Option<String>,
    #[serde(rename = "LineId", default)]
    pub line_id: Option<String>,
    #[serde(rename = "LineNaturalId", default)]
    pub line_natural_id: Option<String>,
    #[serde(rename = "LineName", default)]
    pub line_name: Option<String>,
}

// Flight segment data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlightSegment {
    #[serde(rename = "Type", default)]
    pub segment_type: Option<String>,
    #[serde(rename = "Origin", default)]
    pub origin: Option<String>,
    #[serde(rename = "Destination", default)]
    pub destination: Option<String>,
    #[serde(rename = "OriginLines", default)]
    pub origin_lines: Option<Vec<FlightLine>>,
    #[serde(rename = "DestinationLines", default)]
    pub destination_lines: Option<Vec<FlightLine>>,
}

// Flight data from /ship/flights/{username}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Flight {
    #[serde(rename = "FlightId", default)]
    pub flight_id: Option<String>,
    #[serde(rename = "ShipId", default)]
    pub ship_id: Option<String>,
    #[serde(rename = "Origin", default)]
    pub origin: Option<String>,
    #[serde(rename = "Destination", default)]
    pub destination: Option<String>,
    #[serde(rename = "Segments", default)]
    pub segments: Option<Vec<FlightSegment>>,
    #[serde(rename = "DepartureTimeEpochMs", default)]
    pub departure_time_epoch_ms: Option<i64>,
    #[serde(rename = "ArrivalTimeEpochMs", default)]
    pub arrival_time_epoch_ms: Option<i64>,
    #[serde(rename = "CurrentSegmentIndex", default)]
    pub current_segment_index: Option<i32>,
}

impl Flight {
    /// Extract the origin system natural ID from the first segment's origin lines
    pub fn origin_system_natural_id(&self) -> Option<String> {
        self.segments.as_ref()?.first()?
            .origin_lines.as_ref()?
            .iter()
            .find(|line| line.line_type.as_deref() == Some("system"))
            .and_then(|line| line.line_natural_id.clone())
    }
    
    /// Extract the destination system natural ID from the last segment's destination lines
    pub fn destination_system_natural_id(&self) -> Option<String> {
        self.segments.as_ref()?.last()?
            .destination_lines.as_ref()?
            .iter()
            .find(|line| line.line_type.as_deref() == Some("system"))
            .and_then(|line| line.line_natural_id.clone())
    }
}

// Processed flight for visualization
#[derive(Debug, Clone)]
pub struct FlightPath {
    pub origin_system_id: String,
    pub destination_system_id: String,
    #[allow(dead_code)]
    pub ship_registration: Option<String>,
    pub is_in_system: bool, // true if origin == destination (in-system flight)
}

// User data aggregated from various endpoints
#[derive(Debug, Clone, Default)]
pub struct UserData {
    #[allow(dead_code)]
    pub username: String,
    pub ship_system_ids: HashSet<String>,
    pub base_system_ids: HashSet<String>,
    pub flight_paths: Vec<FlightPath>,
}

// System markers for visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemMarker {
    CommodityExchange,
    Base,
    Ship,
}

impl SystemMarker {
    pub fn color(&self) -> egui::Color32 {
        match self {
            SystemMarker::CommodityExchange => egui::Color32::from_rgb(255, 100, 100), // Red
            SystemMarker::Base => egui::Color32::from_rgb(100, 255, 100), // Green
            SystemMarker::Ship => egui::Color32::from_rgb(100, 150, 255), // Blue
        }
    }
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
    pub natural_id_to_node: HashMap<String, NodeIndex>,
}



impl StarMap {
    pub fn from_systems(systems: Vec<StarSystem>) -> Self {
        let mut graph = UnGraph::new_undirected();
        let mut id_to_index = HashMap::new();
        let mut natural_id_to_node = HashMap::new();

        // First pass: add all nodes
        for sys in &systems {
            let node = StarNode::from(sys);
            let idx = graph.add_node(node);
            id_to_index.insert(sys.system_id.clone(), idx);
            natural_id_to_node.insert(sys.natural_id.clone(), idx);
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
            natural_id_to_node,
        }
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}
