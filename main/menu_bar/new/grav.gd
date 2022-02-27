extends WindowDialog

onready var create = $Contents/Buttons/Create
onready var cancel = $Contents/Buttons/Cancel

onready var desc_name = $Contents/Name/Edit

signal created(desc)

func _ready():
	self.connect("about_to_show", self, "_on_about_to_show")
	create.connect("pressed", self, "_on_create")
	cancel.connect("pressed", self, "_on_cancel")
	
func _on_about_to_show():
	# Reset state
	desc_name.text = "Untitled"
	
func _on_create():
	var desc = GravDescriptor.new()
	desc.name = desc_name.text
	
	emit_signal("created", desc)
	self.hide()

func _on_cancel():
	self.hide()
