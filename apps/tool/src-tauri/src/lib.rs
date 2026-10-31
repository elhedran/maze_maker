//! The adjacency tool's Tauri app: commands and IPC types. Domain logic lives
//! in `maze-core` (DR-0012).

/// Returns `maze-core`'s version, proving the frontend -> Tauri -> core path.
#[tauri::command]
fn core_version() -> String {
    maze_core::version().to_string()
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![core_version])
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
