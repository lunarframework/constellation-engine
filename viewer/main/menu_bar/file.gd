extends MenuButton

signal opened_dir(dir)
signal closed_dir()

onready var popup = get_popup()

var open_dialog

func _ready():
	
	popup.add_item("Open")
	popup.add_item("Close")
	popup.set_item_disabled(1, true)
	popup.connect("id_pressed", self, "_on_item_pressed")
	
	open_dialog = FileDialog.new()
	open_dialog.access = FileDialog.ACCESS_FILESYSTEM
	open_dialog.mode = FileDialog.MODE_OPEN_DIR
	open_dialog.resizable = true
	open_dialog.connect("dir_selected", self, "_on_dir_opened");
	add_child(open_dialog)
	
func on_edit_project(project_manager: ProjectManager):
	popup.set_item_disabled(1, false)
	
func on_save_project(project_manager: ProjectManager):
	pass
	
func on_close_project():
	popup.set_item_disabled(1, true)
	
	
func _on_item_pressed(ID):
	match ID:
		0:
			open_dialog.popup_centered_minsize(Vector2(300.0, 200.0))
		1:
			emit_signal("closed_dir")
		_:
			pass
		
		
func _on_dir_opened(dir):
	emit_signal("opened_dir", dir)
	
	
