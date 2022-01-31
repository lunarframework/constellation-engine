extends Spatial

onready var project_manager = ProjectManager.new()

onready var gui = $GUI

# Called when the node enters the scene tree for the first time.
func _ready():
	OS.set_window_title("Constellation Viewer")


func _on_GUI_opened_dir(dir):
	var error = project_manager.open(dir)
	if project_manager.is_open():
		gui.on_edit_project(project_manager)
		OS.set_window_title(str("Constellation Viewer - " + dir))
	else:
		print(error)


func _on_GUI_closed_dir():
	if project_manager.is_open():
		gui.on_close_project()
		project_manager.close()
		OS.set_window_title("Constellation Viewer")


func _on_GUI_edited_project():
	if project_manager.is_open():
		gui.on_edit_project(project_manager)
