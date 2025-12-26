mod api;
mod data;

use data::{FlightPath, StarMap, StarNode, SystemMarker, UserData};
use eframe::egui;
use petgraph::graph::NodeIndex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use wasm_bindgen::prelude::*;

const AUTH_TOKEN_KEY: &str = "fio_auth_token";
const USERNAME_KEY: &str = "fio_username";

fn get_local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

fn save_auth(token: &str, username: &str) {
    if let Some(storage) = get_local_storage() {
        let _ = storage.set_item(AUTH_TOKEN_KEY, token);
        let _ = storage.set_item(USERNAME_KEY, username);
    }
}

fn load_auth() -> Option<(String, String)> {
    let storage = get_local_storage()?;
    let token = storage.get_item(AUTH_TOKEN_KEY).ok()??;
    let username = storage.get_item(USERNAME_KEY).ok()??;
    Some((token, username))
}

fn clear_auth() {
    if let Some(storage) = get_local_storage() {
        let _ = storage.remove_item(AUTH_TOKEN_KEY);
        let _ = storage.remove_item(USERNAME_KEY);
    }
}

pub struct StarMapApp {
    star_map: Option<Arc<StarMap>>,
    loading: bool,
    error: Option<String>,
    view: MapView,
    selected_star: Option<NodeIndex>,
    hovered_star: Option<NodeIndex>,
    search_query: String,
    show_connections: bool,
    show_labels: bool,
    
    // Authentication
    auth_token: Option<String>,
    username: String,
    password: String,
    login_error: Option<String>,
    logging_in: bool,
    
    // User data
    user_data: Option<UserData>,
    loading_user_data: bool,
    
    // Exchange stations (public data)
    cx_system_ids: HashSet<String>,
    cx_names: HashMap<String, String>, // system_id -> CX name
    
    // System markers (computed from CX + user data) - now stores all markers per system
    system_markers: HashMap<String, Vec<SystemMarker>>,
    
    // Show markers toggle
    show_cx: bool,
    show_bases: bool,
    show_ships: bool,
}

struct MapView {
    offset: egui::Vec2,
    zoom: f32,
    projection: Projection,
}

#[derive(Clone, Copy, PartialEq)]
enum Projection {
    XY,
    XZ,
    YZ,
}

impl Default for MapView {
    fn default() -> Self {
        MapView {
            offset: egui::Vec2::ZERO,
            zoom: 0.3,
            projection: Projection::XY,
        }
    }
}

impl Default for StarMapApp {
    fn default() -> Self {
        Self {
            star_map: None,
            loading: false,
            error: None,
            view: MapView::default(),
            selected_star: None,
            hovered_star: None,
            search_query: String::new(),
            show_connections: true,
            show_labels: false,
            
            auth_token: None,
            username: String::new(),
            password: String::new(),
            login_error: None,
            logging_in: false,
            
            user_data: None,
            loading_user_data: false,
            
            cx_system_ids: HashSet::new(),
            cx_names: HashMap::new(),
            system_markers: HashMap::new(),
            
            show_cx: true,
            show_bases: true,
            show_ships: true,
        }
    }
}

impl StarMapApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn update_system_markers(&mut self) {
        self.system_markers.clear();
        
        // Collect all system IDs that have any marker
        let mut all_system_ids: HashSet<String> = HashSet::new();
        
        if self.show_cx {
            all_system_ids.extend(self.cx_system_ids.iter().cloned());
        }
        
        if let Some(user_data) = &self.user_data {
            if self.show_bases {
                all_system_ids.extend(user_data.base_system_ids.iter().cloned());
            }
            if self.show_ships {
                all_system_ids.extend(user_data.ship_system_ids.iter().cloned());
                // Also add in-system flights as ship markers
                for flight in &user_data.flight_paths {
                    if flight.is_in_system {
                        all_system_ids.insert(flight.origin_system_id.clone());
                    }
                }
            }
        }
        
