extends PanelContainer

onready var system_manager = SystemManager.new()
onready var system_trees = []
	
onready var menu_bar = $VBox/MenuBar
onready var views = $VBox/Docks/HBox/Center/Views

func _ready():
	OS.set_window_title("Constellation Engine")
	
	menu_bar.connect("system_created_grav", self, "_on_system_created_grav")
	menu_bar.connect("system_opened", self, "_on_system_opened")
	menu_bar.connect("system_closed", self, "_on_system_closed")
	menu_bar.connect("system_saved", self, "_on_system_saved")
	
	views.connect("system_selected", self, "_on_system_selected")
	
func on_system_changed(system_tree):
	menu_bar.on_system_changed(system_tree)
	
func _on_system_created_grav(desc):
	print("Creating gravitational system with name: ", desc.name)
	var hierarchy = system_manager.create_grav(desc)
	
	var tree = SystemContext.new(hierarchy)
	
	views.add_system(tree)
	
func _on_system_opened(path):
	print("Loading System from path: ", path)
	
	var hierarchy = system_manager.load(path)
	if !hierarchy.is_empty():
		var tree = SystemContext.new(hierarchy)
		tree.path = path
		views.add_system(tree)
			
func _on_system_closed():
	views.close_current()

func _on_system_saved(path):
	var current = views.get_current()
	if current != null:
		if path != null:
			current.path = path
		
		if current.path:
			system_manager.save(current.path, current.hierarchy)
			
func _on_system_selected(tree):
	on_system_changed(tree)
