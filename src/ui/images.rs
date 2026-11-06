//! Thumbnail texture cache. Decoding happens lazily, once per asset, from the stored thumbnail.
//!
//! Textures are the largest GPU allocation of the app, so the cache keeps about `MAX_TEXTURES`
//! renditions, least-recently-used first out. Only images on screen are requested, and a texture
//! drawn in the current frame is never evicted: otherwise more on-screen images than the limit
//! would be decoded again on every frame.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use eframe::egui::{self, TextureHandle};

use crate::core::assets::AssetRecord;

const MAX_TEXTURES: usize = 12;

#[derive(Default)]
pub struct ImageCache {
    textures: HashMap<String, TextureHandle>,
    order: VecDeque<String>,
    failed: HashSet<String>,
    used_this_frame: HashSet<String>,
    #[cfg(test)]
    decodes: usize,
}

impl ImageCache {
    /// Whether the stored thumbnail of this asset could not be read.
    pub fn has_failed(&self, hash: &str) -> bool {
        self.failed.contains(hash)
    }

    /// Return the texture for an asset, decoding its stored thumbnail on first use.
    pub fn texture(
        &mut self,
        ctx: &egui::Context,
        root: &Path,
        asset: &AssetRecord,
    ) -> Option<&TextureHandle> {
        if self.failed.contains(&asset.hash) {
            return None;
        }
        self.used_this_frame.insert(asset.hash.clone());
        if self.textures.contains_key(&asset.hash) {
            self.touch(&asset.hash);
            return self.textures.get(&asset.hash);
        }

        // Only the bounded thumbnail is decoded on the UI thread. The original is never decoded
        // here, even if the thumbnail is missing.
        #[cfg(test)]
        {
            self.decodes += 1;
        }
        let path = asset.absolute_thumb(root);
        let decoded = image::open(&path).ok().map(|image| image.to_rgba8());
        let Some(rgba) = decoded else {
            self.failed.insert(asset.hash.clone());
            return None;
        };

        let size = [rgba.width() as usize, rgba.height() as usize];
        let color = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
        let handle = ctx.load_texture(
            format!("asset-{}", asset.hash),
            color,
            egui::TextureOptions::LINEAR,
        );

        self.order.push_back(asset.hash.clone());
        self.textures.insert(asset.hash.clone(), handle);
        self.textures.get(&asset.hash)
    }

    /// Evict the oldest textures not drawn in this frame. Call once per frame, after all drawing.
    pub fn end_frame(&mut self) {
        let mut index = 0;
        while self.order.len() > MAX_TEXTURES && index < self.order.len() {
            if self.used_this_frame.contains(&self.order[index]) {
                index += 1;
            } else if let Some(hash) = self.order.remove(index) {
                self.textures.remove(&hash);
            }
        }
        self.used_this_frame.clear();
    }

    fn touch(&mut self, hash: &str) {
        self.order.retain(|value| value != hash);
        self.order.push_back(hash.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::assets::{import_bytes, AssetPaths};
    use image::ImageEncoder;

    fn assets(root: &Path, count: u8) -> Vec<AssetRecord> {
        let paths = AssetPaths::new(root);
        (0..count)
            .map(|seed| {
                let image = image::RgbaImage::from_pixel(4, 4, image::Rgba([seed, 1, 2, 255]));
                let mut bytes = std::io::Cursor::new(Vec::new());
                image::codecs::png::PngEncoder::new(&mut bytes)
                    .write_image(image.as_raw(), 4, 4, image::ExtendedColorType::Rgba8)
                    .unwrap();
                import_bytes(&paths, &bytes.into_inner()).unwrap()
            })
            .collect()
    }

    fn draw(cache: &mut ImageCache, ctx: &egui::Context, root: &Path, assets: &[AssetRecord]) {
        for asset in assets {
            assert!(cache.texture(ctx, root, asset).is_some());
        }
        cache.end_frame();
    }

    #[test]
    fn more_images_on_screen_than_the_limit_are_decoded_only_once() {
        let dir = tempfile::tempdir().unwrap();
        let assets = assets(dir.path(), MAX_TEXTURES as u8 + 4);
        let ctx = egui::Context::default();
        let mut cache = ImageCache::default();
        for _frame in 0..3 {
            draw(&mut cache, &ctx, dir.path(), &assets);
        }
        assert_eq!(cache.decodes, assets.len());
    }

    #[test]
    fn images_off_screen_are_evicted_down_to_the_limit() {
        let dir = tempfile::tempdir().unwrap();
        let assets = assets(dir.path(), MAX_TEXTURES as u8 + 4);
        let ctx = egui::Context::default();
        let mut cache = ImageCache::default();
        draw(&mut cache, &ctx, dir.path(), &assets);
        draw(&mut cache, &ctx, dir.path(), &assets[..1]);
        assert_eq!(cache.textures.len(), MAX_TEXTURES);
        assert!(cache.textures.contains_key(&assets[0].hash));
    }

    #[test]
    fn an_unreadable_thumbnail_is_remembered() {
        let dir = tempfile::tempdir().unwrap();
        let assets = assets(dir.path(), 1);
        std::fs::remove_file(assets[0].absolute_thumb(dir.path())).unwrap();
        let ctx = egui::Context::default();
        let mut cache = ImageCache::default();
        assert!(cache.texture(&ctx, dir.path(), &assets[0]).is_none());
        assert!(cache.has_failed(&assets[0].hash));
    }
}
