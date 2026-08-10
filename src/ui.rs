use crate::types::*;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

fn world_to_screen(world_pos: Vec2, zoom: f32, pan: Vec2) -> Vec2 {
    let screen_center = vec2(screen_width() * 0.5, screen_height() * 0.5);
    (world_pos * zoom) + screen_center + pan
}

fn screen_to_world(screen_pos: Vec2, zoom: f32, pan: Vec2) -> Vec2 {
    let screen_center = vec2(screen_width() * 0.5, screen_height() * 0.5);
    (screen_pos - screen_center - pan) / zoom
}

fn point_line_dist(p: Vec2, v: Vec2, w: Vec2) -> f32 {
    let l2 = v.distance_squared(w);
    if l2 == 0.0 {
        return p.distance(v);
    }
    let t = ((p.x - v.x) * (w.x - v.x) + (p.y - v.y) * (w.y - v.y)) / l2;
    let t = t.clamp(0.0, 1.0);
    let proj = vec2(v.x + t * (w.x - v.x), v.y + t * (w.y - v.y));
    p.distance(proj)
}

fn draw_arrow(p1: Vec2, p2: Vec2, color: Color, thickness: f32) {
    draw_line(p1.x, p1.y, p2.x, p2.y, thickness, color);
    let dir = p2 - p1;
    let len = dir.length();
    if len > 0.0 {
        let n = dir / len;
        let perp = vec2(-n.y, n.x);
        let head_len = 10.0;
        let head_width = 4.0;
        draw_triangle(
            p2,
            p2 - n * head_len + perp * head_width,
            p2 - n * head_len - perp * head_width,
            color,
        );
    }
}