        // For each system, collect all applicable markers in priority order (outer to inner)
        // CX (red) -> Base (green) -> Ship (blue)
        for system_id in all_system_ids {
            let mut markers = Vec::new();
            
            if self.show_cx && self.cx_system_ids.contains(&system_id) {
                markers.push(SystemMarker::CommodityExchange);
            }
            
            if let Some(user_data) = &self.user_data {
                if self.show_bases && user_data.base_system_ids.contains(&system_id) {
                    markers.push(SystemMarker::Base);
                }
                if self.show_ships {
                    // Check for docked ships
                    let has_docked_ship = user_data.ship_system_ids.contains(&system_id);
                    // Check for in-system flights
                    let has_in_system_flight = user_data.flight_paths.iter()
                        .any(|f| f.is_in_system && f.origin_system_id == system_id);
                    
                    if has_docked_ship || has_in_system_flight {
                        markers.push(SystemMarker::Ship);
                    }
                }
            }
            
            if !markers.is_empty() {
                self.system_markers.insert(system_id, markers);
            }
        }
    }

    fn world_to_screen(&self, node: &StarNode, rect: egui::Rect) -> egui::Pos2 {
        let (x, y) = match self.view.projection {
            Projection::XY => (node.position[0], node.position[1]),
            Projection::XZ => (node.position[0], node.position[2]),
            Projection::YZ => (node.position[1], node.position[2]),
        };

        let center = rect.center();
        egui::Pos2::new(
            center.x + (x * self.view.zoom) + self.view.offset.x,
            center.y + (y * self.view.zoom) + self.view.offset.y,
        )
    }

    fn draw_map(&mut self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(
            ui.available_size(),
            egui::Sense::click_and_drag(),
        );

        let rect = response.rect;

        // Handle panning
        if response.dragged() {
            self.view.offset += response.drag_delta();
        }

        // Handle zooming
        if let Some(hover_pos) = response.hover_pos() {
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 {
                let zoom_factor = 1.0 + scroll * 0.001;
                let old_zoom = self.view.zoom;
                self.view.zoom = (self.view.zoom * zoom_factor).clamp(0.05, 5.0);

                // Zoom towards cursor
                let zoom_change = self.view.zoom / old_zoom;
                let cursor_offset = hover_pos - rect.center() - self.view.offset;
                self.view.offset -= cursor_offset * (zoom_change - 1.0);
            }
        }

        // Draw background
        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(10, 10, 20));

        if let Some(star_map) = &self.star_map {
            let star_map = Arc::clone(star_map);
            
            // Draw connections first (behind stars)
            if self.show_connections {
                for edge in star_map.graph.edge_indices() {
                    if let Some((a, b)) = star_map.graph.edge_endpoints(edge) {
                        let node_a = &star_map.graph[a];
                        let node_b = &star_map.graph[b];
                        let pos_a = self.world_to_screen(node_a, rect);
                        let pos_b = self.world_to_screen(node_b, rect);

                        // Only draw if at least one endpoint is visible
                        if rect.contains(pos_a) || rect.contains(pos_b) {
                            painter.line_segment(
                                [pos_a, pos_b],
                                egui::Stroke::new(0.5, egui::Color32::from_rgba_unmultiplied(100, 100, 150, 80)),
                            );
                        }
                    }
                }
            }
            
            // Draw flight paths (blue lines with arrows for inter-system, rings handled with markers)
            let flight_color = egui::Color32::from_rgb(80, 160, 255);
            if self.show_ships {
                if let Some(user_data) = &self.user_data {
                    for flight in &user_data.flight_paths {
                        if !flight.is_in_system {
                            // Inter-system flight: draw line with arrow
                            if let (Some(origin_idx), Some(dest_idx)) = (
                                star_map.natural_id_to_node.get(&flight.origin_system_id),
                                star_map.natural_id_to_node.get(&flight.destination_system_id),
                            ) {
                                let origin_node = &star_map.graph[*origin_idx];
                                let dest_node = &star_map.graph[*dest_idx];
                                let pos_origin = self.world_to_screen(origin_node, rect);
                                let pos_dest = self.world_to_screen(dest_node, rect);
                                
                                // Only draw if at least one endpoint is visible
                                if rect.contains(pos_origin) || rect.contains(pos_dest) {
                                    // Draw the flight line (thicker than connections)
                                    painter.line_segment(
                                        [pos_origin, pos_dest],
                                        egui::Stroke::new(2.0, flight_color),
                                    );
                                    
                                    // Draw arrow at midpoint pointing towards destination
                                    let mid = pos_origin + (pos_dest - pos_origin) * 0.6;
                                    let dir = (pos_dest - pos_origin).normalized();
                                    let arrow_size = 8.0;
                                    let perp = egui::vec2(-dir.y, dir.x);
                                    
                                    let arrow_tip = mid + dir * arrow_size;
                                    let arrow_left = mid - dir * arrow_size * 0.5 + perp * arrow_size * 0.5;
                                    let arrow_right = mid - dir * arrow_size * 0.5 - perp * arrow_size * 0.5;
                                    
                                    painter.add(egui::Shape::convex_polygon(
                                        vec![arrow_tip, arrow_left, arrow_right],
                                        flight_color,
                                        egui::Stroke::NONE,
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            // Draw stars
            let mut new_hovered = None;
            for node_idx in star_map.graph.node_indices() {
                let node = &star_map.graph[node_idx];
                let pos = self.world_to_screen(node, rect);

                if !rect.contains(pos) {
                    continue;
                }

                let base_radius = 3.0 + self.view.zoom * 2.0;
                let is_selected = self.selected_star == Some(node_idx);
                let is_hovered = self.hovered_star == Some(node_idx);

                let radius = if is_selected {
                    base_radius * 1.5
                } else if is_hovered {
                    base_radius * 1.2
                } else {
                    base_radius
                };

                let star_color = node.star_type.color();

                // Check for hover
                if let Some(hover_pos) = response.hover_pos() {
                    if (hover_pos - pos).length() < radius + 5.0 {
                        new_hovered = Some(node_idx);
                    }
                }

                // Draw glow for selected/hovered
                if is_selected || is_hovered {
                    painter.circle_filled(
                        pos,
                        radius * 2.0,
                        egui::Color32::from_rgba_unmultiplied(star_color.r(), star_color.g(), star_color.b(), 30),
                    );
                }

                // Check for system markers (can be multiple stacked rings)
                let markers = self.system_markers.get(&node.natural_id);
                
                // Draw stacked marker rings if present (outer to inner: CX -> Base -> Ship)
                if let Some(markers) = markers {
                    let ring_width = 2.5;
                    let ring_gap = 1.0;
                    
                    // Draw rings from outside in
                    for (i, marker) in markers.iter().enumerate() {
                        let marker_color = marker.color();
                        let ring_radius = radius + 3.0 + (markers.len() - 1 - i) as f32 * (ring_width + ring_gap);
                        
                        painter.circle_stroke(
                            pos,
                            ring_radius,
                            egui::Stroke::new(ring_width, marker_color),
                        );
                    }
                    
                    // Draw inner glow using the innermost marker's color
                    if let Some(innermost) = markers.last() {
                        let glow_color = innermost.color();
                        painter.circle_filled(
                            pos,
                            radius + 1.0,
                            egui::Color32::from_rgba_unmultiplied(glow_color.r(), glow_color.g(), glow_color.b(), 40),
                        );
                    }
                }

                painter.circle_filled(pos, radius, star_color);

                // Draw label
                let has_markers = markers.is_some();
                if self.show_labels || is_hovered || is_selected || has_markers {
                    let label_text = if let Some(cx_name) = self.cx_names.get(&node.natural_id) {
                        format!("{} ({})", node.name, cx_name)
                    } else {
                        node.name.clone()
                    };
                    
                    // Offset label based on number of rings
                    let label_offset = if let Some(m) = markers {
                        radius + 5.0 + m.len() as f32 * 3.5
                    } else {
                        radius + 5.0
                    };
                    
                    painter.text(
                        pos + egui::vec2(label_offset, 0.0),
                        egui::Align2::LEFT_CENTER,
                        &label_text,
                        egui::FontId::proportional(10.0),
                        egui::Color32::WHITE,
                    );
                }
            }

            self.hovered_star = new_hovered;

            // Handle click selection
            if response.clicked() {
                self.selected_star = self.hovered_star;
            }
        }
    }

    fn draw_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.heading("Star Map Controls");
        ui.separator();

        // Loading/status
        if self.loading {
            ui.spinner();
            ui.label("Loading star data...");
        } else if let Some(error) = &self.error {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
        } else if let Some(star_map) = &self.star_map {
            ui.label(format!("Stars: {}", star_map.node_count()));
            ui.label(format!("Connections: {}", star_map.edge_count()));
            ui.label(format!("CX Stations: {}", self.cx_system_ids.len()));
        }

        ui.separator();

        // Projection selection
        ui.label("Projection:");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.view.projection, Projection::XY, "X-Y");
            ui.selectable_value(&mut self.view.projection, Projection::XZ, "X-Z");
            ui.selectable_value(&mut self.view.projection, Projection::YZ, "Y-Z");
        });

        ui.separator();

        // View options
        ui.checkbox(&mut self.show_connections, "Show connections");
        ui.checkbox(&mut self.show_labels, "Show all labels");

        ui.separator();
        
        // Marker visibility
        ui.label("Show markers:");
        let mut markers_changed = false;
        markers_changed |= ui.checkbox(&mut self.show_cx, "🔴 Commodity Exchanges").changed();
        markers_changed |= ui.checkbox(&mut self.show_bases, "🟢 Bases").changed();
        markers_changed |= ui.checkbox(&mut self.show_ships, "🔵 Ships").changed();
        
        if markers_changed {
            self.update_system_markers();
        }

        ui.separator();

        // Zoom controls
        ui.label(format!("Zoom: {:.2}x", self.view.zoom));
        ui.horizontal(|ui| {
            if ui.button("-").clicked() {
                self.view.zoom = (self.view.zoom * 0.8).max(0.05);
            }
            if ui.button("+").clicked() {
                self.view.zoom = (self.view.zoom * 1.25).min(5.0);
            }
            if ui.button("Reset").clicked() {
                self.view = MapView::default();
            }
        });

        ui.separator();

        // Search
        ui.label("Search:");
        ui.text_edit_singleline(&mut self.search_query);
        
        if !self.search_query.is_empty() {
            if let Some(star_map) = &self.star_map {
                let query = self.search_query.to_lowercase();
                let matches: Vec<_> = star_map.graph.node_indices()
                    .filter(|&idx| {
                        let node = &star_map.graph[idx];
                        node.name.to_lowercase().contains(&query) ||
                        node.natural_id.to_lowercase().contains(&query)
                    })
                    .take(10)
                    .collect();

                for idx in matches {
                    let node = &star_map.graph[idx];
                    if ui.selectable_label(
                        self.selected_star == Some(idx),
                        &node.name
                    ).clicked() {
                        self.selected_star = Some(idx);
                        // Center on selected star
                        let pos = node.position;
                        let (x, y) = match self.view.projection {
                            Projection::XY => (pos[0], pos[1]),
                            Projection::XZ => (pos[0], pos[2]),
                            Projection::YZ => (pos[1], pos[2]),
                        };
                        self.view.offset = egui::vec2(-x * self.view.zoom, -y * self.view.zoom);
                    }
                }
            }
        }

        ui.separator();

        // Selected star info
        if let Some(selected_idx) = self.selected_star {
            if let Some(star_map) = &self.star_map {
                let node = &star_map.graph[selected_idx];
                ui.heading(&node.name);
                ui.label(format!("ID: {}", node.natural_id));
                ui.label(format!("Type: {:?}", node.star_type));
                ui.label(format!("Position: ({:.1}, {:.1}, {:.1})", 
                    node.position[0], node.position[1], node.position[2]));
                ui.label(format!("Sector: {}", node.sector_id));
                
                // Show marker info (all markers for this system)
                if let Some(markers) = self.system_markers.get(&node.natural_id) {
                    for marker in markers {
                        let marker_text = match marker {
                            SystemMarker::CommodityExchange => {
                                if let Some(cx_name) = self.cx_names.get(&node.natural_id) {
                                    format!("🔴 CX: {}", cx_name)
                                } else {
                                    "🔴 Commodity Exchange".to_string()
                                }
                            }
                            SystemMarker::Base => "🟢 Your Base".to_string(),
                            SystemMarker::Ship => "🔵 Your Ship".to_string(),
                        };
                        ui.colored_label(marker.color(), marker_text);
                    }
                }

                // Show connections
                let neighbors: Vec<_> = star_map.graph.neighbors(selected_idx).collect();
                if !neighbors.is_empty() {
                    ui.label(format!("Connections: {}", neighbors.len()));
                    egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                        for neighbor_idx in neighbors {
                            let neighbor = &star_map.graph[neighbor_idx];
                            if ui.selectable_label(false, &neighbor.name).clicked() {
                                self.selected_star = Some(neighbor_idx);
                            }
                        }
                    });
                }
            }
        }
    }
    
    fn draw_auth_panel(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.heading("FIO Login");
        
        if self.auth_token.is_some() {
            ui.label(format!("✅ Logged in as: {}", self.username));
            
            if self.loading_user_data {
                ui.spinner();
                ui.label("Loading user data...");
            } else if let Some(user_data) = &self.user_data {
                ui.label(format!("Ships: {} systems", user_data.ship_system_ids.len()));
                ui.label(format!("Bases: {} systems", user_data.base_system_ids.len()));
            }
            
            if ui.button("Logout").clicked() {
                self.auth_token = None;
                self.user_data = None;
                self.username.clear();
                self.password.clear();
                clear_auth();
                self.update_system_markers();
            }
        } else {
            ui.label("Username:");
            ui.text_edit_singleline(&mut self.username);
            
            ui.label("Password:");
            let password_edit = egui::TextEdit::singleline(&mut self.password)
                .password(true);
            ui.add(password_edit);
            
            if let Some(error) = &self.login_error {
                ui.colored_label(egui::Color32::RED, error);
            }
            
            let can_login = !self.username.is_empty() && !self.password.is_empty() && !self.logging_in;
            
            ui.add_enabled_ui(can_login, |ui| {
                if ui.button("Login").clicked() {
                    self.logging_in = true;
                    self.login_error = None;
                }
            });
            
            if self.logging_in {
                ui.spinner();
            }
        }
    }
}

