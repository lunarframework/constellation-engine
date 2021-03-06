extends TabContainer

signal system_selected(tree, path)

var is_empty = false

onready var empty = preload("res://main/views/empty.tscn")
onready var view = preload("res://main/views/view.tscn")

func _ready():
	self.connect("tab_selected", self, "_on_tab_selected")
	
	is_empty = true
	add_child(empty.instance())
	self.current_tab = 0
	
func add_system(tree, path):
	if is_empty:
		get_child(0).queue_free()
		is_empty = false
		
	var current = get_child_count()
	
	var v = view.instance()
	v.tree = tree
	v.path = path
	
	var repeats = 0
	
	var system_name = tree.name()
	
	for i in range(0, get_child_count()):
		if get_child(i).name.begins_with(system_name):
			repeats += 1
	
	if repeats == 0:
		v.set_name(system_name)
	else:
		v.set_name(system_name + " [" + str(repeats) + "]")
	
	add_child(v)
	
func get_current():
	if self.current_tab < get_child_count() and !is_empty:
		var current = get_child(self.current_tab)
		return [current.tree, current.path]
	else:
		return null

func close_current():
	if self.current_tab < get_child_count() and !is_empty:
		get_child(self.current_tab).queue_free()
	if get_child_count() == 0 and !is_empty:
		is_empty = true
		add_child(empty.instance())
		self.current_tab = 0
		
	if is_empty:
		emit_signal("system_selected", null, null)
	else:
		self.current_tab -= 1
		if self.current_tab < 0:
			self.current_tab = 0
		elif self.current_tab >= get_child_count():
			self.current_tab = get_child_count() - 1
			

func _on_tab_selected(tab: int):
	if !is_empty and tab < get_child_count():
		emit_signal("system_selected", get_child(tab).tree, get_child(tab).path)
