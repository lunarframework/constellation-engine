extends TabContainer

signal system_selected(system_tree)

onready var system_trees = []
var is_empty = false

onready var empty = preload("res://main/views/empty.tscn")
onready var view = preload("res://main/views/view.tscn")

func _ready():
	self.connect("tab_selected", self, "_on_tab_selected")
	
	is_empty = true
	add_child(empty.instance())
	self.current_tab = 0
	
func add_system(system_tree):
	if system_trees.size() == 0:
		get_child(0).queue_free()
		is_empty = false
	
	var current = system_trees.size()
	system_trees.append(system_tree)
	
	var v = view.instance()
	
	var repeats = 0
	
	var system_name = system_tree.hierarchy.name()
	
	for i in range(0, get_child_count()):
		if get_child(i).name.begins_with(system_name):
			repeats += 1
	
	if repeats == 0:
		v.set_name(system_name)
	else:
		v.set_name(system_name + " [" + str(repeats) + "]")
	
	add_child(v)
	
func get_current():
	if self.current_tab < system_trees.size() and !is_empty:
		return system_trees[self.current_tab]
	else:
		return null

func close_current():
	if self.current_tab < system_trees.size():
		system_trees.remove(self.current_tab)
		get_child(self.current_tab).queue_free()
	if system_trees.size() == 0 and !is_empty:
		is_empty = true
		add_child(empty.instance())
		self.current_tab = 0
		
	if is_empty:
		emit_signal("system_selected", null)
	else:
		self.current_tab -= 1
		if self.current_tab < 0:
			self.current_tab = 0
		elif self.current_tab >= system_trees.size():
			self.current_tab = system_trees.size() - 1
			

func _on_tab_selected(tab: int):
	if !is_empty and tab < system_trees.size():
		emit_signal("system_selected", system_trees[tab])