impl eframe::App for StarMapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Side panel
        egui::SidePanel::left("controls")
            .min_width(200.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.draw_sidebar(ui);
                    self.draw_auth_panel(ui);
                });
            });

        // Main map area
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_map(ui);
        });

        // Request repaint for smooth interaction
        if self.hovered_star.is_some() || self.loading || self.logging_in || self.loading_user_data {
            ctx.request_repaint();
        }
    }
}

// WASM entry point
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let runner = eframe::WebRunner::new();
        
        // Get the canvas element
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id("canvas")
            .expect("Failed to find canvas element")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("canvas is not an HtmlCanvasElement");
        
        runner
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    let app = StarMapApp::new(cc);
                    Ok(Box::new(AppWrapper::new(app)))
                }),
            )
            .await
            .expect("Failed to start eframe");
    });

    Ok(())
}

// Message types for async operations
enum AppMessage {
    StarSystemsLoaded(Result<Vec<data::StarSystem>, String>),
    ExchangeStationsLoaded(Result<Vec<data::ExchangeStation>, String>),
    LoginResult(Result<(String, String), String>), // (auth_token, username)
    UserDataLoaded(Result<UserData, String>),
}

/// Fetch all user data (ships, flights, bases) from the API
async fn fetch_all_user_data(username: &str, auth_token: &str) -> UserData {
    let mut user_data = UserData {
        username: username.to_string(),
        ship_system_ids: HashSet::new(),
        base_system_ids: HashSet::new(),
        flight_paths: Vec::new(),
    };
    
    // Fetch ships (docked only - ships in flight have empty location)
    if let Ok(ships) = api::fetch_ships(username, auth_token).await {
        for ship in ships {
            if let Some(location) = ship.location {
                if !location.is_empty() {
                    user_data.ship_system_ids.insert(extract_system_from_planet(&location));
                }
            }
        }
    }
    
    // Fetch active flights
    if let Ok(flights) = api::fetch_flights(username, auth_token).await {
        for flight in flights {
            if let (Some(origin), Some(dest)) = (
                flight.origin_system_natural_id(),
                flight.destination_system_natural_id(),
            ) {
                user_data.flight_paths.push(FlightPath {
                    origin_system_id: origin.clone(),
                    destination_system_id: dest.clone(),
                    ship_registration: flight.ship_id,
                    is_in_system: origin == dest,
                });
            }
        }
    }
    
    // Fetch bases/sites
    if let Ok(sites) = api::fetch_sites(username, auth_token).await {
        for site in sites {
            if let Some(planet_id) = site.planet_identifier {
                user_data.base_system_ids.insert(extract_system_from_planet(&planet_id));
            }
        }
    }
    
    user_data
}

