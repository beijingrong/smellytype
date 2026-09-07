-- SmellyType Hammerspoon Integration
--
-- This module provides hotkey support for smellytype on macOS using Hammerspoon.
-- It's an alternative to the built-in rdev hotkey capture that doesn't require
-- granting Accessibility permissions to Terminal.
--
-- Installation:
--   1. Install Hammerspoon: brew install --cask hammerspoon
--   2. Copy this file to ~/.hammerspoon/smellytype.lua
--   3. Add to your ~/.hammerspoon/init.lua:
--      local smellytype = require("smellytype")
--      smellytype.setup({ hotkey = "rightalt" })  -- or your preferred key
--   4. Reload Hammerspoon config (Cmd+Shift+R or click menu bar icon)
--
-- Configuration options:
--   hotkey: The key to use for push-to-talk (default: "rightalt")
--           Common choices: "rightalt", "rightcmd", "f13", "f14", etc.
--   mode: "push_to_talk" (hold to record) or "toggle" (press to start/stop)
--   smellytype_path: Path to smellytype binary (default: auto-detect)

local M = {}

-- Default configuration
M.config = {
    hotkey = "rightalt",
    mode = "push_to_talk",
    smellytype_path = nil,  -- Auto-detect
}

-- State
M.is_recording = false
M.hotkey_binding = nil

-- Find smellytype binary
local function find_smellytype()
    if M.config.smellytype_path then
        return M.config.smellytype_path
    end

    -- Common installation paths
    local paths = {
        "/opt/homebrew/bin/smellytype",
        "/usr/local/bin/smellytype",
        os.getenv("HOME") .. "/.cargo/bin/smellytype",
        os.getenv("HOME") .. "/workspace/smellytype/target/release/smellytype",
    }

    for _, path in ipairs(paths) do
        if hs.fs.attributes(path) then
            return path
        end
    end

    -- Try which
    local handle = io.popen("which smellytype 2>/dev/null")
    if handle then
        local result = handle:read("*a"):gsub("%s+", "")
        handle:close()
        if result ~= "" then
            return result
        end
    end

    return nil
end

-- Execute smellytype command
local function smellytype_cmd(cmd)
    local path = find_smellytype()
    if not path then
        hs.alert.show("smellytype not found!")
        return
    end

    hs.task.new(path, nil, {"record", cmd}):start()
end

-- Start recording
local function start_recording()
    if not M.is_recording then
        M.is_recording = true
        smellytype_cmd("start")
        -- Optional: show visual feedback
        -- hs.alert.show("🎤 Recording...", 0.5)
    end
end

-- Stop recording
local function stop_recording()
    if M.is_recording then
        M.is_recording = false
        smellytype_cmd("stop")
    end
end

-- Toggle recording
local function toggle_recording()
    if M.is_recording then
        stop_recording()
    else
        start_recording()
    end
end

-- Cancel recording
local function cancel_recording()
    if M.is_recording then
        M.is_recording = false
        smellytype_cmd("cancel")
        hs.alert.show("Recording cancelled", 0.5)
    end
end

-- Map key name to Hammerspoon key
local function map_key(key)
    local keymap = {
        rightalt = "rightalt",
        rightoption = "rightalt",
        rightopt = "rightalt",
        leftalt = "alt",
        leftoption = "alt",
        leftopt = "alt",
        rightcmd = "rightcmd",
        rightcommand = "rightcmd",
        leftcmd = "cmd",
        leftcommand = "cmd",
        rightctrl = "rightctrl",
        rightcontrol = "rightctrl",
        leftctrl = "ctrl",
        leftcontrol = "ctrl",
        rightshift = "rightshift",
        leftshift = "shift",
    }

    local lower = key:lower()
    return keymap[lower] or lower
end

-- Setup smellytype hotkey
function M.setup(opts)
    opts = opts or {}

    -- Merge config
    for k, v in pairs(opts) do
        M.config[k] = v
    end

    -- Remove existing binding
    if M.hotkey_binding then
        M.hotkey_binding:delete()
    end

    local key = map_key(M.config.hotkey)

    if M.config.mode == "toggle" then
        -- Toggle mode: single press to start/stop
        M.hotkey_binding = hs.hotkey.bind({}, key, toggle_recording)
    else
        -- Push-to-talk mode: hold to record, release to stop
        M.hotkey_binding = hs.hotkey.bind({}, key, start_recording, stop_recording)
    end

    print("SmellyType: Hotkey '" .. key .. "' bound in " .. M.config.mode .. " mode")
end

-- Add cancel hotkey (optional)
function M.add_cancel_hotkey(mods, key)
    hs.hotkey.bind(mods, key, cancel_recording)
    print("SmellyType: Cancel hotkey bound to " .. table.concat(mods, "+") .. "+" .. key)
end

-- Status check
function M.status()
    local path = find_smellytype()
    if not path then
        return "smellytype not found"
    end

    local handle = io.popen(path .. " status 2>/dev/null")
    if handle then
        local result = handle:read("*a"):gsub("%s+", "")
        handle:close()
        return result
    end
    return "unknown"
end

return M
