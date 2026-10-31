//! The adjacency tool's Tauri app: commands and IPC types. Domain logic lives
//! in `maze-core` (DR-0012).

pub mod ipc;

use ipc::{ReloadResult, TableView, TableViewRequest, ToolError, ToolErrorKind};

/// Returns `maze-core`'s version, proving the frontend -> Tauri -> core path.
#[tauri::command]
fn core_version() -> String {
    maze_core::version().to_string()
}

/// Stub until the table is wired up: the shapes are final (DR-0024).
#[tauri::command]
fn table_view(request: TableViewRequest) -> Result<TableView, ToolError> {
    let _ = request;
    Err(ToolError {
        kind: ToolErrorKind::NotLoaded,
        messages: vec!["The table isn't built yet (stub).".into()],
    })
}

#[tauri::command]
fn reload_tile_set() -> Result<ReloadResult, ToolError> {
    Err(ToolError {
        kind: ToolErrorKind::LoadFailed,
        messages: vec!["Reload isn't built yet (stub).".into()],
    })
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![core_version, table_view, reload_tile_set])
        .run(tauri::generate_context!())
        .expect("error while running the Maze Maker tool");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_version_comes_from_maze_core() {
        assert_eq!(core_version(), maze_core::version());
    }
}
