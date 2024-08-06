# Voxircle

_Voxircle_ is a GUI application for voxelizing circles, meaning approximating circles by square boxes (a.k.a.
blocks, squares, voxels, pixels). This is useful in Minecraft or other voxel/pixel art applications.

## Usage

Download the latest release for your platform (or build the executable yourself with `cargo build --release`) and run
the program. On the right side of the window you will see a grid (the 'viewport') with a number of light gray boxes.
These represent the output of the algorithm. The green circle is the shape the algorithm tries to approximate. The green
dot in the center is the center of the shape, the dark green and red lines through the center are the x- and y-axes. The
viewport can be navigated by dragging and using control-drag to zoom. Double-clicking on the viewport reset the zoom to
be automatic.

On the right of the window there are a number of options for generating the voxelization, as well viewport settings.
I would recommend playing around with the settings to see what they do. The drop-down 'Algorithm' option allows you to
select a number of algorithms or heuristics, and the options below the algorithm selection are for specifying which
shape should be approximated by the algorithm. Note that unchecking 'circle mode' exposes options for making ellipses
which are on (possibly) tilted axes. The sliders can be dragged, or numbers can be entered directly in the field next to
the slider, or the field can be dragged. Hold shift for greater precision.

Below the generation options are the view options, and below that there is a button to generate. 'Auto generate' is on
by default, which makes it so that the effect of changing the generation options is immediately visible.

Below the viewport are some statistics of the currently visible approximation. The notation '1s16' should be read as '1
stack and 16', as in Minecraft where a stack consists of 64 blocks. The block diameter is how many blocks across (in the
cardinal directions) the generated shape is.

Above the viewport are the layer controls. The central number (0 by default) is the current layer, the numbers to the
left and right are the highest and lowest layers that are stored, the current layer can be increased and decreased with
the buttons next to it. Each layer stores the settings used for the generation of the shape, as well as the shape
itself (which is useful only if auto-generate is off), so multiple shapes can be compared easily.

## Scope

The process of voxelization is that of approximating of particular shape by voxels, via a particular heuristic,
implemented by a particular
algorithm. First we will describe what kind of shapes are possible, then we will list the heuristics.

### Shape

The general name for the shapes that are possible to approximate with _Voxircle_ are tilted
offset [superellipses](https://en.wikipedia.org/wiki/Superellipse) with parameter from 0 up to and including infinity.
This includes:

* "Even" and "odd" circles of possibly noninteger radius (squircle parameter 2). (With "even" and "odd" we follow the
  convention common in the Minecraft building community that circles centered on the corner of a block are "even" and
  circles centered in the
  center of a block are "odd", this refers to their block diameter.)
* Ellipses (squircle parameter 2)
* Diamonds stretched along their diagonals (squircle parameter 1), squares stretched along their sides (squircle
  parameter infinity).
* General [squircles](https://en.wikipedia.org/wiki/Squircle) with parameter from 0 up to and including infinity,
  stretched along their axes.
* The three above as "even" and "odd" shaped, in fact with center anywhere in a block.
* The four above with arbitrary tilt.

The position of the center as well as the tilt are arguably not properties of the shape itself, but more how the shape
is placed in relation to the grid (we assume the grid is fixed by voxel art constraints).

### Heuristics

_Voxircle_ has four different heuristics for how the shape can best be approximated by voxels. They are, in approximate
increasing order of complexity:

1. **Centerpoint**: A box is in the voxelization if and only if its centerpoint is in the shape.
2. **Conservative**: A box is in the voxelization if and only if it has *any* overlap with the shape (in the sense of
   nonempty intersection, where both the box and the shape are considered as closed subsets of the plane).
3. **Contained**:  A box is in the voxelization if and only if it is completely contained in the shape (in the same
   sense as above).
4. **Percentage**: A box is in the voxelization if and only if a certain given percentage of the box is in the shape (in
   the sense of area).

The centerpoint heuristic is very simple. The conservative heuristic is especially good at representing thin shapes
well, for example for squircle parameter close to 0 or for thin ellipses. The contained heuristic is a natural opposite
of the contained heuristic, though I have not thought of a use case for it yet. It seems perhaps that many sensible
heuristics lie somewhere between the contained and conservative variants. The percentage heuristic is also natural in a
sense, but quite difficult to compute. Hence, I have only implemented it for the case of circles with arbitrary center
and radius.

## Algorithms and Proofs

I am currently working on a nicely TeX'ed file containing a description of the algorithm as well as proofs of
correctness. Surprisingly the proofs are not very trivial.
