use macroquad::prelude::Vec2;

#[derive(PartialEq, Clone, Copy)]
pub enum AppMode {
    PlaceNodes,
    ConnectBeams,
    SetSupports,
    SetProperties,
    SetLoads,
}

#[derive(PartialEq, Clone, Copy)]
pub enum SupportType {
    Free,
    Pin,
    RollerH,
    RollerV,
    Fixed,
}

#[derive(Clone)]
pub struct Node {
    pub id: usize,
    pub pos: Vec2,
    pub support: SupportType,
    pub fx: f64,
    pub fy: f64,
    pub m: f64,
}

#[derive(Clone)]
pub struct Element {
    pub id: usize,
    pub n1_id: usize,
    pub n2_id: usize,
    pub e: f64,
    pub a: f64,
    pub i: f64,
    pub w: f64,
}

pub struct AppState {
    pub nodes: Vec<Node>,
    pub elements: Vec<Element>,
    pub next_node_id: usize,
    pub next_element_id: usize,
    pub current_mode: AppMode,
    pub selected_node_id: Option<usize>,
    pub selected_support: SupportType,
    pub selected_element_id: Option<usize>,

    pub default_e: f64,
    pub default_a: f64,
    pub default_i: f64,
    pub input_e: String,
    pub input_a: String,
    pub input_i: String,
    pub input_fx: String,
    pub input_fy: String,
    pub input_m: String,
    pub input_w: String,

    pub ui_focus: bool,
    pub should_solve: bool,

    pub cam_zoom: f32,
    pub cam_pan: Vec2,
    pub last_mouse_pos: Vec2,

    pub def_scale: f32,
    pub input_scale: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            elements: Vec::new(),
            next_node_id: 1,
            next_element_id: 1,
            current_mode: AppMode::PlaceNodes,
            selected_node_id: None,
            selected_support: SupportType::Pin,
            selected_element_id: None,
            default_e: 200e9,
            default_a: 0.01,
            default_i: 1e-5,

            input_e: "200000000000".to_string(),
            input_a: "0.01".to_string(),
            input_i: "0.00001".to_string(),
            input_fx: "0.0".to_string(),
            input_fy: "0.0".to_string(),
            input_m: "0.0".to_string(),
            input_w: "0.0".to_string(),

            ui_focus: false,
            should_solve: false,

            cam_zoom: 1.0,
            cam_pan: Vec2::ZERO,
            last_mouse_pos: Vec2::ZERO,

            def_scale: 100.0,
            input_scale: "100.0".to_string(),
        }
    }
}
