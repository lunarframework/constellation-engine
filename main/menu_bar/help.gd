extends MenuButton

onready var popup = get_popup()

onready var about = $About

func _ready():
	popup.add_item("About")
	popup.connect("id_pressed", self, "_on_item_pressed")
	
func _on_item_pressed(ID):
	match ID:
		0:
			about.popup()
		_:
			pass
		
		

