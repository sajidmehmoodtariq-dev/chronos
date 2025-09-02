use chrono::Local;
use rusqlite::{Connection, Result as SqlResult};
use rdev::{listen, Event};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND};
use windows::Win32::System::Console::FreeConsole;
use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
};
use windows::Win32::UI::Input::KeyboardAndMouse::GetLastInputInfo;

// -------------------- logging --------------------

fn log_line(line: &str) {
    let now = Local::now();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("activity_log.txt")
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
        let handle: HANDLE = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            pid
        ).expect("Failed to open process");

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

// -------------------- main loop --------------------

fn main() {
    // hide console if running as built exe (still visible during `cargo run`)
    #[cfg(windows)]
    unsafe {
        let _ = FreeConsole();
    }

    log_line("Chronos started");

    // track last seen times to avoid duplicate browser logs
    let mut last_seen_chromium_unix: i64 = chrono::Utc::now().timestamp();
    let mut last_seen_firefox_unix: i64 = chrono::Utc::now().timestamp();

    // track last activity types to reduce logging noise
    let mut last_window: Option<(String, String)> = None;

    // shared last-input timestamp (ms since epoch) updated by input listener
    let last_input = Arc::new(AtomicI64::new(chrono::Utc::now().timestamp_millis()));
    {
        // spawn input listener thread (rdev) to track presence of keyboard/mouse activity
        let last_input_clone = last_input.clone();
        thread::spawn(move || {
            // rdev::listen runs until error; we only update timestamp on events
            let callback = move |event: Event| {
                let now_ms = chrono::Utc::now().timestamp_millis();
                last_input_clone.store(now_ms, Ordering::SeqCst);
            };
            if let Err(err) = listen(callback) {
                log_line(&format!("rdev listen error: {:?}", err));
            }
        });
    }

    loop {
        // determine idle via last_input timestamp (in ms)
        let last_ms = last_input.load(Ordering::SeqCst);
        let idle_ms = chrono::Utc::now().timestamp_millis() - last_ms;
        let active = idle_ms < 60_000; // active if < 60s since last input

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
        } else {
            // optional — you can log idle/active transitions if you want
            // log_line("User idle");
        }

        thread::sleep(Duration::from_secs(5));
    }
}
