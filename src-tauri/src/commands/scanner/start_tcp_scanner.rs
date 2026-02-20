use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::io::Interest;
use tokio::net::TcpStream;
use tokio::sync::Mutex as TokioMutex;
use crate::runners::mitmproxy_runner::{MitmproxyRunner, MitmproxyCommandEvent};
use crate::SharedConfig;
use crate::scanner_state::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonMessage {
    #[serde(default)]
    message: Option<String>,
    #[serde(flatten)]
    extra: serde_json::Value,
}

/// Start the TCP client scanner
#[tauri::command]
pub async fn start_tcp_scanner(
    app: AppHandle,
    state: State<'_, TcpScannerState>,
    config: State<'_, SharedConfig>,
) -> Result<String, String> {
    let mut connection = state.connection.lock().await;

    // Check if already running
    if connection.is_some() {
        return Err("Scanner already running".to_string());
    }

    // Get the mitm port from config
    let cfg_snapshot = config.load();
    let mitm_port = &cfg_snapshot.mitmproxy.mitm_port.to_string();
    let mitm_extra = &cfg_snapshot.mitmproxy.mitm_extra_args;
    
    // Create and spawn mitmweb process
    let mut args: Vec<&str> = mitm_extra.split(" ").collect::<Vec<_>>();
    args.push("-q");
    args.push("--listen-port");
    args.push( &mitm_port );
    
    let runner = MitmproxyRunner::new(&app)
        .with_args(args);

    let (mut rx, child) = runner.spawn()
        .map_err(|e| {
            eprintln!("[DEBUG] Failed to spawn mitmweb: {}", e);
            format!("Failed to spawn mitmweb: {}", e)
        })?;

    let mitmweb_child = Arc::new(TokioMutex::new(child));
    let shutdown_flag = Arc::new(TokioMutex::new(false));
 
    // Spawn task to read mitmweb events and display for debug
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                MitmproxyCommandEvent::Stdout(data) => {
                    if let Ok(msg) = String::from_utf8(data) {
                        eprintln!("[MITMWEB] {}", msg);
                    }
                }
                MitmproxyCommandEvent::Stderr(data) => {
                    if let Ok(msg) = String::from_utf8(data) {
                        eprintln!("[MITMWEB ERROR] {}", msg);
                    }
                }
                MitmproxyCommandEvent::Error(msg) => {
                    eprintln!("[MITMWEB PROCESS ERROR] {}", msg);
                    
                }
                MitmproxyCommandEvent::Terminated(payload) => {
                    eprintln!("[MITMWEB] Process terminated with code: {:?}", payload.code);
                }
            }
        }
    });
    let shutdown_flag_clone2 = shutdown_flag.clone();
    
    // Spawn the TCP client connection task
    tauri::async_runtime::spawn(async move {
        // Give mitmweb some time to start
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        match TcpStream::connect("127.0.0.1:12000").await {
            Ok(socket) => {
                eprintln!("[DEBUG] Connected to port 12000");
                let mut buffer = vec![0; 81920]; // 80KB buffer

                loop {
                    // Check shutdown flag
                    if *shutdown_flag_clone2.lock().await {
                        eprintln!("[DEBUG] Shutdown signal received");
                        break;
                    }

                    let readyread = socket.ready(Interest::READABLE).await.unwrap();
                    if readyread.is_readable() 
                    {
                        let _ = socket.readable().await.unwrap();
                        match socket.try_read(&mut buffer) {
                            Ok(0) => {
                                eprintln!("[DEBUG] Connection closed");
                                let _ = app_clone.emit("tcp_scanner", ScannerMessage {
                                    message: "Connection closed".to_string(),
                                });
                                break;
                            }
                            Ok(n) => {
                                if let Ok(message_str) = String::from_utf8(buffer[..n].to_vec()) {
                                    eprintln!("[TCP RECEIVED] {}", message_str.trim());
                                    
                                    // Parse complete JSON message
                                    match serde_json::from_str::<JsonMessage>(&message_str) {
                                        Ok(json_msg) => {
                                            let msg = json_msg.message.unwrap_or_else(|| {
                                                json_msg.extra.to_string()
                                            });
                                            let _ = app_clone.emit("tcp_scanner", ScannerMessage {
                                                message: msg,
                                            });
                                        }
                                        Err(e) => {
                                            eprintln!("[JSON PARSE ERROR] {}", e);
                                        }
                                    }
                                }
                            }
                            Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                                continue;
                            }
                            Err(e) => {
                                eprintln!("[TCP ERROR] {}", e);
                                let _ = app_clone.emit("tcp_scanner", ScannerMessage {
                                    message: format!("Error: {}", e),
                                });
                                break;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[DEBUG] Failed to connect to port 12000: {}", e);
                let _ = app_clone.emit("tcp_scanner", ScannerMessage {
                    message: format!("Failed to connect to port 12000: {}", e),
                });
            }
        }
    });

    *connection = Some(TcpStreamHandle {
        shutdown_flag: shutdown_flag,
        mitmweb_child: Some(mitmweb_child),
    });
 
    eprintln!("[DEBUG] network scanner started - mitmweb listening on port {}", mitm_port);
    Ok(format!("network scanner started - mitmweb listening on port {}", mitm_port))
}