pub fn handle_input(state: &mut AppState) {
    let (mx, my) = mouse_position();
    let mouse_pos = vec2(mx, my);
    let ui_x = screen_width() - 250.0;

    let is_mouse_over_side_ui = (state.current_mode == AppMode::SetSupports
        || state.current_mode == AppMode::SetProperties
        || state.current_mode == AppMode::SetLoads)
        && mx > ui_x
        && my < 250.0;

    let wheel = mouse_wheel().1;
    if wheel != 0.0 && !is_mouse_over_side_ui && my > 40.0 {
        let mouse_world_before = screen_to_world(mouse_pos, state.cam_zoom, state.cam_pan);
        let zoom_factor = if wheel > 0.0 { 1.15 } else { 1.0 / 1.15 };
        state.cam_zoom = (state.cam_zoom * zoom_factor).clamp(0.1, 10.0);

        let screen_center = vec2(screen_width() * 0.5, screen_height() * 0.5);
        state.cam_pan = mouse_pos - screen_center - (mouse_world_before * state.cam_zoom);
    }

    let mouse_pos = vec2(mx, my);
    if is_mouse_button_down(MouseButton::Middle)
        || (is_mouse_button_down(MouseButton::Left) && is_key_down(KeyCode::Space))
    {
        let delta = mouse_pos - state.last_mouse_pos;
        state.cam_pan += delta;
    }
    state.last_mouse_pos = mouse_pos;

    if is_mouse_button_pressed(MouseButton::Left) {
        if my < 40.0 || is_mouse_over_side_ui {
            state.ui_focus = true;
        } else {
            state.ui_focus = false;
        }
    }

    if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
        state.ui_focus = false;
        state.selected_node_id = None;
        state.selected_element_id = None;
    }

    if is_mouse_button_pressed(MouseButton::Left) && my < 40.0 {
        let tab_w = screen_width() / 6.0;
        if mx < tab_w {
            state.current_mode = AppMode::PlaceNodes;
        } else if mx < tab_w * 2.0 {
            state.current_mode = AppMode::ConnectBeams;
        } else if mx < tab_w * 3.0 {
            state.current_mode = AppMode::SetSupports;
        } else if mx < tab_w * 4.0 {
            state.current_mode = AppMode::SetProperties;
            state.input_e = state.default_e.to_string();
            state.input_a = state.default_a.to_string();
            state.input_i = state.default_i.to_string();
        } else if mx < tab_w * 5.0 {
            state.current_mode = AppMode::SetLoads;
        } else {
            state.should_solve = true;
        }

        state.selected_node_id = None;
        state.selected_element_id = None;
        return;
    }

    if !state.ui_focus {
        if is_key_pressed(KeyCode::Key1) {
            state.current_mode = AppMode::PlaceNodes;
            state.selected_node_id = None;
            state.selected_element_id = None;
        }
        if is_key_pressed(KeyCode::Key2) {
            state.current_mode = AppMode::ConnectBeams;
            state.selected_node_id = None;
            state.selected_element_id = None;
        }
        if is_key_pressed(KeyCode::Key3) {
            state.current_mode = AppMode::SetSupports;
            state.selected_node_id = None;
            state.selected_element_id = None;
        }
        if is_key_pressed(KeyCode::Key4) {
            state.current_mode = AppMode::SetProperties;
            state.selected_element_id = None;
            state.input_e = state.default_e.to_string();
            state.input_a = state.default_a.to_string();
            state.input_i = state.default_i.to_string();
        }
        if is_key_pressed(KeyCode::Key5) {
            state.current_mode = AppMode::SetLoads;
            state.selected_node_id = None;
            state.selected_element_id = None;
        }
    }

    let world_mouse = screen_to_world(mouse_pos, state.cam_zoom, state.cam_pan);
    let grid_size = 50.0;
    let snap_world = vec2(
        (world_mouse.x / grid_size).round() * grid_size,
        (world_mouse.y / grid_size).round() * grid_size,
    );

    if is_mouse_over_side_ui {
        if is_mouse_button_pressed(MouseButton::Left) && state.current_mode == AppMode::SetSupports
        {
            for i in 0..4 {
                let btn_y = 105.0 + (i as f32 * 25.0);
                if my > btn_y - 15.0 && my < btn_y + 5.0 {
                    state.selected_support = match i {
                        0 => SupportType::Pin,
                        1 => SupportType::RollerH,
                        2 => SupportType::RollerV,
                        _ => SupportType::Fixed,
                    };
                }
            }
        }
    } else {
        let hovered_idx = state.nodes.iter().position(|n| {
            world_to_screen(n.pos, state.cam_zoom, state.cam_pan).distance(mouse_pos) < 15.0
        });

        if is_mouse_button_pressed(MouseButton::Left) {
            match state.current_mode {
                AppMode::PlaceNodes => {
                    if !state.nodes.iter().any(|n| n.pos.distance(snap_world) < 1.0) && my > 40.0 {
                        state.nodes.push(Node {
                            id: state.next_node_id,
                            pos: snap_world,
                            support: SupportType::Free,
                            fx: 0.0,
                            fy: 0.0,
                            m: 0.0,
                        });
                        state.next_node_id += 1;
                    }
                }
                AppMode::ConnectBeams => {
                    if let Some(idx) = hovered_idx {
                        let clicked_id = state.nodes[idx].id;
                        if let Some(first_id) = state.selected_node_id {
                            if first_id != clicked_id
                                && !state.elements.iter().any(|e| {
                                    (e.n1_id == first_id && e.n2_id == clicked_id)
                                        || (e.n1_id == clicked_id && e.n2_id == first_id)
                                })
                            {
                                state.elements.push(Element {
                                    id: state.next_element_id,
                                    n1_id: first_id,
                                    n2_id: clicked_id,
                                    e: state.default_e,
                                    a: state.default_a,
                                    i: state.default_i,
                                    w: 0.0,
                                });
                                state.next_element_id += 1;
                                state.selected_node_id = None;
                            }
                        } else {
                            state.selected_node_id = Some(clicked_id);
                        }
                    }
                }
                AppMode::SetSupports => {
                    if let Some(idx) = hovered_idx {
                        state.nodes[idx].support = state.selected_support;
                    }
                }
                AppMode::SetProperties => {
                    let mut closest = None;
                    let mut min_dist = 15.0;
                    for el in state.elements.iter() {
                        if let (Some(n1), Some(n2)) = (
                            state.nodes.iter().find(|n| n.id == el.n1_id),
                            state.nodes.iter().find(|n| n.id == el.n2_id),
                        ) {
                            let d = point_line_dist(
                                mouse_pos,
                                world_to_screen(n1.pos, state.cam_zoom, state.cam_pan),
                                world_to_screen(n2.pos, state.cam_zoom, state.cam_pan),
                            );
                            if d < min_dist {
                                min_dist = d;
                                closest = Some(el.id);
                            }
                        }
                    }
                    if let Some(id) = closest {
                        state.selected_element_id = Some(id);
                        let el = state.elements.iter().find(|e| e.id == id).unwrap();
                        state.input_e = el.e.to_string();
                        state.input_a = el.a.to_string();
                        state.input_i = el.i.to_string();
                    } else {
                        state.selected_element_id = None;
                    }
                }
                AppMode::SetLoads => {
                    if let Some(idx) = hovered_idx {
                        state.selected_node_id = Some(state.nodes[idx].id);
                        state.selected_element_id = None;
                        state.input_fx = state.nodes[idx].fx.to_string();
                        state.input_fy = state.nodes[idx].fy.to_string();
                        state.input_m = state.nodes[idx].m.to_string();
                    } else {
                        let mut closest = None;
                        let mut min_dist = 15.0;
                        for el in state.elements.iter() {
                            if let (Some(n1), Some(n2)) = (
                                state.nodes.iter().find(|n| n.id == el.n1_id),
                                state.nodes.iter().find(|n| n.id == el.n2_id),
                            ) {
                                let d = point_line_dist(
                                    mouse_pos,
                                    world_to_screen(n1.pos, state.cam_zoom, state.cam_pan),
                                    world_to_screen(n2.pos, state.cam_zoom, state.cam_pan),
                                );
                                if d < min_dist {
                                    min_dist = d;
                                    closest = Some(el.id);
                                }
                            }
                        }
                        if let Some(id) = closest {
                            state.selected_element_id = Some(id);
                            state.selected_node_id = None;
                            let el = state.elements.iter().find(|e| e.id == id).unwrap();
                            state.input_w = el.w.to_string();
                        } else {
                            state.selected_node_id = None;
                            state.selected_element_id = None;
                        }
                    }
                }
            }
        }

        if is_mouse_button_pressed(MouseButton::Right) {
            match state.current_mode {
                AppMode::PlaceNodes => {
                    if let Some(idx) = hovered_idx {
                        let del_id = state.nodes[idx].id;
                        state.nodes.remove(idx);
                        state
                            .elements
                            .retain(|e| e.n1_id != del_id && e.n2_id != del_id);
                    }
                }
                AppMode::ConnectBeams => {
                    let mut closest = None;
                    let mut min_dist = 15.0;
                    for (i, el) in state.elements.iter().enumerate() {
                        if let (Some(n1), Some(n2)) = (
                            state.nodes.iter().find(|n| n.id == el.n1_id),
                            state.nodes.iter().find(|n| n.id == el.n2_id),
                        ) {
                            let d = point_line_dist(
                                mouse_pos,
                                world_to_screen(n1.pos, state.cam_zoom, state.cam_pan),
                                world_to_screen(n2.pos, state.cam_zoom, state.cam_pan),
                            );
                            if d < min_dist {
                                min_dist = d;
                                closest = Some(i);
                            }
                        }
                    }
                    if let Some(idx) = closest {
                        state.elements.remove(idx);
                    } else {
                        state.selected_node_id = None;
                    }
                }
                AppMode::SetSupports => {
                    if let Some(idx) = hovered_idx {
                        state.nodes[idx].support = SupportType::Free;
                    }
                }
                AppMode::SetProperties => {
                    state.selected_element_id = None;
                }
                AppMode::SetLoads => {
                    state.selected_element_id = None;
                    state.selected_node_id = None;
                }
            }
        }
    }
}

