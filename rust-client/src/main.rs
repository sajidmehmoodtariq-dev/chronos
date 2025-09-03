use chrono::Local;
use rusqlite::{Connection, Result as SqlResult};
use rdev::{listen, Event};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::process::Command;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND};
use windows::Win32::System::Console::FreeConsole;
use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
};
use windows::Win32::UI::Input::KeyboardAndMouse::GetLastInputInfo;

// -------------------- logging --------------------

fn get_app_data_dir() -> PathBuf {
    let mut path = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    path.push("Chronos");
    std::fs::create_dir_all(&path).ok();
    path
}

fn log_line(line: &str) {
    let now = Local::now();
    let log_path = get_app_data_dir().join("activity_log.txt");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .unwrap();
    let _ = writeln!(file, "{} - {}", now.format("%Y-%m-%d %H:%M:%S"), line);
}

// -------------------- idle detection --------------------
// Return milliseconds since last user input (keyboard/mouse)
#[allow(non_snake_case)]
#[repr(C)]
struct LASTINPUTINFO {
    cbSize: u32,
    dwTime: u32,
}
fn idle_ms() -> u32 {
    unsafe {
        let mut li = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        let ok = GetLastInputInfo(std::mem::transmute(&mut li));
        if ok.as_bool() {
            let tick = windows::Win32::System::SystemInformation::GetTickCount();
            tick - li.dwTime
        } else {
            0
        }
    }
}

// -------------------- foreground window --------------------

fn active_window_title_and_process() -> Option<(String, String)> {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.is_invalid() {
            return None;
        }

        // Title
        let mut title_buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title_buf);
        let title = String::from_utf16_lossy(&title_buf[..len as usize]).trim().to_string();

        // PID -> exe name
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let exe = process_name_from_pid(pid).unwrap_or_else(|| "Unknown".to_string());

        Some((title, exe))
    }
}

// Get process exe name using Win32 APIs (no extra crate)
fn process_name_from_pid(pid: u32) -> Option<String> {
    unsafe {
        if pid == 0 { return None; }
        let handle_result = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            pid
        );
        
        let handle: HANDLE = match handle_result {
            Ok(h) => h,
            Err(_) => return None, // Can't access this process - skip it
        };

        // buffer for exe name
        let mut buf = [0u16; 260];
        let len = GetModuleBaseNameW(handle, None, &mut buf);
        let _ = CloseHandle(handle);
        if len > 0 {
            Some(String::from_utf16_lossy(&buf[..len as usize]).to_string())
        } else {
            None
        }
    }
}

// -------------------- browser history helpers --------------------

fn chrome_history_path() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    let mut p = PathBuf::from(base);
    p.push("Google");
    p.push("Chrome");
    p.push("User Data");
    p.push("Default");
    p.push("History");
    Some(p)
}

fn edge_history_path() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    let mut p = PathBuf::from(base);
    p.push("Microsoft");
    p.push("Edge");
    p.push("User Data");
    p.push("Default");
    p.push("History");
    Some(p)
}

fn brave_history_path() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    let mut p = PathBuf::from(base);
    p.push("BraveSoftware");
    p.push("Brave-Browser");
    p.push("User Data");
    p.push("Default");
    p.push("History");
    Some(p)
}

// Firefox profile (places.sqlite under %APPDATA%\Mozilla\Firefox\Profiles\<profile>\places.sqlite)
fn firefox_history_path() -> Option<PathBuf> {
    let base = std::env::var_os("APPDATA")?;
    let mut p = PathBuf::from(base);
    p.push("Mozilla");
    p.push("Firefox");
    p.push("Profiles");
    // pick the first profile directory found
    if p.exists() {
        if let Ok(mut entries) = std::fs::read_dir(&p) {
            if let Some(Ok(dir)) = entries.find(|e| e.is_ok()) {
                let mut places = dir.path();
                places.push("places.sqlite");
                return Some(places);
            }
        }
    }
    None
}

