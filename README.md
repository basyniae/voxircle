# Voxircle

[_Voxircle_](https://github.com/basyniae/voxircle) is a GUI application for voxelizing circles, which is approximating
circles by square boxes (a.k.a. blocks, squares, voxels, pixels). This is useful in Minecraft and other voxel/pixel art
applications.

## Usage

Download the latest release for your platform (or build the executable yourself with `cargo build --release`) and run
it. On the right side of the window you will see a grid (the 'viewport'), on the right you will see some settings.
The light gray boxes in the viewport represent the output of the algorithm. The green circle is the shape the algorithm
tries to approximate. The viewport can be navigated by dragging to pan and using control-drag to zoom. Double-clicking
on the viewport sets the zoom to automatic.

The top half of the settings panel deals with options for generating the voxelization. Below that are viewport settings
as well as a 'generate' button.
I would recommend playing around with the generation and viewport settings to see what they do, for a precise
description of the capabilities see the sections [Scope](#scope) and [Algorithms and Proofs](#algorithms-and-proofs)
below. In short: the drop-down 'Algorithm' option allows you to select a number of algorithms or heuristics, and the
options below the algorithm selection are for specifying which
shape should be approximated by the algorithm. Note also that unchecking 'circle mode' exposes options for making
ellipses
which are on (possibly) tilted axes. The sliders can be dragged, or numbers can be entered directly in the field next to
the slider, or the field can be dragged. Hold shift for greater precision.

Below the generation options are the view options (see [Metrics, Stastistics, and Viewport Options](metrics)), and below
that there is a button to generate. 'Auto generate' is on
by default, which makes it so that the effect of changing the generation options is immediately visible.

Above the viewport are the layer controls. The central number (0 by default) is the current layer, the numbers to the
left and right are the highest and lowest layers that are stored, the current layer can be increased and decreased with
the buttons next to it. Each layer stores the settings used for the generation of the shape, as well as the generated
voxelization
itself (which is useful only if auto-generate is off), so multiple shapes can be compared easily.

## Scope

The process of voxelization is that of approximating of particular shape by voxels, via a particular heuristic,
implemented by a particular
algorithm. First we will describe what kind of shapes are possible, then we will list the heuristics. Finally there will
be a description of the stastics.

### Shape

The general name for the shapes that are possible to approximate with _Voxircle_ are tilted
offset (with respect to the standard grid) [superellipses](https://en.wikipedia.org/wiki/Superellipse) with parameter
from 0 up to and including infinity.
This includes:

* "Even" and "odd" circles of possibly noninteger radius (squircle parameter 2). (With "even" and "odd" we follow the
  convention common in the Minecraft building community that circles centered on the corner of a block are "even" and
  circles centered in the
  center of a block are "odd", this refers to their block diameter.)
* [Ellipses](https://en.wikipedia.org/wiki/Ellipse) (squircle parameter 2)
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

The centerpoint heuristic is very simple, but may generate a voxelization that is not connect for thin shapes. The
conservative heuristic is especially good at representing thin shapes
well, for example for squircle parameter close to 0 or for thin ellipses, see the introduction of
the [GPU Gems 2 chapter](https://developer.nvidia.com/gpugems/gpugems2/part-v-image-oriented-computing/chapter-42-conservative-rasterization)
on this topic (we don't use any of the methods described there). The contained heuristic is a natural opposite
of the contained heuristic, though I have not thought of a use case for it yet. It seems perhaps that many sensible
heuristics lie somewhere between the contained and conservative variants. The percentage heuristic is also natural in a
sense, but quite difficult to compute. Hence, I have only implemented it for the case of circles with arbitrary center
and radius.

#### Generation of 3D shapes

Some care should be taking when approximating 3D shapes as a stack of layers of superellipses. (A cone, say, can be
viewed as a stack of circles of decreasing radii.) The centerpoint method works as expected, since it is samples the
shape at a single point. For the conservative algorithm however, we lose the nice property of working well with thin
shapes. Suppose we want to generate a [helicoidal surface](https://en.wikipedia.org/wiki/Helicoid) without holes.
Horizontal slices of this surface are straight lines rotating at a constant rate, so lets approximate them by very thin
ellipses (say minor radius 0.01, major radius 10.0). The conservative method will give a nice connected straight line
for every layer, but the full 3D voxelization is spiky.

A similar situation occurs when making a hollow shapes using the thin boundary of each layer. This boundary only knows
about the 2D horizontal slice so will not include enough blocks for a hole-less appearance from the outside.

### <a name=metrics>Metrics, Stastistics, and Viewport Options</a>

So far only one basic voxel-related stastic of the generated voxelization is computed.
Namely checking the 'boundary' checkbox shows the so-called thin boundary of the generated shape.
Below the viewport are some statistics of the currently visible approximation. The notation '1s16' should be read as '1
stack and 16', as in Minecraft where a stack consists of 64 blocks. The block diameter is how many blocks across (in the
cardinal directions) the generated shape is.

## Algorithms and Proofs

I am currently working on a nicely TeX'ed file containing a description of the algorithm as well as proofs of
correctness. Surprisingly the proofs are not very trivial.