pub fn render(state: &mut AppState, fe: &crate::fe_manager::FeManager) {
    clear_background(WHITE);
    let (mx, my) = mouse_position();
    let mouse_pos = vec2(mx, my);
    let ui_x = screen_width() - 250.0;
    let is_mouse_over_side_ui = (state.current_mode == AppMode::SetSupports
        || state.current_mode == AppMode::SetProperties
        || state.current_mode == AppMode::SetLoads)
        && mx > ui_x
        && my < 250.0;

    let grid_size = 50.0;
    let top_left_world = screen_to_world(vec2(0.0, 0.0), state.cam_zoom, state.cam_pan);
    let bottom_right_world = screen_to_world(
        vec2(screen_width(), screen_height()),
        state.cam_zoom,
        state.cam_pan,
    );

    let start_x = (top_left_world.x / grid_size).floor() * grid_size;
    let end_x = (bottom_right_world.x / grid_size).ceil() * grid_size;
    let start_y = (top_left_world.y / grid_size).floor() * grid_size;
    let end_y = (bottom_right_world.y / grid_size).ceil() * grid_size;

    let mut x = start_x;
    while x <= end_x {
        let p1 = world_to_screen(vec2(x, top_left_world.y), state.cam_zoom, state.cam_pan);
        let p2 = world_to_screen(vec2(x, bottom_right_world.y), state.cam_zoom, state.cam_pan);
        draw_line(p1.x, p1.y, p2.x, p2.y, 1.0, Color::new(0.9, 0.9, 0.9, 1.0));
        x += grid_size;
    }

    let mut y = start_y;
    while y <= end_y {
        let p1 = world_to_screen(vec2(top_left_world.x, y), state.cam_zoom, state.cam_pan);
        let p2 = world_to_screen(vec2(bottom_right_world.x, y), state.cam_zoom, state.cam_pan);
        draw_line(p1.x, p1.y, p2.x, p2.y, 1.0, Color::new(0.9, 0.9, 0.9, 1.0));
        y += grid_size;
    }

    let mut hovered_el_id = None;
    if (state.current_mode == AppMode::SetProperties || state.current_mode == AppMode::SetLoads)
        && !is_mouse_over_side_ui
        && my > 40.0
    {
        let mut min_dist = 15.0;
        for el in &state.elements {
            if let (Some(n1), Some(n2)) = (
                state.nodes.iter().find(|n| n.id == el.n1_id),
                state.nodes.iter().find(|n| n.id == el.n2_id),
            ) {
                let d = point_line_dist(
                    mouse_pos,
                    world_to_screen(n1.pos, state.cam_zoom, state.cam_pan),
                    world_to_screen(n2.pos, state.cam_zoom, state.cam_pan),
                );
                if d < min_dist {
                    min_dist = d;
                    hovered_el_id = Some(el.id);
                }
            }
        }
    }

    let has_solution =
        fe.displacements.len() > 0 && fe.displacements.len() == state.nodes.len() * 3;

    let mut max_node_disp = 0.0f32;
    if has_solution {
        for k in 0..state.nodes.len() {
            let dx = fe.displacements[k * 3] as f32;
            let dy = fe.displacements[k * 3 + 1] as f32;
            let mag = (dx * dx + dy * dy).sqrt();
            if mag > max_node_disp {
                max_node_disp = mag;
            }
        }
    }

    let mut max_span = 100.0f32;
    if !state.nodes.is_empty() {
        let mut min_p = state.nodes[0].pos;
        let mut max_p = state.nodes[0].pos;
        for n in &state.nodes {
            min_p.x = min_p.x.min(n.pos.x);
            min_p.y = min_p.y.min(n.pos.y);
            max_p.x = max_p.x.max(n.pos.x);
            max_p.y = max_p.y.max(n.pos.y);
        }
        max_span = (max_p - min_p).length().max(50.0);
    }

    let global_auto_factor = if max_node_disp > 1e-12 {
        (max_span * 0.1) / max_node_disp
    } else {
        1.0
    };
    let global_def_scale = state.def_scale * global_auto_factor;

    for el in &state.elements {
        if let (Some(i1), Some(i2)) = (
            state.nodes.iter().position(|n| n.id == el.n1_id),
            state.nodes.iter().position(|n| n.id == el.n2_id),
        ) {
            let n1 = &state.nodes[i1];
            let n2 = &state.nodes[i2];
            let sp1 = world_to_screen(n1.pos, state.cam_zoom, state.cam_pan);
            let sp2 = world_to_screen(n2.pos, state.cam_zoom, state.cam_pan);

            let is_selected = (state.current_mode == AppMode::SetProperties
                || state.current_mode == AppMode::SetLoads)
                && state.selected_element_id == Some(el.id);
            let is_hovered = hovered_el_id == Some(el.id);
            let base_color = if is_selected {
                RED
            } else if is_hovered {
                GREEN
            } else {
                BLACK
            };

            let initial_thickness = if has_solution {
                1.5
            } else {
                if is_selected || is_hovered { 6.0 } else { 4.0 }
            };
            draw_line(
                sp1.x,
                sp1.y,
                sp2.x,
                sp2.y,
                initial_thickness * state.cam_zoom.sqrt(),
                base_color,
            );

            if has_solution {
                let u1_x = fe.displacements[i1 * 3] as f32;
                let u1_y = fe.displacements[i1 * 3 + 1] as f32;
                let r1 = -1.0 * (fe.displacements[i1 * 3 + 2] as f32);

                let u2_x = fe.displacements[i2 * 3] as f32;
                let u2_y = fe.displacements[i2 * 3 + 1] as f32;
                let r2 = -1.0 * (fe.displacements[i2 * 3 + 2] as f32);

                let ds1 = vec2(u1_x, -u1_y);
                let ds2 = vec2(u2_x, -u2_y);

                let p1 = n1.pos;
                let p2 = n2.pos;
                let dir_world = p2 - p1;
                let l_world = dir_world.length();

                if l_world > 0.0 {
                    let n_w = dir_world / l_world;
                    let perp_w = vec2(n_w.y, -n_w.x);

                    let u1_axial = ds1.dot(n_w);
                    let v1_trans = ds1.dot(perp_w);
                    let u2_axial = ds2.dot(n_w);
                    let v2_trans = ds2.dot(perp_w);

                    let ndata = 20;
                    let mut prev_screen_pt = world_to_screen(p1, state.cam_zoom, state.cam_pan);
                    let def_scale = global_def_scale;

                    for j in 0..ndata {
                        let xi = j as f32 / (ndata - 1) as f32;

                        let u_xi = (1.0 - xi) * u1_axial + xi * u2_axial;

                        let xi2 = xi * xi;
                        let xi3 = xi2 * xi;
                        let h1 = 1.0 - 3.0 * xi2 + 2.0 * xi3;
                        let h2 = l_world * (xi - 2.0 * xi2 + xi3);
                        let h3 = 3.0 * xi2 - 2.0 * xi3;
                        let h4 = l_world * (-xi2 + xi3);
                        let v_xi = h1 * v1_trans + h2 * r1 + h3 * v2_trans + h4 * r2;

                        let base_world_pt = p1.lerp(p2, xi);
                        let deflection_world = (n_w * u_xi + perp_w * v_xi) * def_scale;
                        let curr_world_pt = base_world_pt + deflection_world;
                        let curr_screen_pt =
                            world_to_screen(curr_world_pt, state.cam_zoom, state.cam_pan);

                        if j > 0 {
                            draw_line(
                                prev_screen_pt.x,
                                prev_screen_pt.y,
                                curr_screen_pt.x,
                                curr_screen_pt.y,
                                3.5 * state.cam_zoom.sqrt(),
                                BLUE,
                            );
                        }
                        prev_screen_pt = curr_screen_pt;
                    }
                }
            }

            if el.w != 0.0 {
                let p1 = sp1;
                let p2 = sp2;
                let dir = (p2 - p1).normalize();
                let perp = vec2(dir.y, -dir.x);
                let load_dir = if el.w > 0.0 { perp } else { -perp };
                let offset = 30.0 * state.cam_zoom;

                let top_p1 = p1 - load_dir * offset;
                let top_p2 = p2 - load_dir * offset;
                draw_line(
                    top_p1.x,
                    top_p1.y,
                    top_p2.x,
                    top_p2.y,
                    1.5 * state.cam_zoom.sqrt(),
                    BLUE,
                );

                let len = p1.distance(p2);
                let num_arrows = ((len / (30.0 * state.cam_zoom)) as i32).max(2);
                for j in 0..=num_arrows {
                    let t = j as f32 / num_arrows as f32;
                    let base_pt = p1.lerp(p2, t);
                    let top_pt = base_pt - load_dir * offset;
                    draw_arrow(
                        top_pt,
                        base_pt - load_dir * 4.0,
                        BLUE,
                        1.5 * state.cam_zoom.sqrt(),
                    );
                }

                let mid = p1.lerp(p2, 0.5) - load_dir * (offset + 15.0);
                draw_text(
                    &format!("w = {} N/m", el.w),
                    mid.x - 35.0,
                    mid.y,
                    16.0,
                    BLUE,
                );
            }
        }
    }

    if state.current_mode == AppMode::ConnectBeams {
        if let Some(first_id) = state.selected_node_id {
            if let Some(n1) = state.nodes.iter().find(|n| n.id == first_id) {
                let sp1 = world_to_screen(n1.pos, state.cam_zoom, state.cam_pan);
                draw_line(sp1.x, sp1.y, mouse_pos.x, mouse_pos.y, 2.0, GRAY);
            }
        }
    }

    let world_mouse = screen_to_world(mouse_pos, state.cam_zoom, state.cam_pan);
    let snap_world = vec2(
        (world_mouse.x / grid_size).round() * grid_size,
        (world_mouse.y / grid_size).round() * grid_size,
    );

    if state.current_mode == AppMode::PlaceNodes && !is_mouse_over_side_ui && my > 40.0 {
        if !state.nodes.iter().any(|n| n.pos.distance(snap_world) < 1.0) {
            let sp = world_to_screen(snap_world, state.cam_zoom, state.cam_pan);
            let r = 6.0 * state.cam_zoom;
            draw_circle(sp.x, sp.y, r, Color::new(0.0, 0.0, 1.0, 0.3));
            draw_circle_lines(sp.x, sp.y, r * 1.8, 1.5, Color::new(0.0, 1.0, 0.0, 0.5));
        }
    }

    let hovered_idx = state.nodes.iter().position(|n| {
        world_to_screen(n.pos, state.cam_zoom, state.cam_pan).distance(mouse_pos) < 15.0
    });

    for (i, node) in state.nodes.iter().enumerate() {
        let sp = world_to_screen(node.pos, state.cam_zoom, state.cam_pan);
        let r = 6.0 * state.cam_zoom;

        if node.fx != 0.0 {
            let (start, end) = if node.fx > 0.0 {
                (vec2(sp.x - 50.0, sp.y), vec2(sp.x - r, sp.y))
            } else {
                (vec2(sp.x + 50.0, sp.y), vec2(sp.x + r, sp.y))
            };
            draw_arrow(start, end, RED, 2.5);
            draw_text(
                &format!("{} N", node.fx),
                sp.x - 25.0,
                sp.y - 15.0,
                14.0,
                RED,
            );
        }

        if node.fy != 0.0 {
            let (start, end) = if node.fy > 0.0 {
                (vec2(sp.x, sp.y + 50.0), vec2(sp.x, sp.y + r))
            } else {
                (vec2(sp.x, sp.y - 50.0), vec2(sp.x, sp.y - r))
            };
            draw_arrow(start, end, RED, 2.5);
            draw_text(
                &format!("{} N", node.fy),
                sp.x + 10.0,
                sp.y + 5.0,
                14.0,
                RED,
            );
        }

        if node.m != 0.0 {
            let m_r = 24.0 * state.cam_zoom;
            let start_a = std::f32::consts::PI / 4.0;
            let end_a = 7.0 * std::f32::consts::PI / 4.0;
            for j in 0..16 {
                let t1 = start_a + (end_a - start_a) * (j as f32 / 16.0);
                let t2 = start_a + (end_a - start_a) * ((j + 1) as f32 / 16.0);
                draw_line(
                    sp.x + t1.cos() * m_r,
                    sp.y - t1.sin() * m_r,
                    sp.x + t2.cos() * m_r,
                    sp.y - t2.sin() * m_r,
                    2.5,
                    PURPLE,
                );
            }
            let tip = vec2(sp.x + end_a.cos() * m_r, sp.y - end_a.sin() * m_r);
            draw_arrow(tip + vec2(2.0, -4.0), tip, PURPLE, 2.5);
            draw_text(
                &format!("{} Nm", node.m),
                sp.x - 15.0,
                sp.y - (m_r + 6.0),
                14.0,
                PURPLE,
            );
        }

        let z = state.cam_zoom;
        match node.support {
            SupportType::Free => {}
            SupportType::Pin => {
                let r = 14.0 * z;
                let gap = 6.0 * z;
                draw_poly(sp.x, sp.y + gap + r, 3, r, -90.0, RED);
                let base_y = sp.y + gap + r + (r * 0.5);
                draw_line(
                    sp.x - 18.0 * z,
                    base_y,
                    sp.x + 18.0 * z,
                    base_y,
                    2.0 * z.sqrt(),
                    RED,
                );
            }
            SupportType::RollerH => {
                let r = 14.0 * z;
                let gap = 6.0 * z;
                draw_poly(sp.x, sp.y + gap + r, 3, r, -90.0, ORANGE);
                let base_y = sp.y + gap + r + (r * 0.5);
                let wheel_r = 3.5 * z;
                let wheel_cy = base_y + wheel_r;
                draw_circle_lines(sp.x - 8.0 * z, wheel_cy, wheel_r, 1.5 * z.sqrt(), ORANGE);
                draw_circle_lines(sp.x + 8.0 * z, wheel_cy, wheel_r, 1.5 * z.sqrt(), ORANGE);
                let floor_y = wheel_cy + wheel_r + 2.0 * z;
                draw_line(
                    sp.x - 22.0 * z,
                    floor_y,
                    sp.x + 22.0 * z,
                    floor_y,
                    2.0 * z.sqrt(),
                    ORANGE,
                );
            }
            SupportType::RollerV => {
                let r = 14.0 * z;
                let gap = 6.0 * z;
                draw_poly(sp.x - (gap + r), sp.y, 3, r, 0.0, ORANGE);
                let base_x = sp.x - (gap + r + (r * 0.5));
                let wheel_r = 3.5 * z;
                let wheel_cx = base_x - wheel_r;
                draw_circle_lines(wheel_cx, sp.y - 8.0 * z, wheel_r, 1.5 * z.sqrt(), ORANGE);
                draw_circle_lines(wheel_cx, sp.y + 8.0 * z, wheel_r, 1.5 * z.sqrt(), ORANGE);
                let wall_x = wheel_cx - wheel_r - 2.0 * z;
                draw_line(
                    wall_x,
                    sp.y - 22.0 * z,
                    wall_x,
                    sp.y + 22.0 * z,
                    2.0 * z.sqrt(),
                    ORANGE,
                );
            }
            SupportType::Fixed => {
                let gap = 6.0 * z;
                let base_y = sp.y + gap;
                let w = 24.0 * z;
                draw_line(sp.x - w, base_y, sp.x + w, base_y, 2.5 * z.sqrt(), MAGENTA);
                for j in 0..=6 {
                    let x_offset = -w + (j as f32 * (2.0 * w / 6.0));
                    draw_line(
                        sp.x + x_offset,
                        base_y,
                        sp.x + x_offset - 8.0 * z,
                        base_y + 12.0 * z,
                        1.5 * z.sqrt(),
                        MAGENTA,
                    );
                }
            }
        }

        draw_circle(sp.x, sp.y, r, BLUE);
        draw_circle(sp.x, sp.y, r * 0.4, WHITE);

        if hovered_idx == Some(i) && my > 40.0 {
            draw_circle_lines(sp.x, sp.y, r * 2.0, 2.0, GREEN);
        }
    }

    if !fe.solver_status.is_empty() {
        let is_error = fe.solver_status.starts_with("ERROR");
        let msg = if is_error {
            fe.solver_status.clone()
        } else {
            format!(
                "SOLVER SUCCESS [Deformation Scale: {:.1}x]",
                state.def_scale
            )
        };
        let color = if is_error { RED } else { DARKGREEN };
        draw_text(&msg, 20.0, screen_height() - 20.0, 20.0, color);
    }

    let tab_w = screen_width() / 6.0;
    let tabs = [
        "[1] NODES",
        "[2] BEAMS",
        "[3] SUPPORTS",
        "[4] PROPS",
        "[5] LOADS",
        "[ENT] SOLVE",
    ];

    for (i, label) in tabs.iter().enumerate() {
        let x = i as f32 * tab_w;
        let is_active = match state.current_mode {
            AppMode::PlaceNodes if i == 0 => true,
            AppMode::ConnectBeams if i == 1 => true,
            AppMode::SetSupports if i == 2 => true,
            AppMode::SetProperties if i == 3 => true,
            AppMode::SetLoads if i == 4 => true,
            _ => false,
        };

        if i == 5 {
            if my < 40.0 && mx > x && mx < x + tab_w {
                draw_rectangle(x, 0.0, tab_w, 40.0, Color::new(0.6, 1.0, 0.6, 1.0));
            } else {
                draw_rectangle(x, 0.0, tab_w, 40.0, Color::new(0.8, 1.0, 0.8, 1.0));
            }
        } else if is_active || (my < 40.0 && mx > x && mx < x + tab_w) {
            draw_rectangle(x, 0.0, tab_w, 40.0, Color::new(0.85, 0.85, 0.85, 1.0));
        } else {
            draw_rectangle(x, 0.0, tab_w, 40.0, Color::new(0.95, 0.95, 0.95, 1.0));
        }

        draw_rectangle_lines(x, 0.0, tab_w, 40.0, 1.0, GRAY);
        let text_size = measure_text(label, None, 20, 1.0);
        draw_text(
            label,
            x + (tab_w - text_size.width) / 2.0,
            25.0,
            20.0,
            BLACK,
        );
    }

    if state.current_mode == AppMode::SetSupports {
        draw_rectangle(ui_x, 60.0, 230.0, 150.0, WHITE);
        draw_rectangle_lines(ui_x, 60.0, 230.0, 150.0, 2.0, BLACK);
        draw_text("Select Support:", ui_x + 15.0, 85.0, 20.0, BLACK);
        let options = [
            (SupportType::Pin, "Pin (Fixed X/Y)"),
            (SupportType::RollerH, "Roller (Fixed Y)"),
            (SupportType::RollerV, "Roller (Fixed X)"),
            (SupportType::Fixed, "Clamped (Fixed All)"),
        ];
        for (j, (s_type, label)) in options.iter().enumerate() {
            let btn_y = 105.0 + (j as f32 * 25.0);
            if mx > ui_x + 10.0 && mx < ui_x + 220.0 && my > btn_y - 15.0 && my < btn_y + 5.0 {
                draw_rectangle(
                    ui_x + 10.0,
                    btn_y - 15.0,
                    210.0,
                    20.0,
                    Color::new(0.9, 0.9, 0.9, 1.0),
                );
            }
            let prefix = if state.selected_support == *s_type {
                "[X]"
            } else {
                "[  ]"
            };
            draw_text(
                &format!("{} {}", prefix, label),
                ui_x + 15.0,
                btn_y,
                20.0,
                BLACK,
            );
        }
    }

    if state.current_mode == AppMode::SetProperties {
        let title = if state.selected_element_id.is_some() {
            "Element Properties"
        } else {
            "Global Defaults & View"
        };
        widgets::Window::new(hash!(), vec2(ui_x, 60.0), vec2(230.0, 230.0))
            .label(title)
            .ui(&mut *root_ui(), |ui| {
                ui.input_text(hash!(), "E (Pa)", &mut state.input_e);
                ui.input_text(hash!(), "A (m^2)", &mut state.input_a);
                ui.input_text(hash!(), "I (m^4)", &mut state.input_i);
                ui.input_text(hash!(), "Deflection Scale", &mut state.input_scale);

                if ui.button(None, "Apply Changes") {
                    let parsed_e = state.input_e.parse::<f64>().unwrap_or(state.default_e);
                    let parsed_a = state.input_a.parse::<f64>().unwrap_or(state.default_a);
                    let parsed_i = state.input_i.parse::<f64>().unwrap_or(state.default_i);

                    state.def_scale = state.input_scale.parse::<f32>().unwrap_or(state.def_scale);

                    if let Some(id) = state.selected_element_id {
                        if let Some(el) = state.elements.iter_mut().find(|e| e.id == id) {
                            el.e = parsed_e;
                            el.a = parsed_a;
                            el.i = parsed_i;
                        }
                    } else {
                        state.default_e = parsed_e;
                        state.default_a = parsed_a;
                        state.default_i = parsed_i;
                    }
                }
            });
    }

    if state.current_mode == AppMode::SetLoads {
        if state.selected_node_id.is_some() {
            widgets::Window::new(hash!(), vec2(ui_x, 60.0), vec2(230.0, 180.0))
                .label("Node Loads")
                .ui(&mut *root_ui(), |ui| {
                    ui.input_text(hash!(), "Fx (N)", &mut state.input_fx);
                    ui.input_text(hash!(), "Fy (N) [+ is UP]", &mut state.input_fy);
                    ui.input_text(hash!(), "Moment", &mut state.input_m);

                    if ui.button(None, "Apply Load") {
                        if let Some(id) = state.selected_node_id {
                            if let Some(node) = state.nodes.iter_mut().find(|n| n.id == id) {
                                node.fx = state.input_fx.parse::<f64>().unwrap_or(0.0);
                                node.fy = state.input_fy.parse::<f64>().unwrap_or(0.0);
                                node.m = state.input_m.parse::<f64>().unwrap_or(0.0);
                            }
                        }
                    }
                });
        } else if state.selected_element_id.is_some() {
            widgets::Window::new(hash!(), vec2(ui_x, 60.0), vec2(230.0, 100.0))
                .label("Beam Uniform Load")
                .ui(&mut *root_ui(), |ui| {
                    ui.input_text(hash!(), "w (N/m)", &mut state.input_w);
                    if ui.button(None, "Apply Distributed Load") {
                        if let Some(id) = state.selected_element_id {
                            if let Some(el) = state.elements.iter_mut().find(|e| e.id == id) {
                                el.w = state.input_w.parse::<f64>().unwrap_or(0.0);
                            }
                        }
                    }
                });
        }
    }
}
