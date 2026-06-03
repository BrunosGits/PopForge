use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use tauri::{Emitter, Manager};

// Global state
static LAST_FILE: Mutex<String> = Mutex::new(String::new());

#[derive(serde::Serialize)]
struct ConversionResult {
    success: bool,
    message: String,
    output_path: Option<String>,
    command_preview: Option<String>,
}

#[derive(serde::Serialize)]
struct ToolStatus {
    name: String,
    available: bool,
    detail: String,
    path: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    #[serde(default)]
    last_output_folder: String,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ConversionProgress {
    current: usize,
    total: usize,
    file_name: String,
    stage: String,
    file_percent: Option<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GameMetadata {
    serial: String,
    title: String,
    region: String,
    cover_path: Option<String>,
    source: String,
    cached: bool,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct ConversionOptions {
    mode: String,
    game_name: String,
    game_id: String,
    compression: u8,
    output_template: String,
    output_folder: String,
    popstation_path: Option<String>,
    chdman_path: Option<String>,
    icon0_path: Option<String>,
    pic0_path: Option<String>,
    pic1_path: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolPaths {
    popstation_path: Option<String>,
    chdman_path: Option<String>,
}

// Learn more about Tauri commands
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn test_backend() -> String {
    "PopForge backend is working".to_string()
}

#[tauri::command]
fn print_file_path(path: String) -> String {
    println!("Selected file: {}", path);

    if let Ok(mut last) = LAST_FILE.lock() {
        *last = path.clone();
    }

    path
}

#[tauri::command]
fn get_last_file() -> String {
    LAST_FILE.lock().unwrap().clone()
}

#[tauri::command]
fn get_toolchain_status(paths: Option<ToolPaths>) -> Vec<ToolStatus> {
    let custom_psxpackager = paths
        .as_ref()
        .and_then(|paths| paths.popstation_path.as_deref());
    let custom_chdman = paths
        .as_ref()
        .and_then(|paths| paths.chdman_path.as_deref());
    let psxpackager_program = resolve_psxpackager_path(custom_psxpackager)
        .to_string_lossy()
        .to_string();
    let chdman_program = resolve_chdman_path(custom_chdman)
        .to_string_lossy()
        .to_string();
    vec![
        tool_status_with_program(
            "psxpackager",
            psxpackager_program,
            custom_psxpackager.is_some(),
        ),
        tool_status_with_program(
            "chdman",
            chdman_program,
            custom_chdman.is_some(),
        ),
    ]
}

#[tauri::command]
async fn run_conversion(
    app: tauri::AppHandle,
    file_path: String,
    options: ConversionOptions,
    queue_index: Option<usize>,
    queue_total: Option<usize>,
) -> ConversionResult {
    let current = queue_index.unwrap_or(0);
    let total = queue_total.unwrap_or(0);
    let file_name = Path::new(&file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&file_path)
        .to_string();

    emit_progress(
        &app,
        ConversionProgress {
            current,
            total,
            file_name: file_name.clone(),
            stage: "starting".to_string(),
            file_percent: None,
        },
    );

    let app_for_task = app.clone();
    let file_path_for_task = file_path.clone();
    let file_name_for_task = file_name.clone();
    let join = tauri::async_runtime::spawn_blocking(move || {
        run_conversion_inner(
            &file_path_for_task,
            &options,
            &app_for_task,
            current,
            total,
            &file_name_for_task,
        )
    });

    let result = match join.await {
        Ok(result) => result,
        Err(error) => ConversionResult {
            success: false,
            message: format!("Conversion task panicked: {}", error),
            output_path: None,
            command_preview: None,
        },
    };

    emit_progress(
        &app,
        ConversionProgress {
            current,
            total,
            file_name: file_name.clone(),
            stage: if result.success {
                "completed".to_string()
            } else {
                "failed".to_string()
            },
            file_percent: Some(if result.success { 1.0 } else { 0.0 }),
        },
    );

    result
}

fn run_conversion_inner(
    file_path: &str,
    options: &ConversionOptions,
    app: &tauri::AppHandle,
    current: usize,
    total: usize,
    file_name: &str,
) -> ConversionResult {
    println!("Conversion started for: {}", file_path);

    if file_path.trim().is_empty() {
        return ConversionResult {
            success: false,
            message: "No input file selected.".to_string(),
            output_path: None,
            command_preview: None,
        };
    }

    let input_path = Path::new(file_path);

    if !input_path.exists() {
        return ConversionResult {
            success: false,
            message: format!("Input file not found: {}", file_path),
            output_path: None,
            command_preview: None,
        };
    }

    if options.output_folder.trim().is_empty() {
        return ConversionResult {
            success: false,
            message: "Choose an output folder before running the queue.".to_string(),
            output_path: None,
            command_preview: None,
        };
    }

    let output_folder = Path::new(&options.output_folder);

    if !output_folder.exists() {
        if let Err(error) = fs::create_dir_all(output_folder) {
            return ConversionResult {
                success: false,
                message: format!(
                    "Could not create output folder '{}': {}",
                    options.output_folder, error
                ),
                output_path: None,
                command_preview: None,
            };
        }
        println!("[info] Created output folder: {}", options.output_folder);
    } else if !output_folder.is_dir() {
        return ConversionResult {
            success: false,
            message: format!(
                "Output path exists but is not a directory: {}",
                options.output_folder
            ),
            output_path: None,
            command_preview: None,
        };
    }

    if options.mode == "extract" {
        let input_title = input_path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("Untitled");
        let output_path = output_folder
            .join(input_title)
            .to_string_lossy()
            .to_string();

        let psxpackager_program = resolve_psxpackager_path(options.popstation_path.as_deref())
            .to_string_lossy()
            .to_string();

        let required_tools = vec![ToolRequirement {
            name: "psxpackager",
            program: psxpackager_program.clone(),
        }];

        if let Some(missing_tool) = first_missing_tool(&required_tools) {
            return ConversionResult {
                success: false,
                message: missing_tool,
                output_path: None,
                command_preview: None,
            };
        }

        let psx_args = vec![
            "-i".to_string(),
            file_path.to_string(),
            "-o".to_string(),
            output_folder.to_string_lossy().to_string(),
            "-x".to_string(),
        ];

        let command_preview = format!(
            "\"{}\" -i \"{}\" -o \"{}\" -x",
            psxpackager_program,
            file_path,
            output_folder.display()
        );

        let step = CommandStep {
            program: psxpackager_program.clone(),
            args: psx_args,
            stage: "psxpackager".to_string(),
        };

        emit_progress(
            app,
            ConversionProgress {
                current,
                total,
                file_name: file_name.to_string(),
                stage: "psxpackager".to_string(),
                file_percent: None,
            },
        );

        let output = Command::new(&step.program)
            .args(&step.args)
            .output()
            .map_err(|error| format!("Failed to start {}: {}", step.program, error));

        let result = match output {
            Ok(output) => {
                if output.status.success() {
                    ConversionResult {
                        success: true,
                        message: format!("Extracted BIN+CUE to: {}", output_folder.display()),
                        output_path: Some(output_path),
                        command_preview: Some(command_preview),
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    let details = if stderr.is_empty() {
                        stdout
                    } else {
                        stderr
                    };
                    ConversionResult {
                        success: false,
                        message: if details.is_empty() {
                            format!("{} failed with status {}", step.program, output.status)
                        } else {
                            format!("{} failed: {}", step.program, details)
                        },
                        output_path: Some(output_path),
                        command_preview: Some(command_preview),
                    }
                }
            }
            Err(error) => ConversionResult {
                success: false,
                message: format!("Failed to start {}: {}", step.program, error),
                output_path: Some(output_path),
                command_preview: Some(command_preview),
            },
        };

        emit_progress(
            app,
            ConversionProgress {
                current,
                total,
                file_name: file_name.to_string(),
                stage: if result.success {
                    "completed".to_string()
                } else {
                    "failed".to_string()
                },
                file_percent: Some(if result.success { 1.0 } else { 0.0 }),
            },
        );

        return result;
    }

    let input_title = input_path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("Untitled");
    let output_path = output_folder
        .join(format!("{}.PBP", input_title))
        .to_string_lossy()
        .to_string();

    let assets = match stage_psp_assets(
        options.icon0_path.as_deref(),
        options.pic0_path.as_deref(),
        options.pic1_path.as_deref(),
    ) {
        Ok(assets) => assets,
        Err(error) => {
            return ConversionResult {
                success: false,
                message: format!("Could not stage PSP assets: {}", error),
                output_path: Some(output_path),
                command_preview: None,
            };
        }
    };
    println!(
        "[info] Staged PSP assets at {}",
        assets.temp_dir.display()
    );

    let pipeline = build_conversion_pipeline(input_path, &output_path, options, &assets);
    let command_preview = pipeline.preview.join("\n");

    if let Some(missing_tool) = first_missing_tool(&pipeline.required_tools) {
        return ConversionResult {
            success: false,
            message: missing_tool,
            output_path: Some(output_path),
            command_preview: Some(command_preview),
        };
    }

    if let Err(message) = run_pipeline(&pipeline.steps, app, current, total, file_name) {
        return ConversionResult {
            success: false,
            message,
            output_path: Some(output_path),
            command_preview: Some(command_preview),
        };
    }

    ConversionResult {
        success: true,
        message: format!("Created EBOOT: {}", output_path),
        output_path: Some(output_path),
        command_preview: Some(command_preview),
    }
}

fn emit_progress(app: &tauri::AppHandle, progress: ConversionProgress) {
    let _ = app.emit("conversion-progress", &progress);
}

struct ConversionPipeline {
    steps: Vec<CommandStep>,
    preview: Vec<String>,
    required_tools: Vec<ToolRequirement>,
}

struct CommandStep {
    program: String,
    args: Vec<String>,
    stage: String,
}

struct ToolRequirement {
    name: &'static str,
    program: String,
}

fn build_conversion_pipeline(
    input_path: &Path,
    output_path: &str,
    options: &ConversionOptions,
    assets: &StagedAssets,
) -> ConversionPipeline {
    let mut steps = Vec::new();
    let mut preview = Vec::new();
    let psxpackager_program = resolve_psxpackager_path(options.popstation_path.as_deref())
        .to_string_lossy()
        .to_string();
    let chdman_program = resolve_chdman_path(options.chdman_path.as_deref())
        .to_string_lossy()
        .to_string();
    let mut required_tools = vec![ToolRequirement {
        name: "psxpackager",
        program: psxpackager_program.clone(),
    }];
    let mut psxpackager_input = input_path.to_path_buf();

    if has_extension(input_path, "chd") {
        required_tools.insert(
            0,
            ToolRequirement {
                name: "chdman",
                program: chdman_program.clone(),
            },
        );
        let normalized_stem = input_path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("popforge-normalized");
        let normalized_dir = std::env::temp_dir().join("popforge").join(normalized_stem);
        let normalized_cue = normalized_dir.join(format!("{}.cue", normalized_stem));
        let normalized_bin = normalized_dir.join(format!("{}.bin", normalized_stem));

        steps.push(CommandStep {
            program: chdman_program.clone(),
            args: vec![
                "extractcd".to_string(),
                "-i".to_string(),
                input_path.to_string_lossy().to_string(),
                "-o".to_string(),
                normalized_cue.to_string_lossy().to_string(),
                "-ob".to_string(),
                normalized_bin.to_string_lossy().to_string(),
                "-f".to_string(),
            ],
            stage: "chdman".to_string(),
        });
        preview.push(format!(
            "\"{}\" extractcd -i \"{}\" -o \"{}\" -ob \"{}\" -f",
            chdman_program,
            input_path.display(),
            normalized_cue.display(),
            normalized_bin.display()
        ));
        psxpackager_input = normalized_cue;
    }

    let output_dir = Path::new(output_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let psx_args = vec![
        "-i".to_string(),
        psxpackager_input.to_string_lossy().to_string(),
        "-o".to_string(),
        output_dir.clone(),
        "-l".to_string(),
        options.compression.to_string(),
        "--icon0".to_string(),
        assets.icon0.to_string_lossy().to_string(),
        "--pic0".to_string(),
        assets.pic0.to_string_lossy().to_string(),
        "--pic1".to_string(),
        assets.pic1.to_string_lossy().to_string(),
        "-x".to_string(),
    ];
    steps.push(CommandStep {
        program: psxpackager_program.clone(),
        args: psx_args,
        stage: "psxpackager".to_string(),
    });
    preview.push(format!(
        "\"{}\" -i \"{}\" -o \"{}\" -l {} --icon0 \"{}\" --pic0 \"{}\" --pic1 \"{}\" -x",
        psxpackager_program,
        psxpackager_input.display(),
        output_dir,
        options.compression,
        assets.icon0.display(),
        assets.pic0.display(),
        assets.pic1.display()
    ));

    ConversionPipeline {
        steps,
        preview,
        required_tools,
    }
}

fn run_pipeline(
    steps: &[CommandStep],
    app: &tauri::AppHandle,
    current: usize,
    total: usize,
    file_name: &str,
) -> Result<(), String> {
    for step in steps {
        ensure_parent_dirs(&step.args)?;

        emit_progress(
            app,
            ConversionProgress {
                current,
                total,
                file_name: file_name.to_string(),
                stage: step.stage.clone(),
                file_percent: None,
            },
        );

        let output = Command::new(&step.program)
            .args(&step.args)
            .output()
            .map_err(|error| format!("Failed to start {}: {}", step.program, error))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let details = if stderr.is_empty() { stdout } else { stderr };

            return Err(if details.is_empty() {
                format!("{} failed with status {}", step.program, output.status)
            } else {
                format!("{} failed: {}", step.program, details)
            });
        }
    }

    Ok(())
}

fn ensure_parent_dirs(args: &[String]) -> Result<(), String> {
    for arg in args {
        let path = Path::new(arg);

        if let Some(parent) = path.parent() {
            if parent.as_os_str().is_empty() || !arg.contains(std::path::MAIN_SEPARATOR) {
                continue;
            }

            fs::create_dir_all(parent)
                .map_err(|error| format!("Could not create {}: {}", parent.display(), error))?;
        }
    }

    Ok(())
}

fn first_missing_tool(tools: &[ToolRequirement]) -> Option<String> {
    tools
        .iter()
        .find(|tool| !command_exists(&tool.program))
        .map(|tool| match tool.name {
            "psxpackager" => format!(
                "PSXPackager was not found at '{}'. Place the PSXPackager binary at src-tauri/bin/PSXPackager, set a custom path in the UI, or install PSXPackager on PATH.",
                tool.program
            ),
            "chdman" => format!(
                "chdman was not found at '{}'. chdman support is currently disabled.",
                tool.program
            ),
            _ => {
                if tool.program == tool.name {
                    tool.name.to_string()
                } else {
                    format!("{} at {}", tool.name, tool.program)
                }
            }
        })
}

fn command_exists(program: &str) -> bool {
    match run_tool_probe(program, &["--version"]) {
        Ok(_) => true,
        Err(error) => error.kind() != ErrorKind::NotFound,
    }
}

fn current_target_triple() -> &'static str {
    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    {
        "aarch64-apple-darwin"
    }
    #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
    {
        "x86_64-apple-darwin"
    }
    #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
    {
        "x86_64-pc-windows-msvc"
    }
    #[cfg(all(target_arch = "aarch64", target_os = "windows"))]
    {
        "aarch64-pc-windows-msvc"
    }
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    {
        "x86_64-unknown-linux-gnu"
    }
    #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
    {
        "aarch64-unknown-linux-gnu"
    }
    #[cfg(not(any(
        all(target_arch = "aarch64", target_os = "macos"),
        all(target_arch = "x86_64", target_os = "macos"),
        all(target_arch = "x86_64", target_os = "windows"),
        all(target_arch = "aarch64", target_os = "windows"),
        all(target_arch = "x86_64", target_os = "linux"),
        all(target_arch = "aarch64", target_os = "linux"),
    )))]
    {
        "unknown"
    }
}

