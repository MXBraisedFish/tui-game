// 提供“继续游戏”功能的相关查询，将存档系统中的游戏 ID 转换为可显示的游戏名称，并同步主菜单中“继续游戏”项的状态

use crate::app::content_cache; // 查找游戏显示名称
use crate::app::menu::Menu; // 更新菜单项的继续目标
use crate::core::save; // 存档读写（继续存档检查、清除）

// 根据游戏 ID 获取其显示名称，找不到返回 "--"
pub fn resolve_saved_game_name(game_id: &str) -> String {
    if let Some(game) = content_cache::find_game(game_id) {
        return game.display_name;
    }
    "--".to_string()
}

// 解析继续游戏目标：查游戏并验证是否有有效存档，返回游戏 ID 和显示名称元组
pub fn resolve_continue_target(game_id: &str) -> Option<(String, String)> {
    let game = content_cache::find_game(game_id)?;
    if !save::game_has_continue_save(game_id) {
        return None;
    }
    Some((game.id, game.display_name))
}

// 同步主菜单的“继续游戏”项：检查最新存档游戏，更新菜单显示；无存档时清除继续目标
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