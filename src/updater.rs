#[cfg(target_os = "windows")]
use serde_json::Value;

#[cfg(target_os = "windows")]
fn cleanup_old_executable() {
    if let Ok(current_exe) = std::env::current_exe() {
        let old_exe = current_exe.with_extension("old.exe");
        if old_exe.exists() {
            let _ = std::fs::remove_file(&old_exe);
        }
    }
}

#[cfg(target_os = "windows")]
pub async fn check_and_update() {
    cleanup_old_executable();

    let client = match reqwest::Client::builder()
        .user_agent("redsky-updater")
        .build()
    {
        Ok(c) => c,
        Err(_) => return,
    };

    let res = client
        .get("https://api.github.com/repos/va1erian/redsky/releases/latest")
        .send()
        .await;
    if let Ok(response) = res {
        if let Ok(release) = response.json::<Value>().await {
            let tag_name = release["tag_name"].as_str().unwrap_or("");
            let current_version = env!("CARGO_PKG_VERSION");
            let tag_version = tag_name.trim_start_matches('v');

            let is_newer = is_newer_version(current_version, tag_version);

            if is_newer {
                if let Some(assets) = release["assets"].as_array() {
                    for asset in assets {
                        if asset["name"].as_str().unwrap_or("") == "redsky-Windows.zip" {
                            let download_url = asset["browser_download_url"].as_str().unwrap_or("");
                            if let Ok(zip_res) = client.get(download_url).send().await {
                                if let Ok(zip_bytes) = zip_res.bytes().await {
                                    apply_update(&zip_bytes);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn is_newer_version(current: &str, tag: &str) -> bool {
    let current_parts: Vec<&str> = current.split('.').collect();
    let tag_parts: Vec<&str> = tag.split('.').collect();

    for i in 0..std::cmp::max(current_parts.len(), tag_parts.len()) {
        let c: u32 = current_parts.get(i).unwrap_or(&"0").parse().unwrap_or(0);
        let t: u32 = tag_parts.get(i).unwrap_or(&"0").parse().unwrap_or(0);
        if t > c {
            return true;
        } else if t < c {
            return false;
        }
    }
    false
}

#[cfg(target_os = "windows")]
fn apply_update(zip_bytes: &[u8]) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let temp_dir = std::env::temp_dir();
    let zip_path = temp_dir.join("redsky_update.zip");
    if std::fs::write(&zip_path, zip_bytes).is_err() {
        return;
    }

    let extract_dir = temp_dir.join("redsky_update");
    let _ = std::fs::remove_dir_all(&extract_dir);
    if std::fs::create_dir_all(&extract_dir).is_err() {
        return;
    }

    let script = format!(
        "Expand-Archive -LiteralPath \"{}\" -DestinationPath \"{}\" -Force",
        zip_path.display(),
        extract_dir.display()
    );

    let status = std::process::Command::new("powershell")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(&script)
        .creation_flags(CREATE_NO_WINDOW)
        .status();

    if status.is_err() || !status.unwrap().success() {
        return;
    }

    let current_exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(_) => return,
    };

    let new_exe = extract_dir.join("redsky.exe");
    if new_exe.exists() {
        let old_exe = current_exe.with_extension("old.exe");
        let _ = std::fs::remove_file(&old_exe); // in case a previous update failed to clean up

        // On Windows, a running executable can be renamed, but not deleted/overwritten directly.
        if std::fs::rename(&current_exe, &old_exe).is_ok() {
            if std::fs::copy(&new_exe, &current_exe).is_err() {
                // If copying the new one fails, try to restore the old one
                let _ = std::fs::rename(&old_exe, &current_exe);
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub async fn check_and_update() {
    // Do nothing on other platforms
}
