extends WindowDialog

onready var start_time = $Contents/VBox/Grid/Start
onready var end_time = $Contents/VBox/Grid/End
onready var iterations = $Contents/VBox/Grid/Iterations
onready var solve = $Contents/VBox/Solve

signal solved(desc)

func _ready():
	self.connect("about_to_show", self, "_on_about_to_show")
	solve.connect("pressed", self, "_on_solved")
	
func _on_about_to_show():
	# Reset state
	start_time.value = 0.0
	end_time.value = 0.0
	iterations.value = 0.0
	
func _on_solved():
	var desc = SolveDescriptor.new()
	desc.start_time = start_time.value
	desc.end_time = end_time.value
	desc.iterations = int(iterations.value)
	emit_signal("solved", desc)
	self.hide()
