extends MenuButton

onready var popup = get_popup()

var open_dialog

func _ready():
	popup.add_item("Open")
	popup.connect("id_pressed", self, "_on_item_pressed")
	
	open_dialog = FileDialog.new()
	open_dialog.access = FileDialog.ACCESS_FILESYSTEM
	open_dialog.mode = FileDialog.MODE_OPEN_FILE
	open_dialog.resizable = true
	open_dialog.connect("file_selected", self, "_on_file_opened");
	add_child(open_dialog)
	
func _on_item_pressed(ID):
	match ID:
		0:
			open_dialog.popup_centered_minsize(Vector2(300.0, 200.0))
		_:
			pass
		
		

func _on_file_opened(path):
	print("Opened " + path)
	pass