fn resolve_tool_path(tool: &str, custom_path: Option<&str>) -> PathBuf {
    if let Some(custom) = custom_path.map(str::trim).filter(|path| !path.is_empty()) {
        return PathBuf::from(custom);
    }

    let exe_suffix = std::env::consts::EXE_SUFFIX;
    let triple = current_target_triple();
    let bare = format!("{}{}", tool, exe_suffix);
    let suffixed = format!("{}-{}{}", tool, triple, exe_suffix);

    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(|p| p.to_path_buf());

        while let Some(d) = dir {
            let candidates = [
                d.join("bin").join(&bare),
                d.join("bin").join(&suffixed),
                d.join("Resources").join(&suffixed),
                d.join("../Resources").join(&suffixed),
            ];

            for candidate in candidates {
                if candidate.is_file() {
                    return candidate;
                }
            }

            dir = d.parent().map(|p| p.to_path_buf());
        }
    }

    PathBuf::from(bare)
}

fn resolve_psxpackager_path(custom_path: Option<&str>) -> PathBuf {
    resolve_tool_path("PSXPackager", custom_path)
}

fn resolve_chdman_path(custom_path: Option<&str>) -> PathBuf {
    resolve_tool_path("chdman", custom_path)
}

fn run_tool_probe(program: &str, args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new(program).args(args).output()
}