fn copy_history_to_temp(src: &Path, dest_name: &str) -> Option<PathBuf> {
    if !src.exists() {
        return None;
    }
    let mut dst = std::env::temp_dir();
    dst.push(dest_name);
    let _ = fs::copy(src, &dst).ok()?;
    Some(dst)
}

// Chrome/Chromium family: convert visit_time to unix seconds
fn read_recent_chromium_visits(history_db: &Path, since_unix: i64, limit: i64) -> SqlResult<Vec<(String, String, i64)>> {
    let conn = Connection::open(history_db)?;
    let mut stmt = conn.prepare(
        r#"
        SELECT
          urls.url,
          urls.title,
          CAST((visits.visit_time/1000000 - 11644473600) AS INTEGER) AS visited_unix
        FROM visits
        JOIN urls ON urls.id = visits.url
        WHERE (visits.visit_time/1000000 - 11644473600) > ?
        ORDER BY visited_unix DESC
        LIMIT ?
        "#,
    )?;

    let rows = stmt.query_map([since_unix, limit], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?))
    })?;

    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

// Firefox: visit_date is in microseconds since Unix epoch
fn read_recent_firefox_visits(history_db: &Path, since_unix: i64, limit: i64) -> SqlResult<Vec<(String, String, i64)>> {
    let conn = Connection::open(history_db)?;
    let mut stmt = conn.prepare(
        r#"
        SELECT
          moz_places.url,
          moz_places.title,
          CAST(visits.visit_date / 1000000 AS INTEGER) AS visited_unix
        FROM moz_historyvisits AS visits
        JOIN moz_places ON moz_places.id = visits.place_id
        WHERE (visits.visit_date / 1000000) > ?
        ORDER BY visited_unix DESC
        LIMIT ?
        "#
    )?;

    let rows = stmt.query_map([since_unix, limit], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?))
    })?;

    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

// -------------------- sync structures --------------------

#[derive(Serialize, Deserialize, Debug)]
struct LogEntry {
    timestamp: String,
    #[serde(rename = "type")]
    log_type: String,
    data: serde_json::Value,
}

#[derive(Serialize)]
struct SyncRequest {
    logs: Vec<LogEntry>,
}

// -------------------- sync functions --------------------

fn read_token_from_user() -> String {
    println!("🚀 Welcome to Chronos!");
    println!("Setting up your account for the first time...");
    println!();
    
    // Open the OAuth page in browser
    let auth_url = "https://chronos-red-five.vercel.app/auth/signin?source=desktop";
    println!("Opening your browser for authentication...");
    
    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(&["/c", "start", auth_url])
            .spawn()
            .ok();
    }
    
    println!("✅ Browser opened! Please:");
    println!("1. Sign in with Google or GitHub");
    println!("2. Go to 'Get Sync Token' in your dashboard");
    println!("3. Copy your sync token");
    println!("4. Paste it here");
    println!();
    print!("Enter your sync token: ");
    io::stdout().flush().unwrap();
    
    let mut token = String::new();
    io::stdin().read_line(&mut token).expect("Failed to read token");
    token.trim().to_string()
}

fn load_token() -> Option<String> {
    let token_path = get_app_data_dir().join("sync_token.txt");
    match std::fs::read_to_string(token_path) {
        Ok(token) => Some(token.trim().to_string()),
        Err(_) => None,
    }
}

fn save_token(token: &str) {
    let token_path = get_app_data_dir().join("sync_token.txt");
    let _ = std::fs::write(token_path, token);
}