// Wrapper to handle async data loading
struct AppWrapper {
    app: StarMapApp,
    message_receiver: std::sync::mpsc::Receiver<AppMessage>,
    message_sender: std::sync::mpsc::Sender<AppMessage>,
}

impl AppWrapper {
    fn new(mut app: StarMapApp) -> Self {
        app.loading = true;
        
        let (tx, rx) = std::sync::mpsc::channel();
        
        // Fetch star systems
        let tx_stars = tx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = api::fetch_star_systems().await;
            let _ = tx_stars.send(AppMessage::StarSystemsLoaded(result));
        });
        
        // Fetch exchange stations (public endpoint)
        let tx_cx = tx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = api::fetch_exchange_stations().await;
            let _ = tx_cx.send(AppMessage::ExchangeStationsLoaded(result));
        });
        
        // Try to restore auth from localStorage
        if let Some((auth_token, username)) = load_auth() {
            app.auth_token = Some(auth_token.clone());
            app.username = username.clone();
            app.loading_user_data = true;
            
            let tx_user = tx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let user_data = fetch_all_user_data(&username, &auth_token).await;
                let _ = tx_user.send(AppMessage::UserDataLoaded(Ok(user_data)));
            });
        }
        
        Self {
            app,
            message_receiver: rx,
            message_sender: tx,
        }
    }
    
    fn handle_login(&self, username: String, password: String) {
        let tx = self.message_sender.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api::login(&username, &password).await {
                Ok(auth_response) => {
                    let _ = tx.send(AppMessage::LoginResult(Ok((auth_response.auth_token, username))));
                }
                Err(e) => {
                    let _ = tx.send(AppMessage::LoginResult(Err(e)));
                }
            }
        });
    }
    
    fn fetch_user_data(&self, username: String, auth_token: String) {
        let tx = self.message_sender.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let user_data = fetch_all_user_data(&username, &auth_token).await;
            let _ = tx.send(AppMessage::UserDataLoaded(Ok(user_data)));
        });
    }
}