struct StagedAssets {
    icon0: PathBuf,
    pic0: PathBuf,
    pic1: PathBuf,
    temp_dir: PathBuf,
}

fn resolve_resources_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let mut dir = exe.parent().map(|p| p.to_path_buf());

    while let Some(d) = dir {
        for candidate in [
            d.join("Resources"),
            d.join("bin").join("Resources"),
            d.join("../Resources"),
            d.join("../../Resources"),
        ] {
            if candidate.is_dir() {
                return Some(candidate);
            }
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }

    None
}

fn stage_psp_assets(
    icon0: Option<&str>,
    pic0: Option<&str>,
    pic1: Option<&str>,
) -> Result<StagedAssets, String> {
    let resources = resolve_resources_dir()
        .ok_or_else(|| "Could not locate bundled Resources directory".to_string())?;

    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_dir = std::env::temp_dir()
        .join("popforge")
        .join(format!("assets-{}-{}", pid, nanos));
    fs::create_dir_all(&temp_dir)
        .map_err(|error| format!("Could not create {}: {}", temp_dir.display(), error))?;

    let icon0_path = stage_single_asset(&resources, "ICON0.PNG", icon0, &temp_dir)?;
    let pic0_path = stage_single_asset(&resources, "PIC0.PNG", pic0, &temp_dir)?;
    let pic1_path = stage_single_asset(&resources, "PIC1.PNG", pic1, &temp_dir)?;

    Ok(StagedAssets {
        icon0: icon0_path,
        pic0: pic0_path,
        pic1: pic1_path,
        temp_dir,
    })
}

