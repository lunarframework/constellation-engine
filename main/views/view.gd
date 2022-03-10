extends Node

var tree
var path

onready var slider = $Time/HBox/HSlider

onready var red_giant = $ViewportContainer/Viewport/RedGiant
onready var blackhole = $ViewportContainer/Viewport/Blackhole
onready var white_dwarf = $ViewportContainer/Viewport/WhiteDwarf

func _ready():
	slider.connect("value_changed", self, "_on_slider_changed")
	
	_on_slider_changed(0.0)
	

func _on_slider_changed(time):
	var positions = tree.positions(time)
	red_giant.translation = positions[0]
	blackhole.translation = positions[1]
	white_dwarf.translation = positions[2]
	

