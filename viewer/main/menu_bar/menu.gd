extends PanelContainer


onready var file_menu = $HBox/File
onready var project_menu  = $HBox/Project

func on_edit_project(project_manager: ProjectManager):
	file_menu.on_edit_project(project_manager)
	project_menu.on_edit_project(project_manager)
	
func on_view_project(project_manager: ProjectManager):
	pass
	
func on_save_project(project_manager: ProjectManager):
	file_menu.on_save_project(project_manager)
	project_menu.on_save_project(project_manager)
	
func on_close_project():
	file_menu.on_close_project()
	project_menu.on_close_project()
