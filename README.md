<p align="center">
  <img src="unicorn-docs/unicorn.png">
</p>

# Unicorn Console 

[![Build Status](https://travis-ci.org/sunjay/UnicornConsole.svg?branch=master)](https://travis-ci.org/Gigoteur/UnicornConsole)
[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Gigoteur/UnicornConsole/blob/master/LICENSE.md)
[![Gitter](https://img.shields.io/gitter/room/nwjs/nw.js.svg)](https://gitter.im/UnicornConsole/Lobby)


Unicorn Console is a quick and dirty engine that let you do what you want with a resolution of 400x240 pixels. The main engine is not dependant of a specific GFX library so you can use it where you want:
  * [unicorn](https://github.com/Gigoteur/UnicornConsole/tree/master/unicorn): Main engine source code
  * [unicorn-devkit](https://github.com/Gigoteur/UnicornConsole/tree/master/unicorn-devkit): SDL2 version
  * [unicorn-libretro](https://github.com/Gigoteur/UnicornConsole/tree/master/unicorn-libretro): [libretro](http://www.libretro.com/index.php/api/) API version
  * [unicorn-web](https://github.com/Gigoteur/UnicornConsole/tree/master/unicorn-web): Webassembly version
  * [unicorn-android](https://github.com/Gigoteur/UnicornConsole/tree/master/unicorn-android): Android version
  
  
TOC:
  * [Features](#features)
  * [Requirements](#requirements)
  * [Download](#download)
    + [Build](#build)
  * [Create](#create)
  * [API](#api)

## Features

  * Display: 400x240 pixels, 32 bits color
  * Palette: predefined palettes/extend existing one
  * Sprite: 3200 8x8 sprites
  * Dynamic sprite: create/save sprites with all size
  * Map: 400x60 cells, 3200x480 pixels
  * Code: Rust/Javascript/Python/Lua
  * Sound: chiptune support via [klystron](http://kometbomb.github.io/klystrack/) engine 
  * Editor: GFX/SOUND/CODE editor
  

## Download
### Build

Cargo feature:
  * cpython: enable python support
  * unicorn_plugin_lua: enable lua support
  * duktape: enable duktape (javascript) support
  * libksnd: use the native version of klystron for the sound

You can choose to build the main UI to play/edit games:
```
cd unicorn-devkit
cargo build --release
```

and run it with the default embedded game:
```
./target/release/uc-devkit
```

or load an existing one:
```
./target/release/uc-devkit ../unicorn-games/floppybird/floppybird.uni
```

You can also choose to build the libretro version:
```
cd unicorn-libretro
cargo build --release
```

And load the shared library with retroarch:
```
retroarch -L target/release/libunicorn_libretro.so ../unicorn/examples/api_demos.uni
```




## Create

## API

The API is available for Rust/Javascript/Python/Lua.
  * [Graphics](#graphics)
    + [camera](#camera)
    + [circ](#circ)
    + [circfill](#circfill)
    + [clip](#clip)
    + [cls](#cls)
    + [color](#color)
    + [ellipse](#ellipse)
    + [ellipsefill](#ellipsefill)
    + [fget](#fget)
    + [font](#font)
    + [line](#line)
    + [mode](#mode)
    + [pal](#pal)
    + [palt](#palt)
    + [pget](#pget)
    + [print](#print)
    + [pset](#pset)
    + [rect](#rect)
    + [rectfill](#rectfill)
    + [sget](#sget)
    + [spr](#spr)
    + [sset](#sset)
    + [sspr](#sspr)
    + [trigon](#trigon)
  * [MAP](#map)
    + [map](#map)
    + [mget](#mget)
    + [mset](#mset)
  * [Noise](#noise)
    + [noise](#noise)
    + [noise_set_seed](#noise_set_seed)
  * [Math](#math)
  * [Memory](#memory)
  * [Mouse Input](#mouse_input)
  * [Palettes](#palettes)
  * [Cart Data](#cart_data)

### Graphics

#### camera

`camera([x, y])`

Set the camera position.

* _x_/_y_ are the coordinates to set the camera, and they could be optional (in this case, 0/0 will be used)

#### circ

`circ(x, y, r, [col])`

Draw a circle:
*  _x_/_y_ are the coordinates
* _r_ is the radius of the circle
* _col_ is the color of the circle

#### circfill

`circfill(x, y, r, [col])`

Draw a filled circle:
*  _x_/_y_ are the coordinates
* _r_ is the radius of the circle
* _col_ is the color of the circle

[[https://j.gifs.com/nZl3GE.gif]]

#### clip

`clip([x, y, w, h])`

Set a screen clipping region where:
* x/y are the coordinate
* w is the width
* h is the height

#### cls

Clear the screen.

#### color

`color(col)`

set default color

#### ellipse

`ellipse(x, y, rx, ry, [col])`

Draw an ellipse

#### ellipsefill

`ellipsefill(x, y, rx, ry, [col])`

draw filled ellipse

#### fget

`fget(n, [f])`

get values of sprite flags

#### font

`font(name)`

Change the font policy ("pico8", "bbc", "cbmII", "appleII")

#### fset

`fset(n, [f], v)`

set values of sprite flags

#### line

`line(x0, y0, x1, y1, [col])`

draw line

#### pal

`pal(c0, c1)`

Switch the color c0 to color c1.

#### palt

`palt(col, t)`

Set the transparency for color 'col', where 't' is a boolean

#### pget

`pget(x, y)`

Get the pixel color in x/y coordinate

#### print

`print (str, [x, y, [col]])`
[Python: **unicorn_print**]

Display a string on the screen

#### pset

`pset(x, y, col)`

Set the pixel color with the value 'col' in x/y coordinate

#### rect

`rect(x0, y0, x1, y1, [col])`

draw a rectangle

#### rectfill

`rectfill(x0, y0, x1, y1, [col])`

draw filled rectangle

[[https://j.gifs.com/76MGDr.gif]]

#### sget

`sget(x, y)`

get spritesheet pixel colour

#### spr

`spr(n, x, y, [w, h], [flip_x], [flip_y])`

Draw a sprite:
* _n_ is the sprite number
* _x_/_y_ are the coordinate
* _w_ and _h_ specify how many sprites wide to blit and are 1/1 values by default
* _flip_x_ to flip horizontally the sprite
* _flip_y_ to flip vertically the sprite

Color 0 will be transparent by default (see [palt](https://github.com/Gigoteur/PX8/wiki/API-Documentation#palt)


#### sset

`sset(x, y, [col])`

set spritesheet pixel colour

#### sspr

`sspr(sx, sy, sw, sh, dx, dy, [dw, dh], [flip_x], [flip_y])`

draw texture from spritesheet

#### trigon

`trigon(x1, y1, x2, y2, x3, y3, [col])`

draw trigon

### Keyboard Input

#### btn([i, [p]])

get button i state for player p

#### btnp([i, [p]])

only true when the button was not pressed the last frame; repeats every 4 frames after button held for 15 frames

### Map

#### map

`map(cel_x, cel_y, sx, sy, cel_w, cel_h, [layer])`

[Python: **spr_map**]

Draw map; layers from flags; sprite 0 is empty

#### mget

`mget(x, y)`

Get a map value

#### mset

`mset(x, y, v)`

Set a map value

### Noise

#### noise

`noise(x, y, z)`

#### noise_set_seed

`noise_set_seed(x)`

### Math
### Memory [**WIP**]
### Mouse input [**WIP**]
### Palettes [**WIP**]
#### Cart Data [**WIP**]

