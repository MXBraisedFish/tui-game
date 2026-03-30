GAME_META = {
  name = "example_mod.game_name",
  description = "example_mod.game_description",
  detail = "example_mod.game_detail",
  save = false,
}

local best = {
  label = "Best Time",
  value = "--:--:--",
}

function init_game()
  register_action("confirm", { "enter", "space" }, "Confirm / Exit")
  register_action("back", { "esc", "q" }, "Back")
end

function game_loop()
  clear()
  draw_text(example_util.center_x(translate("example_mod.game_name")), 4, translate("example_mod.game_name"), "yellow", nil)
  draw_text(example_util.center_x(translate("example_mod.demo_hint")), 7, translate("example_mod.demo_hint"), "white", nil)
  draw_text(example_util.center_x(translate("example_mod.demo_controls")), 10, translate("example_mod.demo_controls"), "dark_gray", nil)

  while true do
    local action = get_action_blocking()
    if action == "confirm" or action == "back" then
      mod_log("info", "example mod exited by user action: " .. action)
      break
    end
  end
end

function best_score()
  return best
end
