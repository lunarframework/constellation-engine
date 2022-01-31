extends Control

signal opened_dir(dir)
signal closed_dir()
signal edited_project()

onready var menu_bar = $Background/MenuBar

func on_edit_project(project_manager: ProjectManager):
	menu_bar.on_edit_project(project_manager)
	
func on_view_project(project_manager: ProjectManager):
	menu_bar.on_view_project(project_manager)
	
func on_save_project(project_manager: ProjectManager):
	menu_bar.on_save_project(project_manager)
	
func on_close_project():
	menu_bar.on_close_project()
	

func _on_File_opened_dir(dir):
	emit_signal("opened_dir", dir)


func _on_File_closed_dir():
	emit_signal("closed_dir")


func _on_Project_edited_project():
	emit_signal("edited_project")
