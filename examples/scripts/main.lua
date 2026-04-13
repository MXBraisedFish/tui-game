local M

-- 游戏脚本的初始化 API
function init_game(state)
  -- 判断是否为nil用于确定是继续游戏还是新游戏
  local new_state = state or {}

  -- 如果为空表则进行初始化
  if next(new_state) == nil then
    new_state = {
      -- 是否赢了
      win = false,
      -- 当前步数
      step = 0,
      -- 最小步数，用于历史最佳记录展示
      min_step = nil,
      -- 玩家坐标
      x = 0
    }
  end

  -- 读取最佳记录
  local best_score_table = get_best_score()

  -- 如果读取到了，那就把最小步数设置为历史记录
  if best_score_table ~= nil then
    new_state.min_step = best_score_table.min_step
  end

  -- 读取辅助脚本

  return new_state
end

-- 游戏事件逻辑处理 API
function handle_event(state, event)
  -- 事件判断
  -- 如果是按键动作就进入判断是哪个按键
  if event.type == "action" then
    -- 如果不是退出
    if event.name ~= "quit" then
      -- 游戏逻辑
      state = move(event.name, state)
    else
      -- 如果是退出就请求退出
      request_exit()
    end
    -- 如果是终端变化，那就请求重绘
  elseif event.type == "resize" then
    request_render()
  end

  return state
end

-- 绘制函数
function render(state)
  -- 外框左上角锚点计算
  local x, y = resolve_rect(1, 1, 7, 3)
  -- 绘制一个矩形边框
  canvas_border_rect(x1, y1, 7, 3, {
    top = '═',
    top_right = '╗',
    right = '║',
    bottom_right = '╝',
    bottom = '═',
    bottom_left = '╚',
    left = '║',
    top_left = '╔'
  })

  -- 绘制玩家
  canvas_draw_text(x + 1 + state.x, y + 1, '@', green)

  -- 绘制终点
  -- 只有玩家不在终点时才绘制
  if state.x ~= 4 then
    canvas_draw_text(x + 5, y + 1, '#', yellow)
  end

  -- 绘制记录文字
  -- 获取文字
  local step = translate("game.step") .. " " .. state.step
  local min_step = translate("game.step") .. " " .. state.min_step

  -- 计算文字宽
  local step_width = get_text_width(step)
  local min_step_width = get_text_width(min_step)

  -- 文字定位
  local step_x, step_y = resolve_rect(1, 1, step_width, 1, -1 * (step_width / 2 + 3), -3)
  local min_step_x, min_step_y = resolve_rect(1, 1, min_step_width, 1, min_step_width / 2 + 3, -3)

  -- 文字绘制
  canvas_draw_text(step_x, step_y, step)
  canvas_draw_text(min_step_x, min_step_y, min_step)

  -- 胜利文字
  if state.win then
    -- 获取文字
    local win = translate("game.win")

    -- 计算文字宽
    local win_width = get_text_width(win)

    -- 文字定位
    local win_x, win_y = resolve_rect(1, 1, win_width, 1, win_width, -2)

    -- 文字绘制
    canvas_draw_text(win_x, win_y, win)
  end
end