fn stage_single_asset(
    resources: &Path,
    default_name: &str,
    user_path: Option<&str>,
    temp_dir: &Path,
) -> Result<PathBuf, String> {
    let source = match user_path.map(str::trim).filter(|s| !s.is_empty()) {
        Some(user) => PathBuf::from(user),
        None => resources.join(default_name),
    };

    if !source.is_file() {
        return Err(format!("Asset not found: {}", source.display()));
    }

    let dest = temp_dir.join(default_name);
    fs::copy(&source, &dest)
        .map_err(|error| format!("Could not copy {}: {}", source.display(), error))?;
    Ok(dest)
}

fn tool_status_with_program(tool: &str, program: String, has_custom_path: bool) -> ToolStatus {
    match run_tool_probe(&program, &["--version"]) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let detail = stdout
                .lines()
                .chain(stderr.lines())
                .map(str::trim)
                .find(|line| !line.is_empty())
                .unwrap_or("Found.")
                .to_string();

            ToolStatus {
                name: tool.to_string(),
                available: true,
                detail,
                path: Some(program),
            }
        }

        Err(error) if error.kind() == ErrorKind::NotFound => {
            let detail = if has_custom_path {
                "Custom path not found.".to_string()
            } else {
                "Not found.".to_string()
            };

            ToolStatus {
                name: tool.to_string(),
                available: false,
                detail,
                path: Some(program),
            }
        }

        Err(error) => ToolStatus {
            name: tool.to_string(),
            available: false,
            detail: format!("Could not probe tool: {}", error),
            path: Some(program),
        },
    }
}

fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case(extension))
        .unwrap_or(false)
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("Could not resolve app config dir: {}", error))?;
    Ok(dir.join("popforge.json"))
}

fn read_settings(app: &tauri::AppHandle) -> AppSettings {
    let Ok(path) = settings_path(app) else {
        return AppSettings::default();
    };

    let Ok(contents) = fs::read_to_string(&path) else {
        return AppSettings::default();
    };

    let Ok(value) = serde_json::from_str::<serde_json::Value>(&contents) else {
        return AppSettings::default();
    };

    AppSettings {
        last_output_folder: value
            .get("last_output_folder")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    }
}

fn write_settings(app: &tauri::AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(app)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create {}: {}", parent.display(), error))?;
    }

    let json = serde_json::json!({
        "last_output_folder": settings.last_output_folder,
    });
    let contents = serde_json::to_string_pretty(&json)
        .map_err(|error| format!("Could not serialize settings: {}", error))?;

    fs::write(&path, contents)
        .map_err(|error| format!("Could not write {}: {}", path.display(), error))
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> AppSettings {
    read_settings(&app)
}

#[tauri::command]
fn save_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    write_settings(&app, &settings)
}

fn extract_serial_from_filename(filename: &str) -> Option<String> {
    let re = regex::Regex::new(r"[A-Z]{4}-\d{4,5}").ok()?;
    re.find(filename).map(|m| m.as_str().to_string())
}

