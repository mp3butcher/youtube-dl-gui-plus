
use tauri::State;
use crate::scanner_state::*;
 
/// Stop the TCP client scanner and mitmproxy
#[tauri::command]
pub async fn stop_tcp_scanner(
    state: State<'_, TcpScannerState>,
) -> Result<String, String> {
  
    let mut connection = state.connection.lock().await;

   
    if let Some(handle) = connection.take() {
        // Signal shutdown
        *handle.shutdown_flag.lock().await = true;

        eprintln!("[DEBUG] TCP scanner stop requested");

        // The mitmweb_child will be dropped here and cleaned up
        handle.mitmweb_child.clone().unwrap().lock().await.kill_tree();
        drop(handle.mitmweb_child);
        //  handle.mitmweb_child.lock().await.take(); // Drop the mitmweb child to clean up the process
   
        Ok("TCP scanner stopped".to_string())
    } else {
        Err("Scanner not running".to_string())
    }
}
