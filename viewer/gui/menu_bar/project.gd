extends MenuButton

signal edited_project()

onready var popup = get_popup()

onready var views = PopupMenu.new()

func _ready():
	
	views.set_name("Views")
	
	popup.add_item("Settings")
	popup.add_separator()
	popup.add_item("Edit Initial Data")
	popup.add_submenu_item("Views", "Views")
	popup.add_child(views)

func on_edit_project(project_manager: ProjectManager):
	self.disabled = false
	
func on_save_project(project_manager: ProjectManager):
	pass
	
func on_close_project():
	self.disabled = true