#[tauri::command]
fn extract_serial(filename: String) -> Option<String> {
    extract_serial_from_filename(&filename)
}

fn extract_title_from_filename(filename: &str) -> Option<String> {
    let stem = filename.rsplit_once('.').map(|(s, _)| s).unwrap_or(filename);
    let re = regex::Regex::new(r"^([^(\[]+?)(?:\s*[\(\[]|$)").ok()?;
    re.captures(stem).map(|c| c[1].trim().to_string())
}

fn region_name_from_serial(serial: &str) -> &'static str {
    match serial.chars().nth(2) {
        Some('U') => "USA",
        Some('E') => "Europe",
        Some('P') => "Japan",
        Some('A') => "Asia",
        Some('K') => "Korea",
        _ => "Unknown",
    }
}

fn psxdatacenter_region_code(serial: &str) -> &'static str {
    match serial.chars().nth(2) {
        Some('U') => "U",
        Some('E') => "P",
        Some('P') => "J",
        Some('A') => "A",
        _ => "U",
    }
}

fn psxdatacenter_url(serial: &str, first_letter: char) -> String {
    let region = psxdatacenter_region_code(serial);
    let letter = first_letter.to_ascii_uppercase();
    format!(
        "https://psxdatacenter.com/games/{}/{}/{}.html",
        region, letter, serial
    )
}

fn psxdatacenter_cover_url(serial: &str) -> String {
    let region = psxdatacenter_region_code(serial);
    format!(
        "https://psxdatacenter.com/games/images/cover/{}/{}.jpg",
        region, serial
    )
}

fn fetch_cover(app: &tauri::AppHandle, serial: &str) -> Option<String> {
    let root = metadata_cache_root(app).ok()?;
    let cover_path = root.join("covers").join(format!("{}.jpg", serial));

    if cover_path.exists() {
        return Some(cover_path.to_string_lossy().to_string());
    }

    let url = psxdatacenter_cover_url(serial);
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(10))
        .build();

    let request = agent
        .get(&url)
        .set("User-Agent", "PopForge/0.1 (+https://github.com/popforge)")
        .set("Accept", "image/jpeg");

    match request.call() {
        Ok(resp) => {
            let mut reader = resp.into_reader();
            let mut file = fs::File::create(&cover_path).ok()?;
            std::io::copy(&mut reader, &mut file).ok()?;
            Some(cover_path.to_string_lossy().to_string())
        }
        Err(_) => None,
    }
}

