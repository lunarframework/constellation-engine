extends MenuButton

onready var popup = get_popup()

var open_dialog

func _ready():
	popup.add_item("Help")
	popup.connect("id_pressed", self, "_on_item_pressed")
	
func _on_item_pressed(ID):
	match ID:
		0:
			print("hello")
		_:
			pass
		
		

