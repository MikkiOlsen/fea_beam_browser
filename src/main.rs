use macroquad::prelude::*;

mod assem;
mod fe_manager;
mod kbeam;
mod types;
mod ui;

#[macroquad::main("Statics Analyzer")]
async fn main() {
    let mut app_state = types::AppState::new();
    let mut fe_manager = fe_manager::FeManager::new();

    loop {
        ui::handle_input(&mut app_state);

        if app_state.should_solve {
            app_state.should_solve = false;
            fe_manager.build_and_solve(&app_state.nodes, &app_state.elements);
        }

        ui::render(&mut app_state, &fe_manager);

        next_frame().await;
    }
}
