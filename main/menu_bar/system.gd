extends MenuButton

# Signals 
signal system_created_grav(desc)
signal system_opened(path)
signal system_closed()
signal system_saved(path)

# Variables
onready var popup = get_popup()
onready var new = PopupMenu.new()
onready var grav = $Gravitational
onready var open = FileDialog.new()
onready var save_as = FileDialog.new()

func _ready():
	# New
	new.set_name("New")
	new.add_item("Gravitational")
	grav.connect("created", self, "_on_grav_created")
	new.connect("id_pressed", self, "_on_new_item_pressed")
	
	popup.add_child(new)
	popup.add_submenu_item("New", "New")
	
	# Open
	open.access = FileDialog.ACCESS_FILESYSTEM
	open.mode = FileDialog.MODE_OPEN_FILE
	open.resizable = true
	open.connect("file_selected", self, "_on_file_opened");
	open.set_filters(PoolStringArray(["*.cesystem ; CE System Files"]))
	add_child(open)
	
	popup.add_item("Open")
	
	# Close 
	popup.add_item("Close")
	popup.set_item_disabled(2, true)
	
	popup.add_separator()
	
	popup.add_item("Save")
	popup.set_item_disabled(4, true)
	
	popup.add_item("Save As")
	popup.set_item_disabled(5, true)
	
	save_as.access = FileDialog.ACCESS_FILESYSTEM
	save_as.mode = FileDialog.MODE_SAVE_FILE
	save_as.resizable = true
	save_as.connect("file_selected", self, "_on_file_saved");
	save_as.set_filters(PoolStringArray(["*.cesystem ; CE System Files"]))
	add_child(save_as)
	
	# Event Handling
	popup.connect("id_pressed", self, "_on_item_pressed")
	
func on_system_changed(system_tree):
	if system_tree != null:
		popup.set_item_disabled(2, false)
		popup.set_item_disabled(4, system_tree.path == null)
		popup.set_item_disabled(5, false)
	else:
		popup.set_item_disabled(2, true)
		popup.set_item_disabled(4, true)
		popup.set_item_disabled(5, true)
	
func _on_item_pressed(ID):
	match ID:
		1:
			open.popup_centered_minsize(Vector2(300.0, 200.0))
		2:
			emit_signal("system_closed")
		4:
			emit_signal("system_saved", null)
		5:
			save_as.popup_centered_minsize(Vector2(300.0, 200.0))
		_:
			pass
		
func _on_new_item_pressed(ID):
	match ID:
		0: 
			grav.popup_centered()
		_:
			pass
			
func _on_grav_created(desc):
	emit_signal("system_created_grav", desc)

func _on_file_opened(path):
	emit_signal("system_opened", path)
	
func _on_file_saved(path):
	emit_signal("system_saved", path)
