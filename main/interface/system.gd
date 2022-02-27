extends Node

onready var some = $Some
onready var none = $None

func _ready():
	some.visible = false;
	none.visible = true;
