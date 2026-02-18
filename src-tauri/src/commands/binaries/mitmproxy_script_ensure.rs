use crate::binaries::binaries_manager::BinariesManager;
use crate::binaries::binaries_state::BinariesState;
use tauri::State;

#[tauri::command]
pub async fn mitmproxy_script_ensure(
  binaries_manager: State<'_, BinariesManager>,
  state: State<'_, BinariesState>  
) -> Result<(), String> {
  if !state.try_start() {
    return Ok(());
  }
  let res = binaries_manager.ensure_mitmproxy_script().await
  .map_err(|e| e.to_string());
  state.finish();
  res
}
