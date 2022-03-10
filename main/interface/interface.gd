extends Control

onready var system = $System
onready var config = $Config

onready var config_none = $Config/None
onready var config_some = $Config/Some

onready var system_none = $System/None
onready var system_some = $System/Some

onready var tree = $System/Some/Tree

func _ready():
	config_none.visible = true
	config_some.visible = false
	
	system_none.visible = true
	system_some.visible = false
	
	var root = tree.create_item()
	root.set_text(0, "Gravitational System")
	
	var nbodies = tree.create_item(root)
	nbodies.set_text(0, "NBody System")
	
	var red_giant = tree.create_item(nbodies)
	red_giant.set_text(0, "Red Giant")
	
	var blackhole = tree.create_item(nbodies)
	blackhole.set_text(0, "Blackhole")
	
	var white_dwarf = tree.create_item(nbodies)
	white_dwarf.set_text(0, "White Dwarf")
	
func on_system_changed(tree, path):
	if tree:
		config_none.visible = false
		config_some.visible = true
	
		system_none.visible = false
		system_some.visible = true
	else:
		config_none.visible = true
		config_some.visible = false
	
		system_none.visible = true
		system_some.visible = false
	
	
