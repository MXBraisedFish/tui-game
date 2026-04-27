use crate::app::content_cache;
use crate::app::menu::Menu;
use crate::core::save;

/// 根据游戏 ID 解析其显示名称。
pub fn resolve_saved_game_name(game_id: &str) -> String {
    if let Some(game) = content_cache::find_game(game_id) {
        return game.display_name;
    }
    "--".to_string()
}

/// 解析"继续游戏"目标，返回游戏 ID 和显示名称。
pub fn resolve_continue_target(game_id: &str) -> Option<(String, String)> {
    let game = content_cache::find_game(game_id)?;
    if !save::game_has_continue_save(game_id) {
        return None;
    }
    Some((game.id, game.display_name))
}

/// 同步主菜单中的“继续游戏”项。
/// 检查最新存档的游戏，并更新菜单项的显示和目标。
pub fn sync_continue_item(menu: &mut Menu) {
    let Some(game_id) = save::latest_saved_game_id() else {
        menu.set_continue_target(None, None);
        return;
    };

    match resolve_continue_target(&game_id) {
        Some((resolved_id, resolved_name)) => {
            menu.set_continue_target(Some(resolved_id), Some(resolved_name));
        }
        None => {
            let _ = save::clear_active_game_save();
            menu.set_continue_target(None, None);
        }
    }
}