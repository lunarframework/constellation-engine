extends Control

onready var camera = $Viewport/Camera
	
func _gui_input(event):
	if event.is_action_pressed("camera_focus"):
		grab_focus()
		
	camera.set_focus(has_focus())
	camera.handle_input(event)
	
func _process(delta):
	camera.set_focus(has_focus())
