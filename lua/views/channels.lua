local Ui = require("ui/ui")
local deviceList = require("device_list")
local widgets = require("ui/widgets")
local View = require("view")

local Channels = View:derive("Channels")

function Channels:new()
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.ui = Ui:new(new)

	new.intrument_list = {}
	for k, v in pairs(deviceList.instruments) do
		table.insert(new.intrument_list, k)
	end
	table.sort(new.intrument_list)

	new.dropdown = widgets.Dropdown:new({ title = "add instrument", list = new.intrument_list, has_state = false })

	return new
end

function Channels:update()
	self.ui:startFrame()
	self.ui.layout:padding()
	self.ui.layout:row(self.w)
	local add_instrument_index = self.dropdown:update(self.ui)

	self.ui.layout:padding(0)

	if add_instrument_index then
		local intrument_name = self.intrument_list[add_instrument_index]

		command.run_and_register(command.newChannel.new(intrument_name))
	end

	for i, v in ipairs(ui_channels) do
		v.widget:update(self.ui, i)
	end

	self.ui:endFrame()
end

function Channels:draw()
	self.ui:draw()
end

return Channels
