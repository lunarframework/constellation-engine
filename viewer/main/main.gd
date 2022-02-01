extends Control

onready var project_manager = ProjectManager.new()

onready var menu_bar = $VBox/MenuBar

onready var star_prefab = preload("res://celestial/star/star.tscn")
onready var stars = $VBox/HBox/Viewport/Viewport/Stars
	

func _ready():
	OS.set_window_title("Constellation Viewer")
	
	
func _on_edit_project():
	for star in stars.get_children():
		star.queue_free()
	
	project_manager.spawn_initial_data(star_prefab, stars)
	menu_bar.on_edit_project(project_manager)
	
	
func _on_closed_project():
	menu_bar.on_close_project()
	
	for star in stars.get_children():
		star.queue_free()
	
	
func _on_File_opened_dir(dir):
	var error = project_manager.open(dir)
	if project_manager.is_open():
		_on_edit_project()
		OS.set_window_title(str("Constellation Viewer - " + dir))
	else:
		print(error)


func _on_File_closed_dir():
	if project_manager.is_open():
		_on_closed_project()
		project_manager.close()
	
		OS.set_window_title("Constellation Viewer")


func _on_Project_edited_project():
	if project_manager.is_open():
		_on_edit_project()

	
