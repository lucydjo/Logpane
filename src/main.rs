use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use anyhow::Context;

use logpane::app::{Geometry, LogpaneApp, Startup, SETTING_GEOMETRY};
use logpane::core::assets::{AssetPaths, AssetWorker};
use logpane::core::models::{Note, NoteBlock};
use logpane::platform::macos::{hotkey::HotkeyService, window};
use logpane::storage::{db_path, load_state, WriterHandle};

fn main() {
    init_tracing();
    let root = data_root();

    if std::env::args().any(|argument| argument == "--smoke-test") {
        let (sandbox, temporary) = match std::env::var("LOGPANE_DATA_DIR") {
            Ok(custom) if !custom.trim().is_empty() => (PathBuf::from(custom), false),
            _ => (
                std::env::temp_dir().join(format!("logpane-smoke-{}", std::process::id())),
                true,
            ),
        };
        let result = smoke_test(&sandbox);
        if temporary {
            let _ = std::fs::remove_dir_all(&sandbox);
        }
        match result {
            Ok(()) => return,
            Err(error) => {
                eprintln!("smoke test failed: {error:#}");
                std::process::exit(1);
            }
        }
    }

    if let Err(error) = run_app(root) {
        let message = format!("{error:#}");
        eprintln!("logpane: failed to start: {message}");
        #[cfg(target_os = "macos")]
        logpane::platform::macos::alert::show_error("Logpane could not start", &message);
        std::process::exit(1);
    }
}

fn run_app(root: PathBuf) -> anyhow::Result<()> {
    let _lock = logpane::instance::acquire(&root)?;
    let loaded = load_state(&root).context("read the Logpane database")?;
    let saved_geometry = loaded
        .settings
        .get(SETTING_GEOMETRY)
        .and_then(|raw| serde_json::from_str::<Geometry>(raw).ok())
        .unwrap_or_default();
    let (x, y, width, height) = window::clamp_geometry(
        saved_geometry.x,
        saved_geometry.y,
        saved_geometry.width,
        saved_geometry.height,
    );
    let initial_geometry = Geometry {
        x,
        y,
        width,
        height,
    };

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Logpane")
            .with_decorations(false)
            .with_resizable(true)
            .with_transparent(false)
            .with_inner_size([width, height])
            .with_min_inner_size([logpane::MIN_WINDOW_WIDTH, logpane::MIN_WINDOW_HEIGHT])
            .with_position([x, y])
            .with_visible(false),
        run_and_return: false,
        ..Default::default()
    };

    let ctx = eframe::egui::Context::default();
    let hotkey = HotkeyService::start(ctx.clone());
    let force_show = std::env::args().any(|argument| argument == "--show");
    let start_visible = hotkey.conflict || force_show;

    let (writer, write_rx) =
        WriterHandle::spawn(root.clone()).context("start the database writer")?;
    let (asset_tx, asset_rx) = mpsc::channel();
    let asset_worker = AssetWorker::spawn(AssetPaths::new(root.clone()), asset_tx);

    let startup = Startup {
        root,
        loaded,
        initial_geometry,
        writer,
        write_rx,
        asset_worker,
        asset_rx,
        show_requested: hotkey.requested.clone(),
        hotkey_label: hotkey.label.clone(),
        hotkey_conflict: hotkey.conflict,
        start_visible,
    };

    eframe::run_native_ext(
        "Logpane",
        options,
        Some(ctx),
        Box::new(move |cc| Ok(Box::new(LogpaneApp::new(cc, startup)))),
    )
    .map_err(|error| anyhow::anyhow!("{error}"))?;
    drop(hotkey);
    Ok(())
}

fn data_root() -> PathBuf {
    if let Ok(custom) = std::env::var("LOGPANE_DATA_DIR") {
        if !custom.trim().is_empty() {
            return PathBuf::from(custom);
        }
    }
    let base = directories::BaseDirs::new()
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("io.github.lucydjo.logpane")
}

fn smoke_test(root: &Path) -> anyhow::Result<()> {
    let mut state = load_state(root)?;
    if state.notes.is_empty() {
        let (mut writer, _events) = WriterHandle::spawn(root.to_path_buf())?;
        let mut note = Note::new();
        note.blocks = vec![NoteBlock::text("smoke test")];
        note.revision = 1;
        writer.save_note(note);
        let _ = writer.flush(Duration::from_secs(5));
        writer.shutdown();
        state = load_state(root)?;
    }
    if state.notes.is_empty() {
        anyhow::bail!("no note could be persisted");
    }
    println!(
        "smoke test ok: storage={} notes={}",
        db_path(root).display(),
        state.notes.len()
    );
    Ok(())
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("logpane=info,warn"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .try_init();
}
