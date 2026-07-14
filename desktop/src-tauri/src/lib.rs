pub mod commands;
pub mod engine;
pub mod model;
pub mod queue;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_jobs,
            commands::ingest_paths,
            commands::set_settings,
            commands::clear_finished,
            commands::cancel_queued,
            commands::has_active_work,
            commands::model_ready,
            commands::download_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running StenoDrop");
}