// Extract system ID from planet identifier (e.g., "UV-351a" -> "UV-351")
fn extract_system_from_planet(planet_id: &str) -> String {
    // Planet IDs typically end with a lowercase letter (a, b, c, etc.)
    // System IDs are the part before that
    let chars: Vec<char> = planet_id.chars().collect();
    if let Some(last) = chars.last() {
        if last.is_ascii_lowercase() {
            return chars[..chars.len()-1].iter().collect();
        }
    }
    planet_id.to_string()
}

impl eframe::App for AppWrapper {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Process all pending messages
        while let Ok(msg) = self.message_receiver.try_recv() {
            match msg {
                AppMessage::StarSystemsLoaded(result) => {
                    match result {
                        Ok(systems) => {
                            self.app.star_map = Some(Arc::new(StarMap::from_systems(systems)));
                            self.app.loading = false;
                            self.app.update_system_markers();
                        }
                        Err(e) => {
                            self.app.error = Some(e);
                            self.app.loading = false;
                        }
                    }
                }
                AppMessage::ExchangeStationsLoaded(result) => {
                    match result {
                        Ok(stations) => {
                            for station in stations {
                                // Use SystemNaturalId to match with star map
                                self.app.cx_system_ids.insert(station.system_natural_id.clone());
                                self.app.cx_names.insert(station.system_natural_id, station.comex_code);
                            }
                            self.app.update_system_markers();
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load exchange stations: {}", e);
                        }
                    }
                }
                AppMessage::LoginResult(result) => {
                    self.app.logging_in = false;
                    match result {
                        Ok((auth_token, username)) => {
                            // Save to localStorage
                            save_auth(&auth_token, &username);
                            
                            self.app.auth_token = Some(auth_token.clone());
                            self.app.username = username.clone();
                            self.app.password.clear();
                            self.app.login_error = None;
                            self.app.loading_user_data = true;
                            
                            // Fetch user data
                            self.fetch_user_data(username, auth_token);
                        }
                        Err(e) => {
                            self.app.login_error = Some(e);
                        }
                    }
                }
                AppMessage::UserDataLoaded(result) => {
                    self.app.loading_user_data = false;
                    match result {
                        Ok(user_data) => {
                            self.app.user_data = Some(user_data);
                            self.app.update_system_markers();
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load user data: {}", e);
                        }
                    }
                }
            }
        }
        
        // Handle login button click
        if self.app.logging_in && self.app.auth_token.is_none() {
            let username = self.app.username.clone();
            let password = self.app.password.clone();
            if !username.is_empty() && !password.is_empty() {
                self.handle_login(username, password);
                // Prevent re-triggering
                self.app.logging_in = false;
                self.app.logging_in = true; // Keep spinner showing
            }
        }
        
        self.app.update(ctx, frame);
    }
}

