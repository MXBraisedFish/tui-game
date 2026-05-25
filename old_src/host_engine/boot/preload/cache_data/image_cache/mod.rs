//! 游戏包图片预渲染缓存
// TODO: 迁移至 storage::CacheStore

mod hash;
mod raster;
mod spec;
mod types;
mod writer;

use std::path::Path;

use crate::host_engine::boot::preload::game_modules::GameModuleRegistry;

use types::GameImageSlot;

type ImageCacheResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 同步所有游戏包 icon/banner 图片缓存。
pub fn sync_image_cache(
    game_module_registry: &GameModuleRegistry,
    image_cache_dir: &Path,
) -> ImageCacheResult<()> {
    for game_module in &game_module_registry.games {
        sync_single_image(
            image_cache_dir,
            game_module.uid.as_str(),
            &game_module.root_dir,
            &game_module.package.icon,
            GameImageSlot::Icon,
        )?;
        sync_single_image(
            image_cache_dir,
            game_module.uid.as_str(),
            &game_module.root_dir,
            &game_module.package.banner,
            GameImageSlot::Banner,
        )?;
    }

    Ok(())
}

fn sync_single_image(
    image_cache_dir: &Path,
    uid: &str,
    package_root_dir: &Path,
    image_value: &serde_json::Value,
    slot: GameImageSlot,
) -> ImageCacheResult<()> {
    let Some(image_spec) = spec::parse_image_spec(package_root_dir, image_value) else {
        return Ok(());
    };

    let source_hash = hash::hash_file(&image_spec.absolute_path)?;
    let cache_path = writer::cache_file_path(image_cache_dir, uid, slot);
    if writer::has_fresh_cache(
        &cache_path,
        image_spec.relative_path.as_str(),
        source_hash.as_str(),
        image_spec.color_mode,
        slot,
    ) {
        return Ok(());
    }

    let dynamic_image = image::open(&image_spec.absolute_path)
        .map_err(|error| spec::invalid_image_error(&image_spec.absolute_path, error))?;
    let lines = raster::render_image_to_ascii(&dynamic_image, slot, image_spec.color_mode);
    writer::write_cache(
        &cache_path,
        image_spec.relative_path,
        source_hash,
        image_spec.color_mode,
        slot,
        lines,
    )?;

    Ok(())
}
