mod api;
mod data;

use data::{StarMap, StarNode};
use eframe::egui;
use petgraph::graph::NodeIndex;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

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
        }
    }
}

impl StarMapApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
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

                let color = node.star_type.color();

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
                        egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 30),
                    );
                }

                painter.circle_filled(pos, radius, color);

                // Draw label
                if self.show_labels || is_hovered || is_selected {
                    painter.text(
                        pos + egui::vec2(radius + 3.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        &node.name,
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
}

impl eframe::App for StarMapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Side panel
        egui::SidePanel::left("controls")
            .min_width(200.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.draw_sidebar(ui);
                });
            });

        // Main map area
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_map(ui);
        });

        // Request repaint for smooth interaction
        if self.hovered_star.is_some() || self.loading {
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

// Wrapper to handle async data loading
struct AppWrapper {
    app: StarMapApp,
    data_receiver: Option<std::sync::mpsc::Receiver<Result<Vec<data::StarSystem>, String>>>,
}

impl AppWrapper {
    fn new(mut app: StarMapApp) -> Self {
        app.loading = true;
        
        let (tx, rx) = std::sync::mpsc::channel();
        
        wasm_bindgen_futures::spawn_local(async move {
            let result = api::fetch_star_systems().await;
            let _ = tx.send(result);
        });
        
        Self {
            app,
            data_receiver: Some(rx),
        }
    }
}

impl eframe::App for AppWrapper {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(rx) = &self.data_receiver {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(systems) => {
                        self.app.star_map = Some(Arc::new(StarMap::from_systems(systems)));
                        self.app.loading = false;
                    }
                    Err(e) => {
                        self.app.error = Some(e);
                        self.app.loading = false;
                    }
                }
                self.data_receiver = None;
            }
        }
        
        self.app.update(ctx, frame);
    }
}