fn parse_psxdatacenter_page(html: &str) -> Option<(String, String)> {
    let text = strip_html(html);

    let title_re = regex::Regex::new(r"(?im)^\s*Official Title\s+(.+?)\s*$").ok()?;
    let common_title_re = regex::Regex::new(r"(?im)^\s*Common Title\s+(.+?)\s*$").ok()?;
    let region_re = regex::Regex::new(r"(?im)^\s*Region\s+(.+?)\s*$").ok()?;

    let title = title_re
        .captures(&text)
        .or_else(|| common_title_re.captures(&text))
        .map(|c| c[1].trim().to_string())?;

    let region = region_re
        .captures(&text)
        .map(|c| c[1].trim().to_string())
        .unwrap_or_default();

    Some((title, region))
}

fn strip_html(html: &str) -> String {
    let tag_re = regex::Regex::new(r"<[^>]+>").unwrap();
    let stripped = tag_re.replace_all(html, " ");
    stripped
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#039;", "'")
}

fn fetch_from_psxdatacenter(serial: &str, title_hint: Option<&str>) -> Option<GameMetadata> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(10))
        .build();

    let mut letters: Vec<char> = Vec::with_capacity(26);
    if let Some(t) = title_hint {
        if let Some(first) = t.chars().next() {
            let upper = first.to_ascii_uppercase();
            if upper.is_ascii_alphabetic() {
                letters.push(upper);
            }
        }
    }
    for c in 'A'..='Z' {
        if !letters.contains(&c) {
            letters.push(c);
        }
    }

    for letter in letters {
        let url = psxdatacenter_url(serial, letter);
        let request = agent
            .get(&url)
            .set("User-Agent", "PopForge/0.1 (+https://github.com/popforge)")
            .set("Accept", "text/html");
        match request.call() {
            Ok(resp) => {
                if let Ok(html) = resp.into_string() {
                    if let Some((title, region)) = parse_psxdatacenter_page(&html) {
                        return Some(GameMetadata {
                            serial: serial.to_string(),
                            title,
                            region: if region.is_empty() {
                                region_name_from_serial(serial).to_string()
                            } else {
                                region
                            },
                            cover_path: None,
                            source: "psxdatacenter".to_string(),
                            cached: false,
                        });
                    }
                }
            }
            Err(_) => continue,
        }
    }

    None
}

fn stub_metadata(serial: &str) -> GameMetadata {
    GameMetadata {
        serial: serial.to_string(),
        title: format!("Game {}", serial),
        region: region_name_from_serial(serial).to_string(),
        cover_path: None,
        source: "stub".to_string(),
        cached: false,
    }
}

fn metadata_cache_root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_cache_dir()
        .map_err(|error| format!("Could not resolve cache dir: {}", error))?;
    let cache = base.join("cache");
    fs::create_dir_all(cache.join("covers"))
        .map_err(|error| format!("Could not create cache covers: {}", error))?;
    Ok(cache)
}

fn load_metadata_cache(app: &tauri::AppHandle) -> HashMap<String, GameMetadata> {
    let Ok(root) = metadata_cache_root(app) else {
        return HashMap::new();
    };
    let path = root.join("metadata.json");
    let Ok(contents) = fs::read_to_string(&path) else {
        return HashMap::new();
    };
    serde_json::from_str(&contents).unwrap_or_default()
}

fn save_metadata_cache(
    app: &tauri::AppHandle,
    cache: &HashMap<String, GameMetadata>,
) -> Result<(), String> {
    let root = metadata_cache_root(app)?;
    let path = root.join("metadata.json");
    let contents = serde_json::to_string_pretty(cache)
        .map_err(|error| format!("Could not serialize metadata: {}", error))?;
    fs::write(&path, contents)
        .map_err(|error| format!("Could not write {}: {}", path.display(), error))
}

#[tauri::command]
async fn scrape_metadata(app: tauri::AppHandle, file_name: String) -> GameMetadata {
    let app_for_task = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        scrape_metadata_blocking(&app_for_task, &file_name)
    })
    .await;
    result.unwrap_or_else(|error| GameMetadata {
        serial: String::new(),
        title: format!("Scrape task panicked: {}", error),
        region: "Unknown".to_string(),
        cover_path: None,
        source: "error".to_string(),
        cached: false,
    })
}

