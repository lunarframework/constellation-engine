extends PanelContainer

signal system_created_grav(desc)
signal system_opened(path)
signal system_closed()
signal system_saved(path)

onready var system_menu = $Padding/HBox/System

func _ready():
	system_menu.connect("system_created_grav", self, "_on_system_created_grav")
	system_menu.connect("system_opened", self, "_on_system_opened")
	system_menu.connect("system_closed", self, "_on_system_closed")
	system_menu.connect("system_saved", self, "_on_system_saved")

func on_system_changed(system_tree):
	system_menu.on_system_changed(system_tree)

func _on_system_created_grav(desc):
	emit_signal("system_created_grav", desc)
	
func _on_system_opened(path):
	emit_signal("system_opened", path)

func _on_system_closed():
	emit_signal("system_closed")
	
func _on_system_saved(path):
	emit_signal("system_saved", path)