async fn sync_logs_to_server(logs: Vec<LogEntry>, token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    let sync_request = SyncRequest { logs };
    
    let response = client
        .post("https://chronos-red-five.vercel.app/api/sync")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&sync_request)
        .send()
        .await?;

    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!("Sync successful: {}", result.get("message").unwrap_or(&serde_json::Value::String("Done".to_string())));
    } else {
        eprintln!("Sync failed: {}", response.status());
    }

    Ok(())
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    // Parse format: "2025-09-02 13:02:55 - Active window: 'Title' (proc: App.exe)"
    let parts: Vec<&str> = line.splitn(3, " - ").collect();
    if parts.len() < 3 {
        return None;
    }

    let timestamp = parts[0].trim();
    let content = parts[2].trim();

    if content.starts_with("Active window:") {
        // Parse window activity
        if let Some(start) = content.find("'") {
            if let Some(end) = content.rfind("'") {
                let title = &content[start + 1..end];
                if let Some(proc_start) = content.find("(proc: ") {
                    if let Some(proc_end) = content.rfind(")") {
                        let process_name = &content[proc_start + 7..proc_end];
                        
                        let mut data = serde_json::Map::new();
                        data.insert("windowTitle".to_string(), serde_json::Value::String(title.to_string()));
                        data.insert("processName".to_string(), serde_json::Value::String(process_name.to_string()));

                        return Some(LogEntry {
                            timestamp: timestamp.to_string(),
                            log_type: "window".to_string(),
                            data: serde_json::Value::Object(data),
                        });
                    }
                }
            }
        }
    } else if content.contains("Browser") && content.contains("visit:") {
        // Parse browser activity
        let parts: Vec<&str> = content.split(" | ").collect();
        if parts.len() >= 3 {
            let browser_type = if content.contains("Firefox") { "Firefox" } else { "Chromium" };
            let title = parts[1].trim();
            let url = parts[2].trim();
            
            let mut data = serde_json::Map::new();
            data.insert("browserType".to_string(), serde_json::Value::String(browser_type.to_string()));
            data.insert("browserTitle".to_string(), serde_json::Value::String(title.to_string()));
            data.insert("url".to_string(), serde_json::Value::String(url.to_string()));

            return Some(LogEntry {
                timestamp: timestamp.to_string(),
                log_type: "browser".to_string(),
                data: serde_json::Value::Object(data),
            });
        }
    }

    None
}

async fn sync_local_logs(token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = get_app_data_dir().join("activity_log.txt");
    if !log_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&log_path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    let mut log_entries = Vec::new();
    
    for line in lines {
        if let Some(entry) = parse_log_line(line) {
            log_entries.push(entry);
        } else {
            // Debug: log lines that couldn't be parsed
            if line.trim().len() > 0 && !line.contains("No new log entries") && !line.contains("Starting main") && !line.contains("Main loop iteration") {
                log_line(&format!("Could not parse log line: {}", line));
            }
        }
    }

    if !log_entries.is_empty() {
        let entry_count = log_entries.len();
        sync_logs_to_server(log_entries, token).await?;
        log_line(&format!("Synced {} log entries to server", entry_count));
    } else {
        log_line("No new log entries to sync");
    }

    Ok(())
}

// -------------------- main loop --------------------

