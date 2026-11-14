//! Fill a data directory with generic demo notes for the documentation captures.
//!
//! ```bash
//! cargo run --example seed_demo -- /tmp/logpane-demo
//! LOGPANE_DATA_DIR=/tmp/logpane-demo cargo run -- --show
//! ```
//!
//! Everything is synthetic: the text is invented and the pictures are drawn here.

use std::path::Path;
use std::time::Duration;

use chrono::{Duration as Days, Local, NaiveTime, TimeZone, Utc};
use image::{ImageEncoder, Rgba, RgbaImage};
use logpane::app::{SETTING_ACTIVE_NOTE, SETTING_PINNED, SETTING_THEME};
use logpane::core::assets::{import_bytes, AssetPaths};
use logpane::core::models::{Note, NoteBlock};
use logpane::storage::WriterHandle;

const BG: Rgba<u8> = Rgba([0x16, 0x18, 0x1A, 255]);
const GRID: Rgba<u8> = Rgba([0x26, 0x29, 0x2C, 255]);
const BAR_A: Rgba<u8> = Rgba([0x5A, 0x71, 0x87, 255]);
const BAR_B: Rgba<u8> = Rgba([0x8F, 0xA6, 0xB8, 255]);
const WARM: Rgba<u8> = Rgba([0xC9, 0xA9, 0x72, 255]);

fn rect(image: &mut RgbaImage, x: i64, y: i64, w: i64, h: i64, color: Rgba<u8>) {
    for py in y.max(0)..(y + h).min(image.height() as i64) {
        for px in x.max(0)..(x + w).min(image.width() as i64) {
            image.put_pixel(px as u32, py as u32, color);
        }
    }
}

fn line(image: &mut RgbaImage, from: (f32, f32), to: (f32, f32), width: i64, color: Rgba<u8>) {
    let steps = (to.0 - from.0).abs().max((to.1 - from.1).abs()) as usize + 1;
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let x = from.0 + (to.0 - from.0) * t;
        let y = from.1 + (to.1 - from.1) * t;
        rect(
            image,
            x as i64 - width / 2,
            y as i64 - width / 2,
            width,
            width,
            color,
        );
    }
}

fn png(image: &RgbaImage) -> Vec<u8> {
    let mut bytes = std::io::Cursor::new(Vec::new());
    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::Rgba8,
        )
        .expect("encode png");
    bytes.into_inner()
}

/// Bars for the median of each run, a warm line for the tail.
fn latency_chart() -> Vec<u8> {
    let (w, h) = (1600_i64, 620_i64);
    let mut image = RgbaImage::from_pixel(w as u32, h as u32, BG);
    for row in 1..6 {
        rect(&mut image, 70, 60 + row * 84, w - 140, 2, GRID);
    }
    let medians = [182, 176, 190, 204, 231, 198, 262, 244, 221, 286, 305, 271];
    let tails = [240, 228, 251, 300, 362, 281, 404, 371, 330, 452, 498, 430];
    let base = h - 76;
    let slot = (w - 200) / medians.len() as i64;
    let mut previous: Option<(f32, f32)> = None;
    for (index, (median, tail)) in medians.iter().zip(tails).enumerate() {
        let x = 100 + index as i64 * slot;
        let height = *median as i64 * 9 / 10;
        let color = if index % 2 == 0 { BAR_A } else { BAR_B };
        rect(&mut image, x + 14, base - height, slot - 44, height, color);
        let point = (
            (x + slot / 2 - 8) as f32,
            (base - tail as i64 * 9 / 10) as f32,
        );
        if let Some(last) = previous {
            line(&mut image, last, point, 5, WARM);
        }
        rect(
            &mut image,
            point.0 as i64 - 7,
            point.1 as i64 - 7,
            14,
            14,
            WARM,
        );
        previous = Some(point);
    }
    rect(&mut image, 70, base, w - 140, 3, GRID);
    png(&image)
}

/// A week-by-hour grid: darker cells are quiet, warm cells are the failures.
fn heatmap() -> Vec<u8> {
    let (cols, rows, cell, gap) = (24_i64, 7_i64, 52_i64, 8_i64);
    let (w, h) = (80 + cols * (cell + gap), 80 + rows * (cell + gap));
    let mut image = RgbaImage::from_pixel(w as u32, h as u32, BG);
    for row in 0..rows {
        for col in 0..cols {
            // Deterministic pseudo-noise, peaking in working hours.
            let seed = (row * 31 + col * 17 + (row * col) % 7) % 11;
            let busy = (9..19).contains(&col) && row < 5;
            let level = if busy { seed + 4 } else { seed / 3 };
            let color = match level {
                0..=1 => GRID,
                2..=6 => BAR_A,
                7..=11 => BAR_B,
                _ => WARM,
            };
            rect(
                &mut image,
                44 + col * (cell + gap),
                44 + row * (cell + gap),
                cell,
                cell,
                color,
            );
        }
    }
    png(&image)
}

