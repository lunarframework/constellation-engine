extends PanelContainer

onready var system_manager = SystemManager.new()
	
onready var menu_bar = $VBox/MenuBar
onready var interface = $VBox/Docks/HBox/Left/Interface
onready var views = $VBox/Docks/HBox/Center/Views

func _ready():
	OS.set_window_title("Constellation Engine")
	
	menu_bar.connect("system_created_grav", self, "_on_system_created_grav")
	menu_bar.connect("system_opened", self, "_on_system_opened")
	menu_bar.connect("system_closed", self, "_on_system_closed")
	menu_bar.connect("system_saved", self, "_on_system_saved")
	
	views.connect("system_selected", self, "_on_system_selected")
	
	
func on_system_changed(tree, path):
	menu_bar.on_system_changed(tree, path)
	interface.on_system_changed(tree, path)
	
func _on_system_created_grav(_desc):
	print("Creating Gravitational System")
	
func _on_system_opened(path):
	print("Loading System from path: ", path)
	
	var tree = system_manager.load(path)
	if !tree.is_none():
		views.add_system(tree, path)
			
func _on_system_closed():
	views.close_current()

func _on_system_saved(path):
	var current = views.get_current()
	if current != null:
		if path != null:
			current[1] = path
		
		if current[1]:
			system_manager.save(current[1], current[0])
			
func _on_system_selected(tree, path):
	on_system_changed(tree, path)