#[tokio::main]
async fn main() {
    // Set up panic hook to log panics
    std::panic::set_hook(Box::new(|info| {
        let location = info.location().map_or("unknown".to_string(), |l| format!("{}:{}", l.file(), l.line()));
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s
        } else {
            "Unknown panic"
        };
        log_line(&format!("PANIC at {}: {}", location, msg));
    }));

    // Handle token setup first (with visible console)
    let token = match load_token() {
        Some(token) => {
            println!("✅ Chronos is running in the background");
            println!("Dashboard: https://chronos-red-five.vercel.app/dashboard");
            println!("Press Ctrl+C to stop or simply close this window");
            
            // Don't hide console immediately - wait until after setup
            token
        }
        None => {
            // Keep console visible for first-time setup
            println!("🎉 Welcome to Chronos Activity Tracker!");
            println!("Opening browser for authentication...");
            
            // Open browser to signin page
            let signin_url = "https://chronos-red-five.vercel.app/auth/signin?source=desktop";
            let _ = Command::new("cmd")
                .args(["/c", "start", signin_url])
                .spawn();
            
            println!();
            println!("Please complete the authentication in your browser, then:");
            println!("1. Visit: https://chronos-red-five.vercel.app/token");
            println!("2. Copy your token and paste it below");
            println!();
            
            let token = read_token_from_user();
            save_token(&token);
            println!("✅ Setup complete! Chronos is now running in the background.");
            println!("You can minimize this window. Check your dashboard for activity data.");
            
            // Wait a bit so user can see the success message
            thread::sleep(Duration::from_secs(3));
            
            token
        }
    };

    log_line("Chronos started");

    // Clone token for sync task
    let sync_token = token.clone();
    
    // Spawn periodic sync task with error handling
    tokio::spawn(async move {
        let mut sync_interval = tokio::time::interval(Duration::from_secs(30)); // Sync every 30 seconds for testing
        loop {
            sync_interval.tick().await;
            if let Err(e) = sync_local_logs(&sync_token).await {
                log_line(&format!("Sync error: {}", e));
            }
        }
    });

    // track last seen times to avoid duplicate browser logs
    let mut last_seen_chromium_unix: i64 = chrono::Utc::now().timestamp();
    let mut last_seen_firefox_unix: i64 = chrono::Utc::now().timestamp();

    // track last activity types to reduce logging noise
    let mut last_window: Option<(String, String)> = None;

    // Simplified approach: assume user is always active for now
    // This will be improved later once we get the basic tracking stable
    log_line("Starting main activity tracking loop (simplified mode)...");

    // Main loop with comprehensive error handling
    let mut loop_count = 0;
    loop {
        loop_count += 1;
        if loop_count % 12 == 1 { // Log every minute (12 * 5 seconds)
            log_line(&format!("Main loop iteration: {}", loop_count));
        }
        
        match tokio::time::timeout(Duration::from_secs(10), async {
            // For now, assume user is always active to test the basic functionality
            let active = true;

            if active {
                if let Some((title, exe)) = active_window_title_and_process() {
                    // only log when window changes
                    if Some((title.clone(), exe.clone())) != last_window {
                        log_line(&format!("Active window: '{}' (proc: {})", title, exe));
                        last_window = Some((title.clone(), exe.clone()));
                    }

                    let exe_lower = exe.to_lowercase();
                    // Chromium family
                    if exe_lower.contains("chrome") || exe_lower.contains("msedge") || exe_lower.contains("brave") {
                        if let Some(src) = chrome_history_path()
                            .or_else(edge_history_path)
                            .or_else(brave_history_path)
                        {
                            if let Some(copy) = copy_history_to_temp(&src, "chronos_chromium_history_copy.sqlite") {
                                if let Ok(visits) = read_recent_chromium_visits(&copy, last_seen_chromium_unix, 20) {
                                    for (url, title, ts) in visits.iter().rev() {
                                        if *ts > last_seen_chromium_unix {
                                            last_seen_chromium_unix = *ts;
                                            let dt = chrono::DateTime::from_timestamp(*ts, 0)
                                                .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap())
                                                .naive_utc();
                                            log_line(&format!("Browser (Chromium) visit: {} | {} | {}", dt, title, url));
                                        }
                                    }
                                }
                                let _ = fs::remove_file(copy);
                            }
                        }
                    }

                    // Firefox
                    if exe_lower.contains("firefox") {
                        if let Some(src) = firefox_history_path() {
                            if let Some(copy) = copy_history_to_temp(&src, "chronos_firefox_history_copy.sqlite") {
                                if let Ok(visits) = read_recent_firefox_visits(&copy, last_seen_firefox_unix, 20) {
                                    for (url, title, ts) in visits.iter().rev() {
                                        if *ts > last_seen_firefox_unix {
                                            last_seen_firefox_unix = *ts;
                                            let dt = chrono::DateTime::from_timestamp(*ts, 0)
                                                .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap())
                                                .naive_utc();
                                            log_line(&format!("Browser (Firefox) visit: {} | {} | {}", dt, title, url));
                                        }
                                    }
                                }
                                let _ = fs::remove_file(copy);
                            }
                        }
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }).await {
            Ok(_) => {}, // Normal execution
            Err(_) => {
                log_line("Main loop timeout - continuing...");
            }
        }
    }
}