fn at(days_ago: i64, hour: u32, minute: u32) -> chrono::DateTime<Utc> {
    let date = Local::now().date_naive() - Days::days(days_ago);
    let naive = date.and_time(NaiveTime::from_hms_opt(hour, minute, 0).expect("time"));
    Local
        .from_local_datetime(&naive)
        .single()
        .expect("local time")
        .with_timezone(&Utc)
}

fn note(title: &str, days_ago: i64, hour: u32, minute: u32, blocks: Vec<NoteBlock>) -> Note {
    let mut note = Note::new().with_title(title);
    note.blocks = blocks;
    note.normalize();
    note.created_at = at(days_ago, hour.saturating_sub(1), minute);
    note.updated_at = at(days_ago, hour, minute);
    note.revision = 1;
    note
}

fn seed(root: &Path) -> anyhow::Result<()> {
    let paths = AssetPaths::new(root);
    let chart = import_bytes(&paths, &latency_chart())?;
    let grid = import_bytes(&paths, &heatmap())?;

    let latency = note(
        "Latency comparison",
        0,
        9,
        31,
        vec![
            NoteBlock::text(
                "09:15 — Baseline run on the small model. Median sits around 180 ms, with two \
                 slow outliers near the end. #latency #baseline",
            ),
            NoteBlock::image(chart.hash.clone()),
            NoteBlock::text(
                "09:23 — Second run with a longer prompt: same shape, heavier tail.\n\
                 Repro with `bench --runs 50 --prompt long`.\n\n\
                 - [x] warm the cache before timing\n\
                 - [ ] rerun on battery power\n\n\
                 > the tail moved more than the median — look there first.\n\n\
                 https://example.org/runs/418",
            ),
        ],
    );
    let flaky = note(
        "Flaky checkout test",
        0,
        8,
        47,
        vec![
            NoteBlock::text(
                "08:40 — Fails about one run in six, always on the first retry. Failures cluster \
                 in working hours, so it smells like shared state. #ci #flaky",
            ),
            NoteBlock::image(grid.hash.clone()),
            NoteBlock::text(
                "08:47 — Passing locally with `--test-threads 1`.\n- [ ] bisect the fixture order",
            ),
        ],
    );
    let keyboard = note(
        "Keyboard notes",
        1,
        17,
        5,
        vec![NoteBlock::text(
            "# Things worth remembering\n- the overlay keeps the caret where it was\n- search \
             ignores accents: `prefere` finds préféré\n- Esc is the save button",
        )],
    );
    let review = note(
        "Onboarding review",
        1,
        11,
        20,
        vec![NoteBlock::text(
            "11:02 — Step three asks for the same thing twice.\n11:14 — The empty state is the \
             best screen of the flow. #ux\n11:20 — Latency of the first paint is fine on wifi.",
        )],
    );
    let ideas = note(
        "Ideas",
        4,
        22,
        12,
        vec![NoteBlock::text(
            "- export a week of notes at once\n- a quieter light theme\n- [ ] ask three people \
             how they capture screenshots today",
        )],
    );
    let reading = note(
        "Reading list",
        9,
        14,
        0,
        vec![NoteBlock::text(
            "> good tools disappear while you use them\n\nhttps://example.org/essays/tools",
        )],
    );

    let (mut writer, _events) = WriterHandle::spawn(root.to_path_buf())?;
    writer.upsert_asset(chart);
    writer.upsert_asset(grid);
    let pinned = vec![keyboard.id];
    let active = latency.id;
    for note in [latency, flaky, keyboard, review, ideas, reading] {
        writer.save_note(note);
    }
    writer.set_setting(SETTING_PINNED, serde_json::to_string(&pinned)?);
    writer.set_setting(SETTING_ACTIVE_NOTE, active.to_string());
    writer.set_setting(SETTING_THEME, "dark");
    writer
        .flush(Duration::from_secs(5))
        .map_err(|message| anyhow::anyhow!(message))?;
    writer.shutdown();
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let Some(root) = std::env::args().nth(1) else {
        anyhow::bail!("usage: seed_demo <empty data directory>");
    };
    let root = Path::new(&root);
    if logpane::storage::db_path(root).exists() {
        anyhow::bail!("{} already holds a Logpane database", root.display());
    }
    std::fs::create_dir_all(root)?;
    seed(root)?;
    println!("seeded {}", root.display());
    Ok(())
}