fn scrape_metadata_blocking(app: &tauri::AppHandle, file_name: &str) -> GameMetadata {
    let serial = match extract_serial_from_filename(file_name) {
        Some(serial) => serial,
        None => {
            return GameMetadata {
                serial: String::new(),
                title: "Unknown title".to_string(),
                region: "Unknown".to_string(),
                cover_path: None,
                source: "no-serial".to_string(),
                cached: false,
            };
        }
    };

    let mut cache = load_metadata_cache(app);
    if let Some(meta) = cache.get(&serial) {
        let mut m = meta.clone();
        m.cached = true;
        return m;
    }

    let title_hint = extract_title_from_filename(file_name);
    let metadata = fetch_from_psxdatacenter(&serial, title_hint.as_deref())
        .unwrap_or_else(|| stub_metadata(&serial));

    let cover_path = fetch_cover(app, &serial);
    let mut metadata = metadata;
    metadata.cover_path = cover_path;

    cache.insert(serial, metadata.clone());
    let _ = save_metadata_cache(app, &cache);
    metadata
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            test_backend,
            print_file_path,
            get_last_file,
            get_toolchain_status,
            run_conversion,
            get_settings,
            save_settings,
            scrape_metadata,
            extract_serial
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_serial_from_filename() {
        assert_eq!(extract_serial_from_filename("Crash Bandicoot [SCUS-94244].iso"), Some("SCUS-94244".to_string()));
        assert_eq!(extract_serial_from_filename("Tekken 3 (SCES-01234).bin"), Some("SCES-01234".to_string()));
        assert_eq!(extract_serial_from_filename("no_serial_here.iso"), None);
    }

    #[test]
    fn test_extract_title_from_filename() {
        assert_eq!(extract_title_from_filename("Crash Bandicoot (USA).iso"), Some("Crash Bandicoot".to_string()));
        assert_eq!(extract_title_from_filename("Tekken 3 [SCES-01234].bin"), Some("Tekken 3".to_string()));
        assert_eq!(extract_title_from_filename("Simple.iso"), Some("Simple".to_string()));
    }

    #[test]
    fn test_region_name_from_serial() {
        assert_eq!(region_name_from_serial("SCUS-94244"), "USA");
        assert_eq!(region_name_from_serial("SCES-01234"), "Europe");
        assert_eq!(region_name_from_serial("SCPS-12345"), "Japan");
        assert_eq!(region_name_from_serial("SCAS-12345"), "Asia");
        assert_eq!(region_name_from_serial("SCKS-12345"), "Korea");
        assert_eq!(region_name_from_serial("XXXX-12345"), "Unknown");
    }

    #[test]
    fn test_psxdatacenter_region_code() {
        assert_eq!(psxdatacenter_region_code("SCUS-94244"), "U");
        assert_eq!(psxdatacenter_region_code("SCES-01234"), "P");
        assert_eq!(psxdatacenter_region_code("SCPS-12345"), "J");
        assert_eq!(psxdatacenter_region_code("SCAS-12345"), "A");
    }

    #[test]
    fn test_psxdatacenter_url() {
        let url = psxdatacenter_url("SCUS-94244", 'C');
        assert!(url.contains("SCUS-94244"));
        assert!(url.contains("/U/"));
        assert!(url.starts_with("https://psxdatacenter.com/games/"));
    }

    #[test]
    fn test_psxdatacenter_cover_url() {
        let url = psxdatacenter_cover_url("SCUS-94244");
        assert!(url.contains("SCUS-94244"));
        assert!(url.contains("/U/"));
        assert!(url.ends_with(".jpg"));
    }

    #[test]
    fn test_strip_html() {
        assert_eq!(strip_html("<b>Hello</b>"), " Hello ");
        assert_eq!(strip_html("A &amp; B"), "A & B");
        assert_eq!(strip_html("no tags"), "no tags");
    }

    #[test]
    fn test_stub_metadata() {
        let meta = stub_metadata("SCUS-94244");
        assert_eq!(meta.serial, "SCUS-94244");
        assert_eq!(meta.region, "USA");
        assert_eq!(meta.source, "stub");
        assert!(!meta.cached);
    }
}
