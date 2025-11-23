use std::fs;
use std::path::Path;
use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

pub fn extract_project_name(project_path: &Path) -> String {
    let mut current_path = project_path.to_path_buf();

    for _ in 0..4 {
        // 1. JetBrains (.idea/.name)
        let idea_name_path = current_path.join(".idea").join(".name");
        if idea_name_path.exists() {
            if let Ok(name) = fs::read_to_string(idea_name_path) {
                return name.trim().to_string();
            }
        }

        // Iterate through the directory to find other project files
        if let Ok(entries) = fs::read_dir(&current_path) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                        match extension {
                            // 2. Visual Studio (.sln)
                            "sln" => {
                                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                    return stem.to_string();
                                }
                            },
                            // 3. R-Studio (.Rproj)
                            "Rproj" => {
                                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                    return stem.to_string();
                                }
                            },
                            // 4. VS Code Workspace (.code-workspace)
                            "code-workspace" => {
                                if let Ok(content) = fs::read_to_string(&path) {
                                    if let Ok(json) = serde_json::from_str::<JsonValue>(&content) {
                                        if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                                            return name.to_string();
                                        }
                                    }
                                }
                            },
                            _ => {}
                        }
                    }

                    if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                        // 5. Node.js (package.json)
                        if filename == "package.json" {
                            if let Ok(content) = fs::read_to_string(&path) {
                                if let Ok(json) = serde_json::from_str::<JsonValue>(&content) {
                                    if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                                        return name.to_string();
                                    }
                                }
                            }
                        }
                        // 6. Python (pyproject.toml)
                        else if filename == "pyproject.toml" {
                            if let Ok(content) = fs::read_to_string(&path) {
                                if let Ok(toml) = content.parse::<TomlValue>() {
                                    if let Some(name) = toml.get("project").and_then(|p| p.get("name")).and_then(|n| n.as_str()) {
                                        return name.to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !current_path.pop() {
            break;
        }
    }

    // Powerlevel10k style fallback
    let parts: Vec<&str> = project_path
        .to_string_lossy()
        .split(|c| c == '\\' || c == '/')
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() <= 2 {
        return parts.last().copied().unwrap_or("Unknown").to_string();
    }

    // Abbreviate all but last 2 segments
    let abbreviated: Vec<String> = parts[..parts.len()-2]
        .iter()
        .filter_map(|s| s.chars().next())
        .map(|c| c.to_string())
        .collect();

    let last_two = parts[parts.len()-2..].join("/");

    format!("{}/{}", abbreviated.join("/"), last_two)
}

/// 파일 크기로 메시지 수 추정 (더 정확한 계산)
pub fn estimate_message_count_from_size(file_size: u64) -> usize {
    // 평균적으로 JSON 메시지는 800-1200 바이트
    // 작은 파일은 최소 1개 메시지로 처리
    ((file_size as f64 / 1000.0).ceil() as usize).max(1)
}
