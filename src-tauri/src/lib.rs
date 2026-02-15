mod file_system;
mod config_manager;
mod ai_client;
mod commands;
mod terminal;
mod database;
mod cache;
mod mcp;

use file_system::*;
use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Starting Karsa IDE...");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(TerminalState::new())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            read_file_content,
            write_file_content,
            list_directory,
            file_exists,
            create_directory,
            create_file,
            delete_path,
            rename_path,
            copy_path,
            get_ai_config,
            save_ai_config,
            check_config_exists,
            fetch_kilo_models,
            test_ai_connection,
            send_chat_completion,
            generate_chat_title,
            send_chat_completion_stream,
            get_session,
            save_session,
            create_conversation,
            get_conversations,
            add_message,
            get_messages,
            delete_conversation,
            update_conversation_title,
            search_conversations,
            toggle_terminal,
            toggle_chat,
            get_shell_info,
            spawn_terminal,
            write_to_terminal,
            mcp_execute,
            mcp_get_tools,
            mcp_get_system_prompt,
        ])
        .setup(|_app| {
            log::info!("Karsa IDE setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
