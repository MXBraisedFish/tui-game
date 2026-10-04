-- The methods used to create and run a game session.
local Session = loader.require("session.lua")
-- The handler for game input and other incoming events.
local input = loader.require("input.lua")
-- The drawing functions for the game screen.
local view = loader.require("view.lua")
-- The tools for reading and saving the best results.
local records = loader.require("records.lua")
-- The current game session.
local session

-- Create the game session and request its translations.
function Init(ctx)
  session = Session.new(ctx)
  -- The request number used to recognize the translation loading result.
  session.i18n_request_id = i18n.create()
end

-- Pass an incoming event to the game input handler.
function HandleEvent(event)
  input.handle(session, event)
end

-- Advance the game by the elapsed time in seconds.
function Update(dt)
  session:update(dt)
end

-- Leave the game unchanged during extra frame updates.
function UpdateFrame(dt, alpha)
end

-- Draw the current game screen.
function Render()
  view.render(session)
end

-- Return the best results in the format used for saving.
function SaveBest()
  return records.serialize(session and session.best or records.load(nil))
end
