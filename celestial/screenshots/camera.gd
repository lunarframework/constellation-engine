extends Camera

# Declare member variables here. Examples:
# var a = 2
# var b = "text"

export(float, 0.0, 1000.0) var movement_speed = 1.0
export(float, 0.0, 100.0) var sensitivity = 1.0
export(float, 0.0, 100.0) var roll_speed = 1.0

var mouse_offset = Vector2(0.0, 0.0)

# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.
	
func input(event):
	if event is InputEventMouseMotion:
			mouse_offset = event.relative
		
	
func _process(delta):
	_move(delta)
		
func _move(delta):
	var velocity = Vector3(0.0, 0.0, 0.0)
	
	velocity += (Input.get_action_strength("camera_forward") - Input.get_action_strength("camera_backward")) * -self.transform.basis.z
	velocity += (Input.get_action_strength("camera_strafe_right") - Input.get_action_strength("camera_strafe_left")) * self.transform.basis.x
		# velocity += (Input.get_action_strength("camera_strafe_up") - Input.get_action_strength("camera_strafe_down")) * camera.transform.basis.y
		
	var yaw = 0.0
	var pitch = 0.0
	var roll = 0.0	
	
	if Input.is_action_pressed("camera_look"):
		pitch -= mouse_offset.y * sensitivity * 0.001
		yaw -= mouse_offset.x * sensitivity * 0.001
	roll -= (Input.get_action_strength("camera_roll_cw") - Input.get_action_strength("camera_roll_ccw")) * roll_speed * 0.01
	
	# camera.translate(velocity * delta * movement_speed)
	self.transform.origin += velocity * delta * movement_speed
	#camera.rotate(camera.global_transform.basis.x, pitch)
	#camera.rotate(camera.global_transform.basis.y, yaw)
	#camera.rotate(camera.global_transform.basis.z, roll)
	var x_axis = Vector3(1.0, 0.0, 0.0)
	var y_axis = Vector3(0.0, 1.0, 0.0)
	var z_axis = Vector3(0.0, 0.0, 1.0)
	
	self.rotate_object_local(x_axis, pitch)
	self.rotate_object_local(y_axis, yaw)
	self.rotate_object_local(z_axis, roll)
	
	mouse_offset = Vector2(0.0, 0.0)

