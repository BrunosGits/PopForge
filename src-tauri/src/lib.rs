use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Mutex;

use tauri::{Emitter, Manager, PhysicalSize, Size, WindowEvent};

// Cancel support
static CANCEL: AtomicBool = AtomicBool::new(false);
static CHILD_PID: AtomicI32 = AtomicI32::new(0);

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

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    #[serde(default)]
    last_output_folder: String,
    #[serde(default)]
    last_mode: String,
    #[serde(default = "default_compression")]
    compression: u8,
    #[serde(default)]
    output_template: String,
    #[serde(default)]
    game_name: String,
    #[serde(default)]
    game_id: String,
    #[serde(default = "default_window_width")]
    window_width: u32,
    #[serde(default = "default_window_height")]
    window_height: u32,
    #[serde(default)]
    subfolder_per_game: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            last_output_folder: String::new(),
            last_mode: String::new(),
            compression: 0,
            output_template: String::new(),
            game_name: String::new(),
            game_id: String::new(),
            window_width: 800,
            window_height: 600,
            subfolder_per_game: false,
        }
    }
}

fn default_compression() -> u8 {
    0
}

fn default_window_width() -> u32 {
    800
}

fn default_window_height() -> u32 {
    600
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
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConversionOptions {
    mode: String,
    game_name: String,
    game_id: String,
    compression: u8,
    output_template: String,
    output_folder: String,
    popstation_path: Option<String>,
    icon0_path: Option<String>,
    pic0_path: Option<String>,
    pic1_path: Option<String>,
    #[serde(default)]
    disc_paths: Vec<String>,
    #[serde(default)]
    subfolder_per_game: bool,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolPaths {
    popstation_path: Option<String>,
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
    LAST_FILE.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

#[tauri::command]
fn get_toolchain_status(paths: Option<ToolPaths>) -> Vec<ToolStatus> {
    let custom_psxpackager = paths
        .as_ref()
        .and_then(|paths| paths.popstation_path.as_deref());
    let psxpackager_program = resolve_psxpackager_path(custom_psxpackager)
        .to_string_lossy()
        .to_string();
    vec![tool_status_with_program(
        "psxpackager",
        psxpackager_program,
        custom_psxpackager.is_some(),
    )]
}

#[tauri::command]
fn cancel_conversion() -> String {
    CANCEL.store(true, Ordering::Relaxed);
    let pid = CHILD_PID.load(Ordering::Relaxed);
    if pid > 0 {
        #[cfg(unix)]
        {
            let _ = std::process::Command::new("kill")
                .arg(pid.to_string())
                .status();
        }
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("taskkill")
                .args(&["/PID", &pid.to_string(), "/F"])
                .status();
        }
    }
    "Cancelled".to_string()
}

#[tauri::command]
async fn run_conversion(
    app: tauri::AppHandle,
    file_path: String,
    options: ConversionOptions,
    queue_index: Option<usize>,
    queue_total: Option<usize>,
) -> ConversionResult {
    CANCEL.store(false, Ordering::Relaxed);
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

    if CANCEL.load(Ordering::Relaxed) {
        CANCEL.store(false, Ordering::Relaxed);
        emit_progress(
            &app,
            ConversionProgress {
                current,
                total,
                file_name: file_name.clone(),
                stage: "cancelled".to_string(),
                file_percent: Some(0.0),
            },
        );
        return ConversionResult {
            success: false,
            message: "Cancelled by user.".to_string(),
            output_path: None,
            command_preview: None,
        };
    }

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

fn derive_output_name(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string();
    let re = regex::Regex::new(
        r"(?i)\s*[\(\[]\s*(disc|cd|disk)\s*\d+\s*[\)\]]|\s*[-–—]\s*(disc|cd|disk)\s*\d+\s*$",
    )
    .expect("hardcoded regex is valid");
    let cleaned = re.replace(&stem, "").trim().to_string();
    if cleaned.is_empty() {
        stem
    } else {
        cleaned
    }
}

fn check_cancelled() -> bool {
    CANCEL.load(Ordering::Relaxed)
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

    let mut all_paths: Vec<&Path> = Vec::new();
    all_paths.push(input_path);
    for disc_path in &options.disc_paths {
        let p = Path::new(disc_path);
        if !all_paths.iter().any(|existing| *existing == p) {
            all_paths.push(p);
        }
    }

    for p in &all_paths {
        if !p.exists() {
            return ConversionResult {
                success: false,
                message: format!("Input file not found: {}", p.display()),
                output_path: None,
                command_preview: None,
            };
        }
    }

    let output_folder: &Path = if options.output_folder.trim().is_empty() {
        input_path.parent().unwrap_or_else(|| Path::new("."))
    } else {
        Path::new(&options.output_folder)
    };

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

    if check_cancelled() {
        return ConversionResult {
            success: false,
            message: "Cancelled by user.".to_string(),
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

        let result = match run_with_cancel(&step) {
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

    emit_progress(
        app,
        ConversionProgress {
            current,
            total,
            file_name: file_name.to_string(),
            stage: "preparing".to_string(),
            file_percent: Some(0.1),
        },
    );

    let output_name = derive_output_name(input_path);
    
    let output_path = if options.subfolder_per_game {
        // SUBFOLDER BRANCH (Toggle ON) - Create ONE level
        let subfolder = output_folder.join(&output_name);
        let _ = fs::create_dir_all(&subfolder);
        subfolder
            .join(format!("{}.PBP", output_name))
            .to_string_lossy()
            .to_string()
    } else {
        output_folder
            .join(format!("{}.PBP", output_name))
            .to_string_lossy()
            .to_string()
    };

    let assets = match stage_psp_assets(None, None, None) {
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
    emit_progress(
        app,
        ConversionProgress {
            current,
            total,
            file_name: file_name.to_string(),
            stage: "assets_ready".to_string(),
            file_percent: Some(0.25),
        },
    );

    let is_multi = all_paths.len() > 1;

    let input_for_pipeline: PathBuf;
    if is_multi {
        let m3u_dir = std::env::temp_dir()
            .join("popforge")
            .join("m3u");
        let _ = fs::create_dir_all(&m3u_dir);
        let m3u_path = m3u_dir.join(format!("{}.m3u", &output_name));
        let mut content = String::new();
        for p in &all_paths {
            content.push_str(&format!("{}\n", p.display()));
        }
        if fs::write(&m3u_path, &content).is_err() {
            return ConversionResult {
                success: false,
                message: "Could not create M3U file for multi-disc conversion.".to_string(),
                output_path: Some(output_path.clone()),
                command_preview: None,
            };
        }
        input_for_pipeline = m3u_path;
    } else {
        input_for_pipeline = all_paths[0].to_path_buf();
    }

    let pipeline = build_conversion_pipeline(&input_for_pipeline, &output_path, &output_name, options, &assets);
    let command_preview = pipeline.preview.join("\n");

    emit_progress(
        app,
        ConversionProgress {
            current,
            total,
            file_name: file_name.to_string(),
            stage: "ready".to_string(),
            file_percent: Some(0.35),
        },
    );

    if let Some(missing_tool) = first_missing_tool(&pipeline.required_tools) {
        return ConversionResult {
            success: false,
            message: missing_tool,
            output_path: Some(output_path),
            command_preview: Some(command_preview),
        };
    }

    let display_name = if is_multi {
        format!("{} ({} discs)", file_name, all_paths.len())
    } else {
        file_name.to_string()
    };

    if let Err(message) = run_pipeline(&pipeline.steps, app, current, total, &display_name) {
        return ConversionResult {
            success: false,
            message,
            output_path: Some(output_path.clone()),
            command_preview: Some(command_preview),
        };
    }

    let output_file = Path::new(&output_path);
    if !output_file.exists() {
        return ConversionResult {
            success: false,
            message: format!(
                "PSXPackager finished but no output file was created at: {}",
                output_path
            ),
            output_path: Some(output_path),
            command_preview: Some(command_preview),
        };
    }

    let disc_count = all_paths.len();
    ConversionResult {
        success: true,
        message: if is_multi {
            format!(
                "Created multi-disc EBOOT ({} discs): {}",
                disc_count, output_path
            )
        } else {
            format!("Created EBOOT: {}", output_path)
        },
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
    output_name: &str,
    options: &ConversionOptions,
    assets: &StagedAssets,
) -> ConversionPipeline {
    let mut steps = Vec::new();
    let mut preview = Vec::new();
    let psxpackager_program = resolve_psxpackager_path(options.popstation_path.as_deref())
        .to_string_lossy()
        .to_string();
    let required_tools = vec![ToolRequirement {
        name: "psxpackager",
        program: psxpackager_program.clone(),
    }];

    let output_dir = Path::new(output_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| Path::new(".").to_string_lossy().to_string());

    let output_dir_str = output_dir.clone();

    let psx_args = vec![
        "-i".to_string(),
        input_path.to_string_lossy().to_string(),
        "-o".to_string(),
        output_dir_str,
        "-f".to_string(),
        output_name.to_string(),
        "-l".to_string(),
        options.compression.to_string(),
        "--import".to_string(),
        "--resource-root".to_string(),
        assets.temp_dir.to_string_lossy().to_string(),
        "-x".to_string(),
    ];

    let preview_line = format!(
        "\"{}\" -i \"{}\" -o \"{}\" -f \"{}\" -l {} --import --resource-root \"{}\" -x",
        psxpackager_program,
        input_path.display(),
        output_dir,
        output_name,
        options.compression,
        assets.temp_dir.display()
    );

    steps.push(CommandStep {
        program: psxpackager_program.clone(),
        args: psx_args,
        stage: "psxpackager".to_string(),
    });
    preview.push(preview_line);

    ConversionPipeline {
        steps,
        preview,
        required_tools,
    }
}

fn run_with_cancel(step: &CommandStep) -> Result<std::process::Output, String> {
    if check_cancelled() {
        return Err("Cancelled by user.".to_string());
    }

    let child = Command::new(&step.program)
        .args(&step.args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to start {}: {}", step.program, error))?;

    CHILD_PID.store(child.id() as i32, Ordering::Relaxed);

    let output = child
        .wait_with_output()
        .map_err(|error| format!("Failed to wait for {}: {}", step.program, error))?;

    CHILD_PID.store(0, Ordering::Relaxed);

    if check_cancelled() {
        return Err("Cancelled by user.".to_string());
    }

    Ok(output)
}

fn run_pipeline(
    steps: &[CommandStep],
    app: &tauri::AppHandle,
    current: usize,
    total: usize,
    file_name: &str,
) -> Result<(), String> {
    let step_count = steps.len();
    for (idx, step) in steps.iter().enumerate() {
        if check_cancelled() {
            return Err("Cancelled by user.".to_string());
        }

        ensure_parent_dirs(&step.args)?;

        let file_percent = if step_count > 0 {
            Some(0.35 + ((idx as f32) / step_count as f32) * 0.55)
        } else {
            None
        };

        emit_progress(
            app,
            ConversionProgress {
                current,
                total,
                file_name: file_name.to_string(),
                stage: step.stage.clone(),
                file_percent,
            },
        );

        let output = run_with_cancel(step)?;

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
                d.join("Resources").join("bin").join(&suffixed),
                d.join("../Resources").join(&suffixed),
                d.join("../Resources").join("bin").join(&suffixed),
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

fn run_tool_probe(program: &str, args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new(program).args(args).output()
}

struct StagedAssets {
    temp_dir: PathBuf,
}

fn resolve_resources_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let mut dir = exe.parent().map(|p| p.to_path_buf());

    while let Some(d) = dir {
        for candidate in [
            d.join("Resources").join("bin").join("Resources"),
            d.join("bin").join("Resources"),
            d.join("Resources"),
            d.join("../Resources").join("bin").join("Resources"),
            d.join("../Resources"),
            d.join("../../Resources").join("bin").join("Resources"),
            d.join("../../Resources"),
        ] {
            if candidate.is_dir() {
                let marker = candidate.join("ICON0.PNG");
                if marker.is_file() {
                    return Some(candidate);
                }
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

    let icon0_subdir = temp_dir.join("ICON0");
    let pic0_subdir = temp_dir.join("PIC0");
    let pic1_subdir = temp_dir.join("PIC1");
    fs::create_dir_all(&icon0_subdir)
        .map_err(|error| format!("Could not create {}: {}", icon0_subdir.display(), error))?;
    fs::create_dir_all(&pic0_subdir)
        .map_err(|error| format!("Could not create {}: {}", pic0_subdir.display(), error))?;
    fs::create_dir_all(&pic1_subdir)
        .map_err(|error| format!("Could not create {}: {}", pic1_subdir.display(), error))?;

    stage_single_asset(&resources, "ICON0.PNG", icon0, &icon0_subdir)?;
    stage_single_asset(&resources, "PIC0.PNG", pic0, &pic0_subdir)?;
    stage_single_asset(&resources, "PIC1.PNG", pic1, &pic1_subdir)?;

    Ok(StagedAssets { temp_dir })
}

fn stage_single_asset(
    resources: &Path,
    default_name: &str,
    user_path: Option<&str>,
    dest_dir: &Path,
) -> Result<PathBuf, String> {
    let source = match user_path.map(str::trim).filter(|s| !s.is_empty()) {
        Some(user) => PathBuf::from(user),
        None => resources.join(default_name),
    };

    if !source.is_file() {
        return Err(format!("Asset not found: {}", source.display()));
    }

    let dest = dest_dir.join(default_name);
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
        last_mode: value
            .get("last_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        compression: value
            .get("compression")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8,
        output_template: value
            .get("output_template")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        game_name: value
            .get("game_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        game_id: value
            .get("game_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        window_width: value
            .get("window_width")
            .and_then(|v| v.as_u64())
            .unwrap_or(800) as u32,
        window_height: value
            .get("window_height")
            .and_then(|v| v.as_u64())
            .unwrap_or(600) as u32,
        subfolder_per_game: value
            .get("subfolder_per_game")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
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
        "last_mode": settings.last_mode,
        "compression": settings.compression,
        "output_template": settings.output_template,
        "game_name": settings.game_name,
        "game_id": settings.game_id,
        "window_width": settings.window_width,
        "window_height": settings.window_height,
        "subfolder_per_game": settings.subfolder_per_game,
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

fn parse_serial_from_iso_name(name: &str) -> Option<String> {
    // Matches SCUS_942.44 or SLES_000.03 (with optional ;1 version suffix)
    let name = name.trim_end_matches(";1").trim_end_matches(';');
    let re = regex::Regex::new(r"([A-Z]{4})[-_](\d{3})\.(\d{2})").ok()?;
    let caps = re.captures(name)?;
    Some(format!("{}-{}{}", &caps[1], &caps[2], &caps[3]))
}

fn detect_sector_format(data: &[u8]) -> Option<(usize, usize)> {
    // Returns (sector_size, user_data_offset_within_sector)
    // Try .iso style (2048 byte sectors, user data = whole sector)
    if data.len() > 16 * 2048 + 6 && &data[16 * 2048 + 1..16 * 2048 + 6] == b"CD001" {
        return Some((2048, 0));
    }
    // Try .bin Mode 2 Form 1 (2352 byte sectors, user data at offset 24)
    let bin_pvd = 16 * 2352;
    if data.len() >= bin_pvd + 30 && &data[bin_pvd + 24 + 1..bin_pvd + 24 + 6] == b"CD001" {
        return Some((2352, 24));
    }
    // Try .bin Mode 1 (2352 byte sectors, user data at offset 16)
    if data.len() >= bin_pvd + 22 && &data[bin_pvd + 16 + 1..bin_pvd + 16 + 6] == b"CD001" {
        return Some((2352, 16));
    }
    None
}

fn resolve_bin_from_cue(cue_path: &str) -> Option<String> {
    let content = std::fs::read_to_string(cue_path).ok()?;
    let parent = std::path::Path::new(cue_path).parent()?;
    for line in content.lines() {
        let line = line.trim();
        if line.len() < 6 || !line[..4].eq_ignore_ascii_case("file") || !line[4..].starts_with(' ') {
            continue;
        }
        let rest = line[5..].trim();
        let bin_name = if let Some(q) = rest.strip_prefix('"') {
            q.split('"').next()?
        } else {
            rest.split_whitespace().next()?
        };
        if bin_name.is_empty() {
            continue;
        }
        let bin_path = parent.join(bin_name);
        if bin_path.exists() {
            return Some(bin_path.to_string_lossy().to_string());
        }
    }
    None
}

fn find_companion_cue(bin_path: &str) -> Option<String> {
    let p = std::path::Path::new(bin_path);
    let parent = p.parent()?;
    let stem = p.file_stem()?;
    let stem_str = stem.to_str()?;
    // Try same stem first (e.g. "Game (Track 2).bin" -> "Game (Track 2).cue")
    let same_stem = parent.join(format!("{}.cue", stem_str));
    if same_stem.exists() {
        return Some(same_stem.to_string_lossy().to_string());
    }
    // Fallback: any .cue file in the same directory
    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.extension().map_or(false, |e| e.eq_ignore_ascii_case("cue")) {
                return Some(entry_path.to_string_lossy().to_string());
            }
        }
    }
    None
}

fn extract_serial_from_iso(path: &str) -> Option<String> {
    let data = std::fs::read(path).ok()?;
    let (sector_size, user_data_off) = detect_sector_format(&data)?;

    // PVD user data starts at LBA 16
    let pvd_off = 16 * sector_size + user_data_off;

    // Root directory record is at PVD offset 156
    let root_record_start = pvd_off + 156;
    let root_extent = u32::from_le_bytes([
        data[root_record_start + 2],
        data[root_record_start + 3],
        data[root_record_start + 4],
        data[root_record_start + 5],
    ]) as usize;
    let root_data_len = u32::from_le_bytes([
        data[root_record_start + 10],
        data[root_record_start + 11],
        data[root_record_start + 12],
        data[root_record_start + 13],
    ]) as usize;

    let root_off = root_extent * sector_size + user_data_off;
    let root_end = root_off + root_data_len;
    if root_end > data.len() {
        return None;
    }
    let root_data = &data[root_off..root_end];

    // Walk directory entries in root directory
    let mut pos = 0;
    while pos + 33 < root_data.len() {
        let record_len = root_data[pos] as usize;
        if record_len == 0 {
            break;
        }
        let name_len = root_data[pos + 32] as usize;
        if pos + 33 + name_len > root_data.len() {
            break;
        }
        let name = &data[root_off + pos + 33..root_off + pos + 33 + name_len];
        if let Ok(name_str) = std::str::from_utf8(name) {
            if let Some(serial) = parse_serial_from_iso_name(name_str) {
                return Some(serial);
            }
            if name_str == "SYSTEM.CNF" || name_str == "SYSTEM.CNF;1" {
                let file_extent = u32::from_le_bytes([
                    root_data[pos + 2],
                    root_data[pos + 3],
                    root_data[pos + 4],
                    root_data[pos + 5],
                ]) as usize;
                let file_data_len = u32::from_le_bytes([
                    root_data[pos + 10],
                    root_data[pos + 11],
                    root_data[pos + 12],
                    root_data[pos + 13],
                ]) as usize;
                let file_off = file_extent * sector_size + user_data_off;
                if file_off + file_data_len <= data.len() {
                    let file_data = &data[file_off..file_off + file_data_len];
                    if let Ok(content) = std::str::from_utf8(file_data) {
                        for line in content.lines() {
                            let line = line.trim();
                            if line.len() > 5
                                && line[..4].eq_ignore_ascii_case("boot")
                                && line.as_bytes().get(4) == Some(&b'=')
                            {
                                let val = line[5..].trim();
                                if let Some(serial) = parse_serial_from_iso_name(val) {
                                    return Some(serial);
                                }
                            }
                        }
                    }
                }
                break;
            }
        }
        pos += record_len;
    }

    // Fallback: try the Volume ID from PVD
    if let Ok(vol_id) = std::str::from_utf8(&data[pvd_off + 40..pvd_off + 72]) {
        let vol_id = vol_id.trim_end_matches(' ').trim_end_matches('\0');
        if !vol_id.is_empty() {
            if let Some(serial) = parse_serial_from_iso_name(vol_id) {
                return Some(serial);
            }
        }
    }

    None
}

fn extract_serial_from_file(path: &str) -> Option<String> {
    let resolved = if path.to_lowercase().ends_with(".cue") {
        resolve_bin_from_cue(path).unwrap_or(path.to_string())
    } else {
        path.to_string()
    };

    if let Some(serial) = extract_serial_from_iso(&resolved) {
        return Some(serial);
    }

    // If the file is a .bin and extraction failed (audio track), try companion .cue
    if resolved.to_lowercase().ends_with(".bin") {
        if let Some(cue) = find_companion_cue(&resolved) {
            if let Some(data_bin) = resolve_bin_from_cue(&cue) {
                if data_bin.to_lowercase() != resolved.to_lowercase() {
                    return extract_serial_from_iso(&data_bin);
                }
            }
        }
    }

    None
}

#[tauri::command]
fn extract_serial(filename: String, file_path: Option<String>) -> Option<String> {
    // Try filename regex first (fast)
    if let Some(serial) = extract_serial_from_filename(&filename) {
        return Some(serial);
    }
    // Fall back to reading from disc image contents
    if let Some(path) = file_path {
        if let Some(serial) = extract_serial_from_file(&path) {
            return Some(serial);
        }
    }
    None
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

fn strip_html(html: &str) -> String {
    let tag_re = regex::Regex::new(r"<[^>]+>").expect("hardcoded regex is valid");
    let stripped = tag_re.replace_all(html, " ");
    stripped
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#039;", "'")
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

fn fetch_from_psxdatacenter(serial: &str, title_hint: Option<&str>) -> Option<GameMetadata> {
    let hint_letter = title_hint
        .and_then(|t| t.chars().next())
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_uppercase())
        .unwrap_or('A');

    let mut letters: Vec<char> = ('A'..='Z').collect();
    letters.sort_by_key(|&c| if c == hint_letter { 0 } else { 1 });

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;

    let found = AtomicBool::new(false);
    let result: Mutex<Option<GameMetadata>> = Mutex::new(None);

    let serial = serial.to_string();

    std::thread::scope(|scope| {
        for &letter in &letters {
            let serial = &serial;
            let found = &found;
            let result = &result;

            scope.spawn(move || {
                if found.load(Ordering::Relaxed) {
                    return;
                }

                let url = psxdatacenter_url(serial, letter);
                let agent = ureq::AgentBuilder::new()
                    .timeout(std::time::Duration::from_secs(10))
                    .build();

                let request = agent
                    .get(&url)
                    .set("User-Agent", "PopForge/0.1 (+https://github.com/popforge)")
                    .set("Accept", "text/html");

                if let Ok(resp) = request.call() {
                    if let Ok(html) = resp.into_string() {
                        if let Some((title, region)) = parse_psxdatacenter_page(&html) {
                            let meta = GameMetadata {
                                serial: serial.to_string(),
                                title,
                                region: if region.is_empty() {
                                    region_name_from_serial(serial).to_string()
                                } else {
                                    region
                                },
                            };
                            if let Ok(mut guard) = result.lock() {
                                *guard = Some(meta);
                            }
                            found.store(true, Ordering::Relaxed);
                        }
                    }
                }
            });
        }
    });

    result.into_inner().ok().and_then(|opt| opt)
}

#[tauri::command]
fn scrape_metadata(file_name: String, file_path: Option<String>) -> GameMetadata {
    let title_hint = extract_title_from_filename(&file_name);
    let serial = extract_serial_from_filename(&file_name)
        .or_else(|| file_path.as_deref().and_then(extract_serial_from_file));

    match serial {
        Some(serial) => {
            fetch_from_psxdatacenter(&serial, title_hint.as_deref())
                .unwrap_or_else(|| {
                    let title = title_hint
                        .unwrap_or_else(|| format!("Game {}", serial));
                    GameMetadata {
                        serial: serial.to_string(),
                        title,
                        region: region_name_from_serial(&serial).to_string(),
                    }
                })
        }
        None => GameMetadata {
            serial: String::new(),
            title: title_hint.unwrap_or_else(|| "Unknown title".to_string()),
            region: "Unknown".to_string(),
        },
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[cfg(target_os = "macos")]
fn strip_quarantine(path: &Path) {
    if path.exists() {
        let _ = std::process::Command::new("xattr")
            .arg("-cr")
            .arg(path)
            .output();
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle();
            let settings = read_settings(handle);
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_size(Size::Physical(PhysicalSize {
                    width: settings.window_width,
                    height: settings.window_height,
                }));
                let handle_clone = handle.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::Resized(size) = event {
                        let mut s = read_settings(&handle_clone);
                        s.window_width = size.width;
                        s.window_height = size.height;
                        let _ = write_settings(&handle_clone, &s);
                    }
                });
            }
            #[cfg(target_os = "macos")]
            {
                let psxpackager = resolve_psxpackager_path(None);
                strip_quarantine(&psxpackager);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            test_backend,
            print_file_path,
            get_last_file,
            get_toolchain_status,
            run_conversion,
            cancel_conversion,
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
    fn test_strip_html() {
        assert_eq!(strip_html("<b>Hello</b>"), " Hello ");
        assert_eq!(strip_html("A &amp; B"), "A & B");
        assert_eq!(strip_html("no tags"), "no tags");
    }
}
