extends Node

onready var length = $VBox/Grid/LengthOptions
onready var mass = $VBox/Grid/MassOptions
onready var time = $VBox/Grid/TimeOptions

func _ready():
	length.add_item("Meters (m)", 0)
	length.add_item("Kilometers (km)", 1)
	length.add_item("Light Years (ly)", 2)
	length.selected = 0
	
	mass.add_item("Kilograms (kg)", 0)
	mass.selected = 0
	
	time.add_item("Seconds (s)", 0)
	time.selected = 0
