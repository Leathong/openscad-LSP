module children(index) {}

module echo(msgn) {}

module import(file, center=false, dpi=96, convexity=1) {}

/**

Creates a cube in the first octant. When center is true, the cube is
centered on the origin. Argument names are optional if given in the
order shown here.

```scad
cube(size = [x,y,z], center = true/false);
cube(size =  x ,     center = true/false);
```

**parameters**:

**size**

single value, cube with all sides this length

3 value array [x,y,z], cube with dimensions x, y and z.

**center**

**false** (default), 1st (positive) octant, one corner at (0,0,0)

**true**, cube is centered at (0,0,0)

```scad
default values:  cube();   yields:  cube(size = [1, 1, 1], center = false);
```

**examples**:

![](https://upload.wikimedia.org/wikipedia/commons/thumb/5/55/OpenSCAD_example_Cube.jpg/150px-OpenSCAD_example_Cube.jpg)

```scad
// equivalent scripts for this example
cube(size = 18);
cube(18);
cube([18,18,18]);

cube(18,false);
cube([18,18,18],false);
cube([18,18,18],center=false);
cube(size = [18,18,18], center = false);
cube(center = false,size = [18,18,18] );
```

![](https://upload.wikimedia.org/wikipedia/commons/thumb/2/29/OpenSCAD_example_Box.jpg/150px-OpenSCAD_example_Box.jpg)

```scad
// equivalent scripts for this example
cube([18,28,8],true);
box=[18,28,8];cube(box,true);
```
*/
module cube(size, center) {}

/**

Creates a cylinder or cone centered about the z axis. When center is
true, it is also centered vertically along the z axis.

Parameter names are optional if given in the order shown here. If a
parameter is named, all following parameters must also be named.

NOTES:

The 2nd & 3rd positional parameters are r1 & r2, if r, d, d1 or d2 are
used they must be named.

Using r1 & r2 or d1 & d2 with either value of zero will make a cone
shape, a non-zero non-equal value will produce a section of a cone (a
<a href="https://en.wikipedia.org/wiki/Frustum" class="extiw" title="w:Frustum">Conical Frustum</a>). r1 & d1 define the base width,
at [0,0,0], and r2 & d2 define the top width.

```scad
cylinder(h = height, r1 = BottomRadius, r2 = TopRadius, center = true/false);
```

**Parameters**

**h** : height of the cylinder or cone

**r**  : radius of cylinder. r1 = r2 = r.

**r1** : radius, bottom of cone.

**r2** : radius, top of cone.

**d**  : diameter of cylinder. r1 = r2 = d / 2. [Note: Requires version
2014.03]

**d1** : diameter, bottom of cone. r1 = d1 / 2. [Note: Requires version
2014.03]

**d2** : diameter, top of cone. r2 = d2 / 2. [Note: Requires version
2014.03]

**center**

**false** (default), z ranges from 0 to h

**true**, z ranges from -h/2 to +h/2

**$fa** : minimum angle (in degrees) of each fragment.

**$fs** : minimum circumferential length of each fragment.

**$fn** : **fixed** number of fragments in 360 degrees. Values of 3 or
more override $fa and $fs

$fa, $fs and $fn must be named parameters. [click here for more
details,](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Other_Language_Features "OpenSCAD User Manual/Other Language Features").

```scad
defaults: cylinder();  yields: cylinder($fn = 0, $fa = 12, $fs = 2, h = 1, r1 = 1, r2 = 1, center = false);
```

![](https://upload.wikimedia.org/wikipedia/commons/thumb/d/d1/OpenSCAD_Cone_15x10x20.jpg/200px-OpenSCAD_Cone_15x10x20.jpg)

```scad
// equivalent scripts
cylinder(h=15, r1=9.5, r2=19.5, center=false);
cylinder(  15,    9.5,    19.5, false);
cylinder(  15,    9.5,    19.5);
cylinder(  15,    9.5, d2=39  );
cylinder(  15, d1=19,  d2=39  );
cylinder(  15, d1=19,  r2=19.5);
```

![](https://upload.wikimedia.org/wikipedia/commons/thumb/2/24/OpenSCAD_Cone_15x10x0.jpg/200px-OpenSCAD_Cone_15x10x0.jpg)

```scad
// equivalent scripts
cylinder(h=15, r1=10, r2=0, center=true);
cylinder(  15,    10,    0,        true);
cylinder(h=15, d1=20, d2=0, center=true);
```

- ![](https://upload.wikimedia.org/wikipedia/commons/thumb/f/fa/OpenSCAD_Cylinder_20x10_false.jpg/112px-OpenSCAD_Cylinder_20x10_false.jpg)

center = false

- ![](https://upload.wikimedia.org/wikipedia/commons/thumb/d/dc/OpenSCAD_Cylinder_20x10_true.jpg/100px-OpenSCAD_Cylinder_20x10_true.jpg)

center = true

<!-- -->

```scad
// equivalent scripts
cylinder(h=20, r=10, center=true);
cylinder(  20,   10, 10,true);
cylinder(  20, d=20, center=true);
cylinder(  20,r1=10, d2=20, center=true);
cylinder(  20,r1=10, d2=2*10, center=true);
```

**use of $fn**

Larger values of $fn create smoother, more circular, surfaces at the
cost of longer rendering time. Some use medium values during development
for the faster rendering, then change to a larger value for the final F6
rendering.

However, use of small values can produce some interesting non circular
objects. A few examples are show here:

- ![](https://upload.wikimedia.org/wikipedia/commons/thumb/9/95/3_sided_fiqure.jpg/120px-3_sided_fiqure.jpg)

- ![](https://upload.wikimedia.org/wikipedia/commons/thumb/2/24/4_sided_pyramid.jpg/120px-4_sided_pyramid.jpg)

- ![](https://upload.wikimedia.org/wikipedia/commons/thumb/9/93/4_sided_part_pyramid.jpg/120px-4_sided_part_pyramid.jpg)

<!-- -->

```scad
scripts for these examples
cylinder(20,20,20,$fn=3);
cylinder(20,20,00,$fn=4);
cylinder(20,20,10,$fn=4);
```

**undersized holes**

Using cylinder() with difference() to place holes in objects creates
undersized holes. This is because circular paths are approximated with
polygons inscribed within in a circle. The points of the polygon are on
the circle, but straight lines between are inside. To have all of the
hole larger than the true circle, the polygon must lie wholly outside of
the circle (circumscribed). [Modules for circumscribed
holes](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/undersized_circular_objects "OpenSCAD User Manual/undersized circular objects")

- ![](https://upload.wikimedia.org/wikipedia/commons/thumb/8/85/OpenSCAD_Under_size_hole.jpg/120px-OpenSCAD_Under_size_hole.jpg)

<!-- -->

```scad
script for this example
poly_n = 6;
color("blue") translate([0, 0, 0.02]) linear_extrude(0.1) circle(10, $fn=poly_n);
color("green") translate([0, 0, 0.01]) linear_extrude(0.1) circle(10, $fn=360);
color("purple") linear_extrude(0.1) circle(10/cos(180/poly_n), $fn=poly_n);
```
*/
module cylinder(r) {}

/**

A polyhedron is the most general 3D primitive solid. It can be used to
create any regular or irregular shape including those with concave as
well as convex features. Curved surfaces are approximated by a series of
flat surfaces.

```scad
polyhedron( points = [ [X0, Y0, Z0], [X1, Y1, Z1], ... ], triangles = [ [P0, P1, P2], ... ], convexity = N);   // before 2014.03
polyhedron( points = [ [X0, Y0, Z0], [X1, Y1, Z1], ... ], faces = [ [P0, P1, P2, P3, ...], ... ], convexity = N);   // 2014.03 & later
```

**Parameters**

**points**

Vector of 3d points or vertices. Each point is in turn a vector,
[x,y,z], of its coordinates.

Points may be defined in any order. N points are referenced, in the
order defined, as 0 to N-1.

**triangles** [*Deprecated: **triangles** will be removed in future
releases. Use **faces** parameter instead*]

Vector of faces that collectively enclose the solid. Each face is a
vector containing the indices (0 based) of 3 points from the points
vector.

**faces** [Note: Requires version 2014.03]

Vector of faces that collectively enclose the solid. Each face is a
vector containing the indices (0 based) of 3 or more points from the
points vector.

Faces may be defined in any order. Define enough faces to fully enclose
the solid, with no overlap.

If points that describe a single face are not on the same plane, the
face is automatically split into triangles as needed.

**convexity**

Integer. The convexity parameter specifies the maximum number of faces a
ray intersecting the object might penetrate. This parameter is needed
only for correct display of the object in OpenCSG preview mode. It has
no effect on the polyhedron rendering. For display problems, setting it
to 10 should work fine for most cases.

```scad
default values: polyhedron(); yields: polyhedron(points = undef, faces = undef, convexity = 1);
```

It is arbitrary which point you start with, but all faces must have
points ordered in **clockwise** direction when looking at each face from
outside **inward**. The back is viewed from the back, the bottom from
the bottom, etc. Another way to remember this ordering requirement is to
use the right-hand rule. Using your right-hand, stick your thumb up and
curl your fingers as if giving the thumbs-up sign, point your thumb into
the face, and order the points in the direction your fingers curl. Try
this on the example below.

**Example 1** Using polyhedron to generate cube( [ 10, 7, 5 ] );

![](https://upload.wikimedia.org/wikipedia/commons/b/b1/Cube_numbers.jpg)

point numbers for cube

![](https://upload.wikimedia.org/wikipedia/commons/d/d0/Cube_flat.jpg)

unfolded cube faces

```scad
CubePoints = [
[  0,  0,  0 ],  //0
[ 10,  0,  0 ],  //1
[ 10,  7,  0 ],  //2
[  0,  7,  0 ],  //3
[  0,  0,  5 ],  //4
[ 10,  0,  5 ],  //5
[ 10,  7,  5 ],  //6
[  0,  7,  5 ]]; //7
  
CubeFaces = [
[0,1,2,3],  // bottom
[4,5,1,0],  // front
[7,6,5,4],  // top
[5,6,2,1],  // right
[6,7,3,2],  // back
[7,4,0,3]]; // left
  
polyhedron( CubePoints, CubeFaces );
```

```scad
equivalent descriptions of the bottom face
[0,1,2,3],
[0,1,2,3,0],
[1,2,3,0],
[2,3,0,1],
[3,0,1,2],
[0,1,2],[2,3,0],   // 2 triangles with no overlap
[1,2,3],[3,0,1],
[1,2,3],[0,1,3],
```

**Example 2** A square base pyramid:

![](https://upload.wikimedia.org/wikipedia/commons/d/db/Openscad-polyhedron-squarebasepyramid.png)

A simple polyhedron, square base pyramid

```scad
polyhedron(
points=[ [10,10,0],[10,-10,0],[-10,-10,0],[-10,10,0], // the four points at base
           [0,0,10]  ],                                 // the apex point 
faces=[ [0,1,4],[1,2,4],[2,3,4],[3,0,4],              // each triangle side
              [1,0,3],[2,1,3] ]                         // two triangles for square base
);
```

**Example 3** A triangular prism:

![](https://upload.wikimedia.org/wikipedia/commons/thumb/7/7e/Polyhedron_Prism.png/600px-Polyhedron_Prism.png)

<a href="/wiki/File:Polyhedron_Prism.png" class="internal" title="Enlarge"></a>

A polyhedron triangular prism

```scad
   module prism(l, w, h){
       polyhedron(
               points=[[0,0,0], [l,0,0], [l,w,0], [0,w,0], [0,w,h], [l,w,h]],
               faces=[[0,1,2,3],[5,4,3,2],[0,4,5,1],[0,3,4],[5,2,1]]
               );
       
       // preview unfolded (do not include in your function
       z = 0.08;
       separation = 2;
       border = .2;
       translate([0,w+separation,0])
           cube([l,w,z]);
       translate([0,w+separation+w+border,0])
           cube([l,h,z]);
       translate([0,w+separation+w+border+h+border,0])
           cube([l,sqrt(w*w+h*h),z]);
       translate([l+border,w+separation+w+border+h+border,0])
           polyhedron(
                   points=[[0,0,0],[h,0,0],[0,sqrt(w*w+h*h),0], [0,0,z],[h,0,z],[0,sqrt(w*w+h*h),z]],
                   faces=[[0,1,2], [3,5,4], [0,3,4,1], [1,4,5,2], [2,5,3,0]]
                   );
       translate([0-border,w+separation+w+border+h+border,0])
           polyhedron(
                   points=[[0,0,0],[0-h,0,0],[0,sqrt(w*w+h*h),0], [0,0,z],[0-h,0,z],[0,sqrt(w*w+h*h),z]],
                   faces=[[1,0,2],[5,3,4],[0,1,4,3],[1,2,5,4],[2,0,3,5]]
                   );
       }
   
   prism(10, 5, 3);
```

#### Debugging polyhedra

------------------------------------------------------------------------

Mistakes in defining polyhedra include not having all faces in clockwise
order, overlap of faces and missing faces or portions of faces. As a
general rule, the polyhedron faces should also satisfy manifold
conditions:

-   exactly two faces should meet at any polyhedron edge.
-   if two faces have a vertex in common, they should be in the same
    cycle face-edge around the vertex.

The first rule eliminates polyhedra like two cubes with a common edge
and not watertight models; the second excludes polyhedra like two cubes
with a common vertex.

When viewed from the outside, the points describing each face must be in
the same clockwise order, and provides a mechanism for detecting
counterclockwise. When the thrown together view (F12) is used with F5,
CCW faces are shown in pink. Reorder the points for incorrect faces.
Rotate the object to view all faces. The pink view can be turned off
with F10.

OpenSCAD allows, temporarily, commenting out part of the face
descriptions so that only the remaining faces are displayed. Use // to
comment out the rest of the line. Use /\* and \*\/ to start and end a
comment block. This can be part of a line or extend over several lines.
Viewing only part of the faces can be helpful in determining the right
points for an individual face. Note that a solid is not shown, only the
faces. If using F12, all faces have one pink side. Commenting some faces
helps also to show any internal face.

![](https://upload.wikimedia.org/wikipedia/commons/9/9e/Cube_2_face.jpg)

example 1 showing only 2 faces

```scad
CubeFaces = [
/* [0,1,2,3],  // bottom
   [4,5,1,0],  // front *\/
   [7,6,5,4],  // top
/* [5,6,2,1],  // right
   [6,7,3,2],  // back *\/
   [7,4,0,3]]; // left
```

After defining a polyhedron, its preview may seem correct. The
polyhedron alone may even render fine. However, to be sure it is a valid
manifold and that it can generate a valid STL file, union it with any
cube and render it (F6). If the polyhedron disappears, it means that it
is not correct. Revise the winding order of all faces and the two rules
stated above.

#### Mis-ordered faces

------------------------------------------------------------------------

**Example 4** a more complex polyhedron with mis-ordered faces

When you select 'Thrown together' from the view menu and **compile** the
design (**not** compile and render!) the preview shows the mis-oriented
polygons highlighted. Unfortunately this highlighting is not possible in
the OpenCSG preview mode because it would interfere with the way the
OpenCSG preview mode is implemented.)

Below you can see the code and the picture of such a problematic
polyhedron, the bad polygons (faces or compositions of faces) are in
pink.

```scad
// Bad polyhedron
polyhedron
    (points = [
           [0, -10, 60], [0, 10, 60], [0, 10, 0], [0, -10, 0], [60, -10, 60], [60, 10, 60], 
           [10, -10, 50], [10, 10, 50], [10, 10, 30], [10, -10, 30], [30, -10, 50], [30, 10, 50]
           ], 
     faces = [
          [0,2,3],   [0,1,2],  [0,4,5],  [0,5,1],   [5,4,2],  [2,4,3],
                  [6,8,9],  [6,7,8],  [6,10,11], [6,11,7], [10,8,11],
          [10,9,8], [0,3,9],  [9,0,6], [10,6, 0],  [0,4,10],
                  [3,9,10], [3,10,4], [1,7,11],  [1,11,5], [1,7,8],  
                  [1,8,2],  [2,8,11], [2,11,5]
          ]
     );
```

![](https://upload.wikimedia.org/wikipedia/commons/f/f2/Openscad-bad-polyhedron.png)

Polyhedron with badly oriented polygons

A correct polyhedron would be the following:

```scad
polyhedron
    (points = [
           [0, -10, 60], [0, 10, 60], [0, 10, 0], [0, -10, 0], [60, -10, 60], [60, 10, 60], 
           [10, -10, 50], [10, 10, 50], [10, 10, 30], [10, -10, 30], [30, -10, 50], [30, 10, 50]
           ], 
     faces = [
          [0,3,2],  [0,2,1],  [4,0,5],  [5,0,1],  [5,2,4],  [4,2,3],
                  [6,8,9],  [6,7,8],  [6,10,11],[6,11,7], [10,8,11],
          [10,9,8], [3,0,9],  [9,0,6],  [10,6, 0],[0,4,10],
                  [3,9,10], [3,10,4], [1,7,11], [1,11,5], [1,8,7],  
                  [2,8,1],  [8,2,11], [5,11,2]
          ]
     );
```

Beginner's tip  

If you don't really understand "orientation", try to identify the
mis-oriented pink faces and then invert the sequence of the references
to the points vectors until you get it right. E.g. in the above example,
the third triangle (*[0,4,5]*) was wrong and we fixed it as
*[4,0,5]*. Remember that a face list is a circular list. In addition,
you may select "Show Edges" from the "View Menu", print a screen capture
and number both the points and the faces. In our example, the points are
annotated in black and the faces in blue. Turn the object around and
make a second copy from the back if needed. This way you can keep track.

Clockwise Technique  

Orientation is determined by clockwise circular indexing. This means
that if you're looking at the triangle (in this case [4,0,5]) from the
outside you'll see that the path is clockwise around the center of the
face. The winding order [4,0,5] is clockwise and therefore good. The
winding order [0,4,5] is counter-clockwise and therefore bad.
Likewise, any other clockwise order of [4,0,5] works: [5,4,0] &
[0,5,4] are good too. If you use the clockwise technique, you'll
always have your faces outside (outside of OpenSCAD, other programs do
use counter-clockwise as the outside though).

Think of it as a Left Hand Rule:

If you place your left hand on the face with your fingers curled in the
direction of the order of the points, your thumb should point outward.
If your thumb points inward, you need to reverse the winding order.

![](https://upload.wikimedia.org/wikipedia/commons/7/7f/Openscad-bad-polyhedron-annotated.png)

Polyhedron with badly oriented polygons

Succinct description of a 'Polyhedron'

```scad
* Points define all of the points/vertices in the shape.
* Faces is a list of flat polygons that connect up the points/vertices. 
```

Each point, in the point list, is defined with a 3-tuple x,y,z position
specification. Points in the point list are automatically enumerated
starting from zero for use in the faces list (0,1,2,3,... etc).

Each face, in the faces list, is defined by selecting 3 or more of the
points (using the point order number) out of the point list.

e.g. faces=[ [0,1,2] ] defines a triangle from the first point
(points are zero referenced) to the second point and then to the third
point.

When looking at any face from the outside, the face must list all points
in a clockwise order.

#### Point repetitions in a polyhedron point list

The point list of the polyhedron definition may have repetitions. When
two or more points have the same coordinates they are considered the
same polyhedron vertex. So, the following polyhedron:

```scad
points = [[ 0, 0, 0], [10, 0, 0], [ 0,10, 0],
          [ 0, 0, 0], [10, 0, 0], [ 0,10, 0],
          [ 0,10, 0], [10, 0, 0], [ 0, 0,10],
          [ 0, 0, 0], [ 0, 0,10], [10, 0, 0],
          [ 0, 0, 0], [ 0,10, 0], [ 0, 0,10]];
polyhedron(points, [[0,1,2], [3,4,5], [6,7,8], [9,10,11], [12,13,14]]);
```

define the same tetrahedron as:

```scad
points = [[0,0,0], [0,10,0], [10,0,0], [0,0,10]];
polyhedron(points, [[0,2,1], [0,1,3], [1,2,3], [0,3,2]]);
```

Retrieved from
"<https://en.wikibooks.org/w/index.php?title=OpenSCAD_User_Manual/Primitive_Solids&oldid=4040599>"

[Category](https://en.wikibooks.org/wiki/Special:Categories "Special:Categories"):

-   [Book:OpenSCAD User
    Manual](https://en.wikibooks.org/wiki/Category:Book:OpenSCAD_User_Manual "Category:Book:OpenSCAD User Manual")
*/
module polyhedron(points, faces, convexity=1) {}

/**

Creates a sphere at the origin of the coordinate system. The r argument
name is optional. To use d instead of r, d must be named.

**Parameters**

r  
Radius. This is the radius of the sphere. The resolution of the sphere
is based on the size of the sphere and the $fa, $fs and $fn variables.
For more information on these special variables look at:
[OpenSCAD_User_Manual/Other_Language_Features](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Other_Language_Features "OpenSCAD User Manual/Other Language Features")

d  
Diameter. This is the diameter of the sphere.

$fa  
Fragment angle in degrees

$fs  
Fragment size in mm

$fn  
Resolution

<!-- -->

```scad
default values:  sphere();   yields:   sphere($fn = 0, $fa = 12, $fs = 2, r = 1);
```

**Usage Examples**

```scad
sphere(r = 1);
sphere(r = 5);
sphere(r = 10);
sphere(d = 2);
sphere(d = 10);
sphere(d = 20);
```

```scad
// this creates a high resolution sphere with a 2mm radius
sphere(2, $fn=100); 
```

```scad
// also creates a 2mm high resolution sphere but this one 
// does not have as many small triangles on the poles of the sphere
sphere(2, $fa=5, $fs=0.1); 
```

![](https://upload.wikimedia.org/wikipedia/commons/e/ed/OpenSCAD_sphere_in_different_sizes.png)
*/
module sphere(rad) {}

/**

Creates a circle at the origin. All parameters, except r, **must** be
named.

```scad
circle(r=radius | d=diameter);
```

**Parameters**

**r** : circle radius. r name is the only one optional with circle.

circle resolution is based on size, using $fa or $fs.

For a small, high resolution circle you can make a large circle, then
scale it down, or you could set $fn or other special variables. Note:
These examples exceed the resolution of a 3d printer as well as of the
display screen.

```scad
scale([1/100, 1/100, 1/100]) circle(200); // create a high resolution circle with a radius of 2.
circle(2, $fn=50);                        // Another way.
```

**d**  : circle diameter (only available in versions later than
2014.03).

**$fa** : minimum angle (in degrees) of each fragment.

**$fs** : minimum circumferential length of each fragment.

**$fn** : **fixed** number of fragments in 360 degrees. Values of 3 or
more override $fa and $fs.

If they are used, $fa, $fs and $fn must be named parameters. [click here
for more
details,](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Other_Language_Features "OpenSCAD User Manual/Other Language Features").

```scad
defaults:  circle(); yields:  circle($fn = 0, $fa = 12, $fs = 2, r = 1);
```

![](https://upload.wikimedia.org/wikipedia/commons/f/fc/OpenSCAD_Circle_10.jpg)

Equivalent scripts for this example

```scad
circle(10);
circle(r=10);
circle(d=20);
circle(d=2+9*2);
```

#### Ellipses

------------------------------------------------------------------------

An ellipse can be created from a circle by using either `scale()` or
`resize()` to make the x and y dimensions unequal. See [OpenSCAD User
Manual/Transformations](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Transformations "OpenSCAD User Manual/Transformations")

![](https://upload.wikimedia.org/wikipedia/commons/f/f9/OpenScad_Ellipse_from_circle.jpg)
![](https://upload.wikimedia.org/wikipedia/commons/8/8e/OpenScad_Ellipse_from_circle_top_view.jpg)

```scad
// equivalent scripts for this example
resize([30,10])circle(d=20);
scale([1.5,.5])circle(d=20);
```

#### Regular Polygons

------------------------------------------------------------------------

A regular polygon of 3 or more sides can be created by using `circle()`
with $fn set to the number of sides. The following two pieces of code
are equivalent.

```scad
circle(r=1, $fn=4);
```

```scad
module regular_polygon(order = 4, r=1){
     angles=[ for (i = [0:order-1]) i*(360/order) ];
     coords=[ for (th=angles) [r*cos(th), r*sin(th)] ];
     polygon(coords);
}
regular_polygon();
```

These result in the following shapes, where the polygon is inscribed
within the circle with all sides (and angles) equal. One corner points
to the positive x direction. For irregular shapes see the polygon
primitive below.

![](https://upload.wikimedia.org/wikipedia/commons/thumb/f/f8/OpenSCAD_regular_polygon_using_circle.jpg/300px-OpenSCAD_regular_polygon_using_circle.jpg)

```scad
script for these examples
translate([-42,  0]){circle(20,$fn=3);%circle(20,$fn=90);}
translate([  0,  0]) circle(20,$fn=4);
translate([ 42,  0]) circle(20,$fn=5);
translate([-42,-42]) circle(20,$fn=6);
translate([  0,-42]) circle(20,$fn=8);
translate([ 42,-42]) circle(20,$fn=12);
 
color("black"){
     translate([-42,  0,1])text("3",7,,center);
     translate([  0,  0,1])text("4",7,,center);
     translate([ 42,  0,1])text("5",7,,center);
     translate([-42,-42,1])text("6",7,,center);
     translate([  0,-42,1])text("8",7,,center);
     translate([ 42,-42,1])text("12",7,,center);
}
```
*/
module circle(rad) {}

/**

The function polygon() creates a multiple sided shape from a list of x,y
coordinates. A polygon is the most powerful 2D object. It can create
anything that circle and squares can, as well as much more. This
includes irregular shapes with both concave and convex edges. In
addition it can place holes within that shape.

```scad
polygon(points = [ [x, y], ... ], paths = [ [p1, p2, p3..], ...], convexity = N);
```

Parameters

**points**

The list of x,y points of the polygon. : A vector of 2 element vectors.

Note: points are indexed from 0 to n-1.

**paths**

default

If no path is specified, all points are used in the order listed.

single vector

The order to traverse the points. Uses indices from 0 to n-1. May be in
a different order and use all or part, of the points listed.

multiple vectors

Creates primary and secondary shapes. Secondary shapes are subtracted
from the primary shape (like `difference()`). Secondary shapes may be
wholly or partially within the primary shape.

A closed shape is created by returning from the last point specified to
the first.

**convexity**

Integer number of "inward" curves, ie. expected path crossings of an
arbitrary line through the polygon. See below.

```scad
defaults:   polygon();  yields:  polygon(points = undef, paths = undef, convexity = 1);
```

#### Without holes

![](https://upload.wikimedia.org/wikipedia/commons/d/df/OpenSCAD_Polygon_Example_Rhomboid.jpg)

```scad
// equivalent scripts for this example
polygon(points=[[0,0],[100,0],[130,50],[30,50]]);
polygon([[0,0],[100,0],[130,50],[30,50]], paths=[[0,1,2,3]]);
polygon([[0,0],[100,0],[130,50],[30,50]],[[3,2,1,0]]);
polygon([[0,0],[100,0],[130,50],[30,50]],[[1,0,3,2]]);
    
a=[[0,0],[100,0],[130,50],[30,50]];
b=[[3,0,1,2]];
polygon(a);
polygon(a,b);
polygon(a,[[2,3,0,1,2]]);
```

#### One hole

![](https://upload.wikimedia.org/wikipedia/commons/8/80/Openscad-polygon-example1.png)

```scad
// equivalent scripts for this example
polygon(points=[[0,0],[100,0],[0,100],[10,10],[80,10],[10,80]], paths=[[0,1,2],[3,4,5]],convexity=10);

triangle_points =[[0,0],[100,0],[0,100],[10,10],[80,10],[10,80]];
triangle_paths =[[0,1,2],[3,4,5]];
polygon(triangle_points,triangle_paths,10);
```

The 1st path vector, [0,1,2], selects the points,
[0,0],[100,0],[0,100], for the primary shape. The 2nd path vector,
[3,4,5], selects the points, [10,10],[80,10],[10,80], for the
secondary shape. The secondary shape is subtracted from the primary (
think `difference()` ). Since the secondary is wholly within the
primary, it leaves a shape with a hole.

#### Multi hole

[Note: Requires version 2015.03] (for use of `concat()`)

![](https://upload.wikimedia.org/wikipedia/commons/f/f2/OpenSCAD_romboid_with_holes.jpg)

```scad
      //example polygon with multiple holes
a0 = [[0,0],[100,0],[130,50],[30,50]];     // main
b0 = [1,0,3,2];
a1 = [[20,20],[40,20],[30,30]];            // hole 1
b1 = [4,5,6];
a2 = [[50,20],[60,20],[40,30]];            // hole 2
b2 = [7,8,9];
a3 = [[65,10],[80,10],[80,40],[65,40]];    // hole 3
b3 = [10,11,12,13];
a4 = [[98,10],[115,40],[85,40],[85,10]];   // hole 4
b4 = [14,15,16,17];
a  = concat (a0,a1,a2,a3,a4);
b  = [b0,b1,b2,b3,b4];
polygon(a,b);
      //alternate 
polygon(a,[b0,b1,b2,b3,b4]);
```

#### Extruding a 3D shape from a polygon

![](https://upload.wikimedia.org/wikipedia/commons/e/e0/Example_openscad_3dshape.png)

```scad
   translate([0,-20,10]) {
       rotate([90,180,90]) {
           linear_extrude(50) {
               polygon(
                   points = [
                      //x,y
                       /*
                                  O  .
                       *\/
                       [-2.8,0],
                       /*
                                O__X  .
                       *\/
                       [-7.8,0],
                       /*
                              O
                               \
                                X__X  .
                       *\/
                       [-15.3633,10.30],
                       /*
                              X_______._____O
                               \         
                                X__X  .
                       *\/
                       [15.3633,10.30],
                       /*
                              X_______._______X
                               \             /
                                X__X  .     O
                       *\/
                       [7.8,0],
                       /*
                              X_______._______X
                               \             /
                                X__X  .  O__X
                       *\/
                       [2.8,0],
                       /*
                           X__________.__________X
                            \                   /
                             \              O  /
                              \            /  /
                               \          /  /
                                X__X  .  X__X
                       *\/
                       [5.48858,5.3],
                       /*
                           X__________.__________X
                            \                   /
                             \   O__________X  /
                              \            /  /
                               \          /  /
                                X__X  .  X__X
                       *\/
                       [-5.48858,5.3],
                                   ]
                               );
                           }
       }
   }
```

#### convexity

The convexity parameter specifies the maximum number of front sides
(back sides) a ray intersecting the object might penetrate. This
parameter is needed only for correct display of the object in OpenCSG
preview mode and has no effect on the polyhedron rendering.

![](https://upload.wikimedia.org/wikipedia/commons/thumb/0/0c/Openscad_convexity.jpg/400px-Openscad_convexity.jpg)

This image shows a 2D shape with a convexity of 2, as the ray indicated
in red crosses the 2D shapes outside⇒inside (or inside⇒outside) a
maximum of 2 times. The convexity of a 3D shape would be determined in a
similar way. Setting it to 10 should work fine for most cases.
*/
module polygon(pts) {}

/**

Creates a square or rectangle in the first quadrant. When `center` is
true the square is centered on the origin. Argument names are optional
if given in the order shown here.

```scad
square(size = [x, y], center = true/false);
square(size =  x    , center = true/false);
```

**parameters**:

**size**

single value, square with both sides this length

2 value array [x,y], rectangle with dimensions x and y

**center**

**false** (default), 1st (positive) quadrant, one corner at (0,0)

**true**, square is centered at (0,0)

```scad
default values:  square();   yields:  square(size = [1, 1], center = false);
```

**examples**:

![](https://upload.wikimedia.org/wikipedia/commons/d/d3/OpenScad_Square_10_x_10.jpg)

```scad
// equivalent scripts for this example
square(size = 10);
square(10);
square([10,10]);
.
square(10,false);
square([10,10],false);
square([10,10],center=false);
square(size = [10, 10], center = false);
square(center = false,size = [10, 10] );
```

![](https://upload.wikimedia.org/wikipedia/commons/thumb/3/3f/OpenScad_Square_20x10.jpg/150px-OpenScad_Square_20x10.jpg)

```scad
// equivalent scripts for this example
square([20,10],true);
a=[20,10];square(a,true);
```
*/
module square(size) {}

module surface(file, center=false, invert=false, convexity=1) {}

/**
The `text` module creates text as a 2D geometric object, using fonts
installed on the local system or provided as separate font file.

[Note: Requires version 2015.03]

**Parameters**

text  
String. The text to generate.

<!-- -->

size  
Decimal. The generated text has an ascent (height above the baseline) of
approximately the given value. Default is 10. Different fonts can vary
somewhat and may not fill the size specified exactly, typically they
render slightly smaller.

<!-- -->

font  
String. The name of the font that should be used. This is not the name
of the font file, but the logical font name (internally handled by the
fontconfig library). This can also include a style parameter, see below.
A list of installed fonts & styles can be obtained using the font list
dialog (Help -> Font List).

<!-- -->

halign  
String. The horizontal alignment for the text. Possible values are
"left", "center" and "right". Default is "left".

<!-- -->

valign  
String. The vertical alignment for the text. Possible values are "top",
"center", "baseline" and "bottom". Default is "baseline".

<!-- -->

spacing  
Decimal. Factor to increase/decrease the character spacing. The default
value of 1 results in the normal spacing for the font, giving a value
greater than 1 causes the letters to be spaced further apart.

<!-- -->

direction  
String. Direction of the text flow. Possible values are "ltr"
(left-to-right), "rtl" (right-to-left), "ttb" (top-to-bottom) and "btt"
(bottom-to-top). Default is "ltr".

<!-- -->

language  
String. The language of the text. Default is "en".

<!-- -->

script  
String. The script of the text. Default is "latin".

<!-- -->

$fn  
used for subdividing the curved path segments provided by freetype

**Example**

![](https://upload.wikimedia.org/wikipedia/commons/thumb/d/d2/OpenSCAD_text%28%29_example.png/220px-OpenSCAD_text%28%29_example.png)

<a href="/wiki/File:OpenSCAD_text()_example.png" class="internal" title="Enlarge"></a>

Example 1: Result.

```scad
text("OpenSCAD");
```

Notes

To allow specification of particular
<a href="https://en.wikipedia.org/wiki/Unicode" class="extiw" title="w:Unicode">Unicode</a> characters, you can specify them in a
string with the following escape codes;

-   \x*03    * - hex char-value (only hex values from 01 to 7f are
    supported)
-   \u*0123  * - Unicode char with 4 hexadecimal digits (note: lowercase
    \u)
-   \U*012345* - Unicode char with 6 hexadecimal digits (note: uppercase
    \U)

The null character (NUL) is mapped to the space character (SP).

```scad
assert(version() == [2019, 5, 0]);
assert(ord(" ") == 32);
assert(ord("\x00") == 32);
assert(ord("\u0000") == 32);
assert(ord("\U000000") == 32);
```

**Example**

```scad
; // 10 euro and a smilie
```

## Contents

<span class="toctogglespan"></span>

-   [<span class="tocnumber">1</span> <span class="toctext">Using Fonts
    & Styles</span>](#Using_Fonts_&_Styles)
-   [<span class="tocnumber">2</span> <span     class="toctext">Alignment</span>](#Alignment)
    -   [<span class="tocnumber">2.1</span> <span         class="toctext">Vertical alignment</span>](#Vertical_alignment)
    -   [<span class="tocnumber">2.2</span> <span         class="toctext">Horizontal
        alignment</span>](#Horizontal_alignment)
-   [<span class="tocnumber">3</span> <span class="toctext">3D
    text</span>](#3D_text)

### Using Fonts & Styles

Fonts are specified by their logical font name; in addition a style
parameter can be added to select a specific font style like "**bold**"
or "*italic*", such as:



The font list dialog (available under Help > Font List) shows the font
name and the font style for each available font. For reference, the
dialog also displays the location of the font file. You can drag a font
in the font list, into the editor window to use in the text() statement.

![](https://upload.wikimedia.org/wikipedia/commons/thumb/e/e1/OpenSCAD_font_list_dialog.png/400px-OpenSCAD_font_list_dialog.png)

<a href="/wiki/File:OpenSCAD_font_list_dialog.png" class="internal" title="Enlarge"></a>

OpenSCAD font list dialog

OpenSCAD includes the fonts *Liberation Mono*, *Liberation Sans*, and
*Liberation Serif*. Hence, as fonts in general differ by platform type,
use of these included fonts is likely to be portable across platforms.

For common/casual text usage, the specification of one of these fonts is
**recommended** for this reason. Liberation Sans is the default font to
encourage this.

In addition to the installed fonts ( for windows only fonts installed as
admin for all users ), it's possible to add project specific font files.
Supported font file formats are
<a href="https://en.wikipedia.org/wiki/TrueType" class="extiw" title="w:TrueType">TrueType</a> Fonts (\*.ttf) and
<a href="https://en.wikipedia.org/wiki/OpenType" class="extiw" title="w:OpenType">OpenType</a> Fonts (\*.otf). The files need to be
registered with use<>.

```scad
use <ttf/paratype-serif/PTF55F.ttf>
```

After the registration, the font is listed in the font list dialog, so
in case logical name of a font is unknown, it can be looked up as it was
registered.

OpenSCAD uses fontconfig to find and manage fonts, so it's possible to
list the system configured fonts on command line using the fontconfig
tools in a format similar to the GUI dialog.

```scad
$ fc-list -f "%-60{{%{family[0]}%{:style[0]=}}}%{file}\n" | sort

...
Liberation Mono:style=Bold Italic /usr/share/fonts/truetype/liberation2/LiberationMono-BoldItalic.ttf
Liberation Mono:style=Bold        /usr/share/fonts/truetype/liberation2/LiberationMono-Bold.ttf
Liberation Mono:style=Italic      /usr/share/fonts/truetype/liberation2/LiberationMono-Italic.ttf
Liberation Mono:style=Regular     /usr/share/fonts/truetype/liberation2/LiberationMono-Regular.ttf
...
```

Under windows font are in register base. To get a file with the name of
the police use the command line :

`reg query "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts" /s > List_Font_Windows.txt`

**Example**

![](https://upload.wikimedia.org/wikipedia/commons/thumb/b/bb/OpenSCAD_text%28%29_font_style_example.png/220px-OpenSCAD_text%28%29_font_style_example.png)

<a href="/wiki/File:OpenSCAD_text()_font_style_example.png" class="internal" title="Enlarge"></a>

Example 2: Result.

```scad
square(10);
 
translate([15, 15]) {
   text("OpenSCAD", font = "Liberation Sans");
}
 
translate([15, 0]) {
   text("OpenSCAD", font = "Liberation Sans:style=Bold Italic");
}
```

### Alignment

#### Vertical alignment

top  
The text is aligned with the top of the bounding box at the given Y
coordinate.

<!-- -->

center  
The text is aligned with the center of the bounding box at the given Y
coordinate.

<!-- -->

baseline  
The text is aligned with the font baseline at the given Y coordinate.
This is the default.

<!-- -->

bottom  
The text is aligned with the bottom of the bounding box at the given Y
coordinate.

![](https://upload.wikimedia.org/wikipedia/commons/thumb/d/d1/OpenSCAD_text_align_vertical.png/220px-OpenSCAD_text_align_vertical.png)

<a href="/wiki/File:OpenSCAD_text_align_vertical.png" class="internal" title="Enlarge"></a>

OpenSCAD vertical text alignment

```scad
text = "Align";
font = "Liberation Sans";
 
valign = [
   [  0, "top"],
   [ 40, "center"],
   [ 75, "baseline"],
   [110, "bottom"]
];
 
for (a = valign) {
   translate([10, 120 - a[0], 0]) {
     color("red") cube([135, 1, 0.1]);
     color("blue") cube([1, 20, 0.1]);
     linear_extrude(height = 0.5) {
       text(text = str(text,"_",a[1]), font = font, size = 20, valign = a[1]);
     }
   }
}
```

#### Horizontal alignment

left  
The text is aligned with the left side of the bounding box at the given
X coordinate. This is the default.

<!-- -->

center  
The text is aligned with the center of the bounding box at the given X
coordinate.

<!-- -->

right  
The text is aligned with the right of the bounding box at the given X
coordinate.

![](https://upload.wikimedia.org/wikipedia/commons/thumb/9/91/OpenSCAD_text_align_horizontal.png/220px-OpenSCAD_text_align_horizontal.png)

<a href="/wiki/File:OpenSCAD_text_align_horizontal.png" class="internal" title="Enlarge"></a>

OpenSCAD horizontal text alignment

```scad
text = "Align";
font = "Liberation Sans";
 
halign = [
   [10, "left"],
   [50, "center"],
   [90, "right"]
];
 
for (a = halign) {
   translate([140, a[0], 0]) {
     color("red") cube([115, 2,0.1]);
     color("blue") cube([2, 20,0.1]);
     linear_extrude(height = 0.5) {
       text(text = str(text,"_",a[1]), font = font, size = 20, halign = a[1]);
     }
   }
}
```

### 3D text

Text can be changed from a 2 dimensional object into a 3D object by
using the
[linear_extrude](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/2D_to_3D_Extrusion#Linear_Extrude "OpenSCAD User Manual/2D to 3D Extrusion")
function.

```scad
//3d Text Example
linear_extrude(4)
    text("Text");
```

![](https://upload.wikimedia.org/wikipedia/commons/thumb/0/05/Openscad_Text_3dText.jpg/220px-Openscad_Text_3dText.jpg)

<a href="/wiki/File:Openscad_Text_3dText.jpg" class="internal" title="Enlarge"></a>

3D text example

Retrieved from
"<https://en.wikibooks.org/w/index.php?title=OpenSCAD_User_Manual/Text&oldid=4055228>"

[Category](https://en.wikibooks.org/wiki/Special:Categories "Special:Categories"):

-   [Book:OpenSCAD User
    Manual](https://en.wikibooks.org/wiki/Category:Book:OpenSCAD_User_Manual "Category:Book:OpenSCAD User Manual")

*/
module text(args) {}

/**
Displays the child elements using the specified RGB color + alpha value.
This is only used for the F5 preview as CGAL and STL (F6) do not
currently support color. The alpha value defaults to 1.0 (opaque) if not
specified.

#### Function signature:

```scad
color( c = [r, g, b, a] ) { ... }
color( c = [r, g, b], alpha = 1.0 ) { ... }
color( "#hexvalue" ) { ... }
color( "colorname", 1.0 ) { ... }
```

Note that the `r, g, b, a` values are limited to floating point values
in the range **[0,1]** rather than the more traditional integers { 0
... 255 }. However, nothing prevents you to using `R, G, B` values from
{0 ... 255} with appropriate scaling:
`color([ R/255, G/255, B/255 ]) { ... } `

[Note: Requires version 2011.12] Colors can also be defined by name
(case **in**sensitive). For example, to create a red sphere, you can
write `color("red") sphere(5);`. Alpha is specified as an extra
parameter for named colors: `color("Blue",0.5) cube(5);`

[Note: Requires version 2019.05] Hex values can be given in 4 formats,
`#rgb`, `#rgba`, `#rrggbb` and `#rrggbbaa`. If the alpha value is given
in both the hex value and as separate alpha parameter, the alpha
parameter takes precedence.

**Warning:** alpha processing (transparency) is order-sensitive.
Transparent objects must be listed after non-transparent objects to
display them correctly. Some combinations involving multiple transparent
objects cannot be handled correctly. See issue
<a href="https://github.com/openscad/openscad/issues/1390" class="external text">#1390</a>.

The available color names are taken from the World Wide Web consortium's
<a href="http://www.w3.org/TR/css3-color/" class="external text">SVG
color list</a>. A chart of the color names is as follows,  
<span class="small">*(note that both spellings of grey/gray including
slategrey/slategray etc are valid)*</span>:

<table width="100%">
<colgroup>
<col style="width: 20%" />
<col style="width: 20%" />
<col style="width: 20%" />
<col style="width: 20%" />
<col style="width: 20%" />
</colgroup>
<tbody>
<tr class="odd">
<td width="20%"><table width="100%">
<tbody>
<tr class="odd">
<td><strong>Purples</strong></td>
</tr>
<tr class="even">
<td>Lavender</td>
</tr>
<tr class="odd">
<td>Thistle</td>
</tr>
<tr class="even">
<td>Plum</td>
</tr>
<tr class="odd">
<td>Violet</td>
</tr>
<tr class="even">
<td>Orchid</td>
</tr>
<tr class="odd">
<td>Fuchsia</td>
</tr>
<tr class="even">
<td>Magenta</td>
</tr>
<tr class="odd">
<td>MediumOrchid</td>
</tr>
<tr class="even">
<td>MediumPurple</td>
</tr>
<tr class="odd">
<td>BlueViolet</td>
</tr>
<tr class="even">
<td>DarkViolet</td>
</tr>
<tr class="odd">
<td>DarkOrchid</td>
</tr>
<tr class="even">
<td>DarkMagenta</td>
</tr>
<tr class="odd">
<td>Purple</td>
</tr>
<tr class="even">
<td>Indigo</td>
</tr>
<tr class="odd">
<td>DarkSlateBlue</td>
</tr>
<tr class="even">
<td>SlateBlue</td>
</tr>
<tr class="odd">
<td>MediumSlateBlue</td>
</tr>
<tr class="even">
<td><strong>Pinks</strong></td>
</tr>
<tr class="odd">
<td>Pink</td>
</tr>
<tr class="even">
<td>LightPink</td>
</tr>
<tr class="odd">
<td>HotPink</td>
</tr>
<tr class="even">
<td>DeepPink</td>
</tr>
<tr class="odd">
<td>MediumVioletRed</td>
</tr>
<tr class="even">
<td>PaleVioletRed</td>
</tr>
</tbody>
</table></td>
<td width="20%"><table width="100%">
<tbody>
<tr class="odd">
<td><strong>Blues</strong></td>
</tr>
<tr class="even">
<td>Aqua</td>
</tr>
<tr class="odd">
<td>Cyan</td>
</tr>
<tr class="even">
<td>LightCyan</td>
</tr>
<tr class="odd">
<td>PaleTurquoise</td>
</tr>
<tr class="even">
<td>Aquamarine</td>
</tr>
<tr class="odd">
<td>Turquoise</td>
</tr>
<tr class="even">
<td>MediumTurquoise</td>
</tr>
<tr class="odd">
<td>DarkTurquoise</td>
</tr>
<tr class="even">
<td>CadetBlue</td>
</tr>
<tr class="odd">
<td>SteelBlue</td>
</tr>
<tr class="even">
<td>LightSteelBlue</td>
</tr>
<tr class="odd">
<td>PowderBlue</td>
</tr>
<tr class="even">
<td>LightBlue</td>
</tr>
<tr class="odd">
<td>SkyBlue</td>
</tr>
<tr class="even">
<td>LightSkyBlue</td>
</tr>
<tr class="odd">
<td>DeepSkyBlue</td>
</tr>
<tr class="even">
<td>DodgerBlue</td>
</tr>
<tr class="odd">
<td>CornflowerBlue</td>
</tr>
<tr class="even">
<td>RoyalBlue</td>
</tr>
<tr class="odd">
<td>Blue</td>
</tr>
<tr class="even">
<td>MediumBlue</td>
</tr>
<tr class="odd">
<td>DarkBlue</td>
</tr>
<tr class="even">
<td>Navy</td>
</tr>
<tr class="odd">
<td>MidnightBlue</td>
</tr>
<tr class="even">
<td><strong>Reds</strong></td>
</tr>
<tr class="odd">
<td>IndianRed</td>
</tr>
<tr class="even">
<td>LightCoral</td>
</tr>
<tr class="odd">
<td>Salmon</td>
</tr>
<tr class="even">
<td>DarkSalmon</td>
</tr>
<tr class="odd">
<td>LightSalmon</td>
</tr>
<tr class="even">
<td>Red</td>
</tr>
<tr class="odd">
<td>Crimson</td>
</tr>
<tr class="even">
<td>FireBrick</td>
</tr>
<tr class="odd">
<td>DarkRed</td>
</tr>
</tbody>
</table></td>
<td width="20%"><table width="100%">
<tbody>
<tr class="odd">
<td><strong>Greens</strong></td>
</tr>
<tr class="even">
<td>GreenYellow</td>
</tr>
<tr class="odd">
<td>Chartreuse</td>
</tr>
<tr class="even">
<td>LawnGreen</td>
</tr>
<tr class="odd">
<td>Lime</td>
</tr>
<tr class="even">
<td>LimeGreen</td>
</tr>
<tr class="odd">
<td>PaleGreen</td>
</tr>
<tr class="even">
<td>LightGreen</td>
</tr>
<tr class="odd">
<td>MediumSpringGreen</td>
</tr>
<tr class="even">
<td>SpringGreen</td>
</tr>
<tr class="odd">
<td>MediumSeaGreen</td>
</tr>
<tr class="even">
<td>SeaGreen</td>
</tr>
<tr class="odd">
<td>ForestGreen</td>
</tr>
<tr class="even">
<td>Green</td>
</tr>
<tr class="odd">
<td>DarkGreen</td>
</tr>
<tr class="even">
<td>YellowGreen</td>
</tr>
<tr class="odd">
<td>OliveDrab</td>
</tr>
<tr class="even">
<td>Olive</td>
</tr>
<tr class="odd">
<td>DarkOliveGreen</td>
</tr>
<tr class="even">
<td>MediumAquamarine</td>
</tr>
<tr class="odd">
<td>DarkSeaGreen</td>
</tr>
<tr class="even">
<td>LightSeaGreen</td>
</tr>
<tr class="odd">
<td>DarkCyan</td>
</tr>
<tr class="even">
<td>Teal</td>
</tr>
<tr class="odd">
<td><strong>Oranges</strong></td>
</tr>
<tr class="even">
<td>LightSalmon</td>
</tr>
<tr class="odd">
<td>Coral</td>
</tr>
<tr class="even">
<td>Tomato</td>
</tr>
<tr class="odd">
<td>OrangeRed</td>
</tr>
<tr class="even">
<td>DarkOrange</td>
</tr>
<tr class="odd">
<td>Orange</td>
</tr>
</tbody>
</table></td>
<td width="20%"><table width="100%">
<tbody>
<tr class="odd">
<td><strong>Yellows</strong></td>
</tr>
<tr class="even">
<td>Gold</td>
</tr>
<tr class="odd">
<td>Yellow</td>
</tr>
<tr class="even">
<td>LightYellow</td>
</tr>
<tr class="odd">
<td>LemonChiffon</td>
</tr>
<tr class="even">
<td>LightGoldenrodYellow</td>
</tr>
<tr class="odd">
<td>PapayaWhip</td>
</tr>
<tr class="even">
<td>Moccasin</td>
</tr>
<tr class="odd">
<td>PeachPuff</td>
</tr>
<tr class="even">
<td>PaleGoldenrod</td>
</tr>
<tr class="odd">
<td>Khaki</td>
</tr>
<tr class="even">
<td>DarkKhaki</td>
</tr>
<tr class="odd">
<td><strong>Browns</strong></td>
</tr>
<tr class="even">
<td>Cornsilk</td>
</tr>
<tr class="odd">
<td>BlanchedAlmond</td>
</tr>
<tr class="even">
<td>Bisque</td>
</tr>
<tr class="odd">
<td>NavajoWhite</td>
</tr>
<tr class="even">
<td>Wheat</td>
</tr>
<tr class="odd">
<td>BurlyWood</td>
</tr>
<tr class="even">
<td>Tan</td>
</tr>
<tr class="odd">
<td>RosyBrown</td>
</tr>
<tr class="even">
<td>SandyBrown</td>
</tr>
<tr class="odd">
<td>Goldenrod</td>
</tr>
<tr class="even">
<td>DarkGoldenrod</td>
</tr>
<tr class="odd">
<td>Peru</td>
</tr>
<tr class="even">
<td>Chocolate</td>
</tr>
<tr class="odd">
<td>SaddleBrown</td>
</tr>
<tr class="even">
<td>Sienna</td>
</tr>
<tr class="odd">
<td>Brown</td>
</tr>
<tr class="even">
<td>Maroon</td>
</tr>
</tbody>
</table></td>
<td width="20%"><table width="100%">
<tbody>
<tr class="odd">
<td><strong>Whites</strong></td>
</tr>
<tr class="even">
<td>White</td>
</tr>
<tr class="odd">
<td>Snow</td>
</tr>
<tr class="even">
<td>Honeydew</td>
</tr>
<tr class="odd">
<td>MintCream</td>
</tr>
<tr class="even">
<td>Azure</td>
</tr>
<tr class="odd">
<td>AliceBlue</td>
</tr>
<tr class="even">
<td>GhostWhite</td>
</tr>
<tr class="odd">
<td>WhiteSmoke</td>
</tr>
<tr class="even">
<td>Seashell</td>
</tr>
<tr class="odd">
<td>Beige</td>
</tr>
<tr class="even">
<td>OldLace</td>
</tr>
<tr class="odd">
<td>FloralWhite</td>
</tr>
<tr class="even">
<td>Ivory</td>
</tr>
<tr class="odd">
<td>AntiqueWhite</td>
</tr>
<tr class="even">
<td>Linen</td>
</tr>
<tr class="odd">
<td>LavenderBlush</td>
</tr>
<tr class="even">
<td>MistyRose</td>
</tr>
<tr class="odd">
<td><strong>Grays</strong></td>
</tr>
<tr class="even">
<td>Gainsboro</td>
</tr>
<tr class="odd">
<td>LightGrey</td>
</tr>
<tr class="even">
<td>Silver</td>
</tr>
<tr class="odd">
<td>DarkGray</td>
</tr>
<tr class="even">
<td>Gray</td>
</tr>
<tr class="odd">
<td>DimGray</td>
</tr>
<tr class="even">
<td>LightSlateGray</td>
</tr>
<tr class="odd">
<td>SlateGray</td>
</tr>
<tr class="even">
<td>DarkSlateGray</td>
</tr>
<tr class="odd">
<td>Black</td>
</tr>
</tbody>
</table></td>
</tr>
</tbody>
</table>

#### Example

![](https://upload.wikimedia.org/wikipedia/commons/thumb/0/04/Wavy_multicolor_object.jpg/220px-Wavy_multicolor_object.jpg)

<a href="/wiki/File:Wavy_multicolor_object.jpg" class="internal" title="Enlarge"></a>

A 3-D multicolor sine wave

Here's a code fragment that draws a wavy multicolor object

```scad
for(i=[0:36]) {
    for(j=[0:36]) {
      color( [0.5+sin(10*i)/2, 0.5+sin(10*j)/2, 0.5+sin(10*(i+j))/2] )
      translate( [i, j, 0] )
      cube( size = [1, 1, 11+10*cos(10*i)*sin(10*j)] );
    }
}
```

↗ Being that -1<=sin(*x*)<=1 then 0<=(1/2 + sin(*x*)/2)<=1 ,
allowing for the RGB components assigned to color to remain within the
[0,1] interval.

*<span class="small"><a href="https://en.wikipedia.org/wiki/Web_colors" class="external text">Chart based on "Web Colors" from Wikipedia</a></span>*

#### Example 2

In cases where you want to optionally set a color based on a parameter
you can use the following trick:

```scad
module myModule(withColors=false) {
    c=withColors?"red":undef;
    color(c) circle(r=10);
}
```

Setting the colorname to undef keeps the default colors.
*/
module color(c) { /* group */ }

/**
Subtracts the 2nd (and all further) child nodes from the first one
(logical **and not**).  
May be used with either 2D or 3D objects, but don't mix them.

![](https://upload.wikimedia.org/wikipedia/commons/thumb/c/c6/Openscad_difference.jpg/400px-Openscad_difference.jpg)

```scad
Usage example:
difference() {
    cylinder (h = 4, r=1, center = true, $fn=100);
    rotate ([90,0,0]) cylinder (h = 4, r=0.9, center = true, $fn=100);
}
```

**Note:** It is mandatory that surfaces that are to be removed by a
difference operation have an overlap, and that the negative piece being
removed extends fully outside of the volume it is removing that surface
from. Failure to follow this rule can cause [preview
artifacts](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/FAQ#What_are_those_strange_flickering_artifacts_in_the_preview? "OpenSCAD User Manual/FAQ")
and can result in non-manifold render warnings or the removal of pieces
from the render output. See the description above in union for why this
is required and an example of how to do this by this using a small
epsilon value.

##### difference with multiple children

Note, in the second instance, the result of adding a union of the 1st
and 2nd children.

![](https://upload.wikimedia.org/wikipedia/commons/9/9d/Bollean_Difference_3.jpg)

```scad
// Usage example for difference of multiple children:
$fn=90;
difference(){
                                            cylinder(r=5,h=20,center=true);
    rotate([00,140,-45]) color("LightBlue") cylinder(r=2,h=25,center=true);
    rotate([00,40,-50])                     cylinder(r=2,h=30,center=true);
    translate([0,0,-10])rotate([00,40,-50]) cylinder(r=1.4,h=30,center=true);
}
   
// second instance with added union
translate([10,10,0]){
    difference(){
      union(){        // combine 1st and 2nd children
                                                cylinder(r=5,h=20,center=true);
        rotate([00,140,-45]) color("LightBlue") cylinder(r=2,h=25,center=true);
      }
      rotate([00,40,-50])                       cylinder(r=2,h=30,center=true);
      translate([0,0,-10])rotate([00,40,-50])   cylinder(r=1.4,h=30,center=true);
    }
}
```
*/
module difference() { /* group */ }

module group() { /* group */ }

/**
![](https://upload.wikimedia.org/wikipedia/commons/thumb/1/15/Openscad_hull_example_1a.png/200px-Openscad_hull_example_1a.png)

<a href="/wiki/File:Openscad_hull_example_1a.png" class="internal" title="Enlarge"></a>

Two cylinders

![](https://upload.wikimedia.org/wikipedia/commons/thumb/b/b3/Openscad_hull_example_2a.png/200px-Openscad_hull_example_2a.png)

<a href="/wiki/File:Openscad_hull_example_2a.png" class="internal" title="Enlarge"></a>

Convex hull of two cylinders

Displays the <a href="https://www.cgal.org/Manual/latest/doc_html/cgal_manual/Convex_hull_2/Chapter_main.html"
class="external text">convex hull</a> of child nodes.

Usage example:

```scad
hull() {
    translate([15,10,0]) circle(10);
    circle(10);
}
```

The Hull of 2D objects uses their projections (shadows) on the xy plane,
and produces a result on the xy plane. Their Z-height is not used in the
operation.

A note on limitations: Running `hull() { a(); b(); }` is the same as
`hull() { hull() a(); hull() b(); }` so unless you accept/want
`hull() a();` and `hull() b();`, the result will not match expectations.

## Combining transformations

When combining transformations, it is a sequential process, but going
right-to-left. That is

```scad
rotate( ... ) translate ( ... ) cube(5) ;
```

would first move the cube, and then move it in an arc (while also
rotating it by the same angle) at the radius given by the translation.

```scad
translate ( ... ) rotate( ... ) cube(5) ;
```

would first rotate the cube and then move it to the offset defined by
the translate.

![](https://upload.wikimedia.org/wikipedia/commons/thumb/2/25/Openscad_combined_transform.png/200px-Openscad_combined_transform.png)

<a href="/wiki/File:Openscad_combined_transform.png" class="internal" title="Enlarge"></a>

Combine two transforms

```scad
color("red")   translate([0,10,0])  rotate([45,0,0])     cube(5);
color("green") rotate([45,0,0])     translate([0,10,0])  cube(5);
```

Retrieved from
"<https://en.wikibooks.org/w/index.php?title=OpenSCAD_User_Manual/Transformations&oldid=4082820>"

[Category](https://en.wikibooks.org/wiki/Special:Categories "Special:Categories"):

-   [Book:OpenSCAD User
    Manual](https://en.wikibooks.org/wiki/Category:Book:OpenSCAD_User_Manual "Category:Book:OpenSCAD User Manual")
*/
module hull() { /* group */ }

/**
Creates the intersection of all child nodes. This keeps the
**overlapping** portion (logical **and**).  
Only the area which is common or shared by **all** children is
retained.  
May be used with either 2D or 3D objects, but don't mix them.

![](https://upload.wikimedia.org/wikipedia/commons/thumb/f/f8/Openscad_intersection.jpg/400px-Openscad_intersection.jpg)

```scad
//Usage example:
intersection() {
    cylinder (h = 4, r=1, center = true, $fn=100);
    rotate ([90,0,0]) cylinder (h = 4, r=0.9, center = true, $fn=100);
}
```
*/
module intersection() { /* group */ }

/**
Linear Extrusion is an operation that takes a 2D object as input and
generates a 3D object as a result.

In OpenSCAD Extrusion is always performed on the projection (shadow) of
the 2d object xy plane and along the **Z** axis; so if you rotate or
apply other transformations to the 2d object before extrusion, its
shadow shape is what is extruded.

Although the extrusion is linear along the **Z** axis, a twist parameter
is available that causes the object to be rotated around the **Z** axis
as it is extruding upward. This can be used to rotate the object at its
center, as if it is a spiral pillar, or produce a helical extrusion
around the **Z** axis, like a pig's tail.

A scale parameter is also included so that the object can be expanded or
contracted over the extent of the extrusion, allowing extrusions to be
flared inward or outward.

#### Usage

```scad
linear_extrude(height = 5, center = true, convexity = 10, twist = -fanrot, slices = 20, scale = 1.0, $fn = 16) {...}
```

You must use parameter names due to a backward compatibility issue.

`height` must be positive.

`$fn` is optional and specifies the resolution of the linear_extrude
(higher number brings more "smoothness", but more computation time is
needed).

If the extrusion fails for a non-trivial 2D shape, try setting the
convexity parameter (the default is not 10, but 10 is a "good" value to
try). See explanation further down.

#### Twist

Twist is the number of degrees of through which the shape is extruded.
Setting the parameter twist = 360 extrudes through one revolution. The
twist direction follows the left hand rule.

![](https://upload.wikimedia.org/wikipedia/commons/thumb/3/39/Openscad_linext_01.jpg/400px-Openscad_linext_01.jpg)

**0° of Twist**

```scad
linear_extrude(height = 10, center = true, convexity = 10, twist = 0)
translate([2, 0, 0])
circle(r = 1);
```

![](https://upload.wikimedia.org/wikipedia/commons/thumb/e/ee/Openscad_linext_02.jpg/400px-Openscad_linext_02.jpg)

**-100° of Twist**

```scad
linear_extrude(height = 10, center = true, convexity = 10, twist = -100)
translate([2, 0, 0])
circle(r = 1);
```

![](https://upload.wikimedia.org/wikipedia/commons/thumb/f/f3/Openscad_linext_03.jpg/400px-Openscad_linext_03.jpg)

**100° of Twist**

```scad
linear_extrude(height = 10, center = true, convexity = 10, twist = 100)
translate([2, 0, 0])
circle(r = 1);
```

![](https://upload.wikimedia.org/wikipedia/commons/thumb/3/3d/Spring_100x20_in_OpenSCAD.gif/220px-Spring_100x20_in_OpenSCAD.gif)

<a href="/wiki/File:Spring_100x20_in_OpenSCAD.gif" class="internal" title="Enlarge"></a>

Helical spring, 5x360° plus 8° at each end.

![](https://upload.wikimedia.org/wikipedia/commons/thumb/c/c0/Openscad_linext_04.jpg/400px-Openscad_linext_04.jpg)

**-500° of Twist**

```scad
linear_extrude(height = 10, center = true, convexity = 10, twist = -500)
translate([2, 0, 0])
circle(r = 1);
```

#### Center

It is similar to the parameter center of cylinders. If `center` is false
the linear extrusion Z range is from 0 to height; if it is true, the
range is from -height/2 to height/2.

![](https://upload.wikimedia.org/wikipedia/commons/thumb/c/c0/Openscad_linext_04.jpg/400px-Openscad_linext_04.jpg)

**center = true**

```scad
linear_extrude(height = 10, center = true, convexity = 10, twist = -500)
translate([2, 0, 0])
circle(r = 1);
```

![](https://upload.wikimedia.org/wikipedia/commons/thumb/3/3d/Openscad_linext_05.jpg/400px-Openscad_linext_05.jpg)

**center = false**

```scad
linear_extrude(height = 10, center = false, convexity = 10, twist = -500)
translate([2, 0, 0])
circle(r = 1);
```

#### Mesh Refinement

The slices parameter defines the number of intermediate points along the
Z axis of the extrusion. Its default increases with the value of twist.
Explicitly setting slices may improve the output refinement. Additional
the segments parameter adds vertices (points) to the extruded polygon
resulting in smoother twisted geometries. Segments need to be a multiple
of the polygon's fragments to have an effect (6 or 9.. for a
circle($fn=3), 8,12.. for a square() ).

![](https://upload.wikimedia.org/wikipedia/commons/thumb/0/0d/Openscad_linext_06.jpg/400px-Openscad_linext_06.jpg)

```scad
linear_extrude(height = 10, center = false, convexity = 10, twist = 360, slices = 100)
translate([2, 0, 0])
circle(r = 1);
```

The [special
variables](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Other_Language_Features "OpenSCAD User Manual/Other Language Features")
$fn, $fs and $fa can also be used to improve the output. If slices is
not defined, its value is taken from the defined $fn value.

![](https://upload.wikimedia.org/wikipedia/commons/thumb/6/63/Openscad_linext_07.jpg/400px-Openscad_linext_07.jpg)

```scad
linear_extrude(height = 10, center = false, convexity = 10, twist = 360, $fn = 100)
translate([2, 0, 0])
circle(r = 1);
```

#### Scale

Scales the 2D shape by this value over the height of the extrusion.
Scale can be a scalar or a vector:

```scad
linear_extrude(height = 10, center = true, convexity = 10, scale=3)
translate([2, 0, 0])
circle(r = 1);
```

![](https://upload.wikimedia.org/wikipedia/commons/thumb/5/59/Openscad_linext_09.png/400px-Openscad_linext_09.png)

```scad
linear_extrude(height = 10, center = true, convexity = 10, scale=[1,5], $fn=100)
translate([2, 0, 0])
circle(r = 1);
```

![](https://upload.wikimedia.org/wikipedia/commons/thumb/f/f5/OpenScad_linear_extrude_scale_example2.png/400px-OpenScad_linear_extrude_scale_example2.png)

Note that if scale is a vector, the resulting side walls may be
nonplanar. Use `twist=0` and the `slices` parameter to avoid
<a href="https://github.com/openscad/openscad/issues/1341" class="external text">asymmetry</a>.

```scad
linear_extrude(height=10, scale=[1,0.1], slices=20, twist=0)
polygon(points=[[0,0],[20,10],[20,-10]]);
```
*/
module linear_extrude(height, center=false, convexity=10, twist=0, slices=20, scale=1.0) { /* group */ }

/**
![](https://upload.wikimedia.org/wikipedia/commons/thumb/9/94/Openscad_minkowski_example_1a.png/200px-Openscad_minkowski_example_1a.png)

<a href="/wiki/File:Openscad_minkowski_example_1a.png" class="internal" title="Enlarge"></a>

A box and a cylinder

![](https://upload.wikimedia.org/wikipedia/commons/thumb/1/10/Openscad_minkowski_example_2a.png/200px-Openscad_minkowski_example_2a.png)

<a href="/wiki/File:Openscad_minkowski_example_2a.png" class="internal" title="Enlarge"></a>

Minkowski sum of the box and cylinder

Displays the <a href="https://www.cgal.org/Manual/latest/doc_html/cgal_manual/Minkowski_sum_3/Chapter_main.html"
class="external text">minkowski sum</a> of child nodes.

Usage example:

Say you have a flat box, and you want a rounded edge. There are multiple
ways to do this (for example, see [hull](#hull) below), but minkowski is
elegant. Take your box, and a cylinder:

```scad
$fn=50;
cube([10,10,1]);
cylinder(r=2,h=1);
```

Then, do a minkowski sum of them (note that the outer dimensions of the
box are now 10+2+2 = 14 units by 14 units by 2 units high as the heights
of the objects are summed):

```scad
$fn=50;
minkowski()
{
cube([10,10,1]);
cylinder(r=2,h=1);
}
```

NB: The <u>**origin**</u> of the second object is used for the addition.
If the second object is not centered, then the addition is asymmetric.
The following minkowski sums are different: the first expands the
original cube by 0.5 units in all directions, both positive and
negative. The second expands it by +1 in each positive direction, but
doesn't expand in the negative directions.

```scad
minkowski() {
    cube([10, 10, 1]);
    cylinder(1, center=true);
}
```

```scad
minkowski() {
    cube([10, 10, 1]);
    cylinder(1);
}
```

**Warning:** for high values of $fn the minkowski sum may end up
consuming lots of CPU and memory, since it has to combine every child
node of each element with all the nodes of each other element. So if for
example $fn=100 and you combine two cylinders, then it does not just
perform 200 operations as with two independent cylinders, but 100\*100 =
10000 operations.
*/
module minkowski() { /* group */ }

/**
Mirrors the child element on a plane through the origin. The argument to
mirror() is the normal vector of a plane intersecting the origin through
which to mirror the object.

#### Function signature:

```scad
mirror(v= [x, y, z] ) { ... }
```

#### Examples

The original is on the right side. Note that mirror doesn't make a copy.
Like rotate and scale, it changes the object.

- ![](https://upload.wikimedia.org/wikipedia/commons/thumb/b/b8/Mirror-x.png/170px-Mirror-x.png)

`hand(); // original`  
    `mirror([1,0,0]) hand();`

- ![](https://upload.wikimedia.org/wikipedia/commons/thumb/d/d2/Mirror-x-y.png/170px-Mirror-x-y.png)

`hand(); // original`  
    `mirror([1,1,0]) hand();`

- ![](https://upload.wikimedia.org/wikipedia/commons/thumb/d/d0/Mirror-x-y-z.png/170px-Mirror-x-y-z.png)

`hand(); // original`  
    `mirror([1,1,1]) hand();`

<!-- -->

```scad
rotate([0,0,10]) cube([3,2,1]);
mirror([1,0,0]) translate([1,0,0]) rotate([0,0,10]) cube([3,2,1]);
```

![](https://upload.wikimedia.org/wikipedia/commons/c/c9/OpenSCAD_mirror%28%29_example.JPG)
*/
module mirror(v) { /* group */ }

/**
Multiplies the geometry of all child elements with the given
<a href="https://en.wikipedia.org/wiki/Affine_transformation" class="extiw" title="w:Affine transformation">affine transformation</a>
matrix, where the matrix is 4×3 - a vector of 3 row vectors with 4
elements each, or a 4×4 matrix with the 4th row always forced to
[0,0,0,1].

Usage: multmatrix(m = [...]) { ... }

This is a breakdown of what you can do with the independent elements in
the matrix (for the first three rows):  

<table>
<tbody>
<tr class="odd">
<td>[Scale X]</td>
<td>[Shear X along Y]</td>
<td>[Shear X along Z]</td>
<td>[Translate X]</td>
</tr>
<tr class="even">
<td>[Shear Y along X]</td>
<td>[Scale Y]</td>
<td>[Shear Y along Z]</td>
<td>[Translate Y]</td>
</tr>
<tr class="odd">
<td>[Shear Z along X]</td>
<td>[Shear Z along Y]</td>
<td>[Scale Z]</td>
<td>[Translate Z]</td>
</tr>
</tbody>
</table>

The fourth row is forced to [0,0,0,1] and can be omitted unless you
are combining matrices before passing to multmatrix, as it is not
processed in OpenSCAD. Each matrix operates on the points of the given
geometry as if each vertex is a 4 element vector consisting of a 3D
vector with an implicit 1 as its 4th element, such as v=[x, y, z, 1].
The role of the implicit fourth row of m is to preserve the implicit 1
in the 4th element of the vectors, permitting the translations to work.
The operation of multmatrix therefore performs m\*v for each vertex v.
Any elements (other than the 4th row) not specified in m are treated as
zeros.

This example rotates by 45 degrees in the XY plane and translates by
[10,20,30], i.e. the same as translate([10,20,30])
rotate([0,0,45]) would do.

```scad
angle=45;
multmatrix(m = [ [cos(angle), -sin(angle), 0, 10],
                 [sin(angle),  cos(angle), 0, 20],
                 [         0,           0, 1, 30],
                 [         0,           0, 0,  1]
              ]) union() {
   cylinder(r=10.0,h=10,center=false);
   cube(size=[10,10,10],center=false);
}
```

The following example demonstrates combining affine transformation
matrices by matrix multiplication, producing in the final version a
transformation equivalent to rotate([0, -35, 0]) translate([40, 0,
0]) Obj();. Note that the signs on the sin function appear to be in a
different order than the above example, because the positive one must be
ordered as x into y, y into z, z into x for the rotation angles to
correspond to rotation about the other axis in a right-handed coordinate
system.

```scad
y_ang=-35;
mrot_y = [ [ cos(y_ang), 0,  sin(y_ang), 0],
           [         0,  1,           0, 0],
           [-sin(y_ang), 0,  cos(y_ang), 0],
           [         0,  0,           0, 1]
         ];
mtrans_x = [ [1, 0, 0, 40],
             [0, 1, 0,  0],
             [0, 0, 1,  0],
             [0, 0, 0,  1]
           ];
module Obj() {
   cylinder(r=10.0,h=10,center=false);
   cube(size=[10,10,10],center=false);
}

echo(mrot_y*mtrans_x);
Obj();
multmatrix(mtrans_x) Obj();
multmatrix(mrot_y * mtrans_x) Obj();
```

This example skews a model, which is not possible with the other
transformations.

```scad
M = [ [ 1  , 0  , 0  , 0   ],
      [ 0  , 1  , 0.7, 0   ],  // The "0.7" is the skew value; pushed along the y axis as z changes.
      [ 0  , 0  , 1  , 0   ],
      [ 0  , 0  , 0  , 1   ] ] ;
multmatrix(M) {  union() {
    cylinder(r=10.0,h=10,center=false);
    cube(size=[10,10,10],center=false); 
} }
```

This example shows how a vector is transformed with a multmatrix vector,
like this all points in a point array (polygon) can be transformed
sequentially. Vector (v) is transformed with a rotation matrix (m),
resulting in a new vector (vtrans) which is now rotated and is moving
the cube along a circular path radius=v around the z axis without
rotating the cube.

```scad
angle=45;
m=[
        [cos(angle), -sin(angle), 0, 0],
        [sin(angle),  cos(angle), 0, 0],
        [         0,           0, 1, 0]
   ];
              
v=[10,0,0];
vm=concat(v,[1]); // need to add [1]
vtrans=m*vm;
echo(vtrans);
translate(vtrans)cube();
```

#### More?

Learn more about it here:

-   <a     href="https://en.wikipedia.org/wiki/Transformation_matrix#Affine_transformations"
    class="external text">Affine Transformations</a> on wikipedia
-   <a href="http://www.senocular.com/flash/tutorials/transformmatrix/"     class="external free">http://www.senocular.com/flash/tutorials/transformmatrix/</a>
*/
module multmatrix(m) { /* group */ }

/**
[Note: Requires version 2015.03]

Offset generates a new 2d interior or exterior outline from an existing
outline. There are two modes of operation. radial and offset. The offset
method creates a new outline whose sides are a fixed distance outer
(delta > 0) or inner (delta < 0) from the original outline. The radial
method creates a new outline as if a circle of some radius is rotated
around the exterior (r>0) or interior (r<0) original outline.

The construction methods can either produce an outline that is interior
or exterior to the original outline. For exterior outlines the corners
can be given an optional chamfer.

Offset is useful for making thin walls by subtracting a negative-offset
construction from the original, or the original from a Positive offset
construction.

Offset can be used to simulate some common solid modeling operations:

-   Fillet: offset(r=-3) offset(delta=+3) rounds all inside (concave)
    corners, and leaves flat walls unchanged. However, holes less than
    2\*r in diameter vanish.
-   Round: offset(r=+3) offset(delta=-3) rounds all outside (convex)
    corners, and leaves flat walls unchanged. However, walls less than
    2\*r thick vanish.

Parameters

r  
Double. Amount to offset the polygon. When negative, the polygon is
offset inward. R specifies the radius of the circle that is rotated
about the outline, either inside or outside. This mode produces rounded
corners.

delta  
Double. Amount to offset the polygon. Delta specifies the distance of
the new outline from the original outline, and therefore reproduces
angled corners. When negative, the polygon is offset inward. No inward
perimeter is generated in places where the perimeter would cross itself.

chamfer  
Boolean. (default *false*) When using the delta parameter, this flag
defines if edges should be chamfered (cut off with a straight line) or
not (extended to their intersection).

![](https://upload.wikimedia.org/wikipedia/commons/thumb/9/93/OpenSCAD_offset_join-type_out.svg/350px-OpenSCAD_offset_join-type_out.svg.png)

Positive r/delta value

![](https://upload.wikimedia.org/wikipedia/commons/thumb/7/7b/OpenSCAD_offset_join-type_in.svg/350px-OpenSCAD_offset_join-type_in.svg.png)

Negative r/delta value

Result for different parameters. The black polygon is the input for the
offset() operation.

**Examples**

![](https://upload.wikimedia.org/wikipedia/commons/thumb/0/08/OpenSCAD_offset_example.png/220px-OpenSCAD_offset_example.png)

<a href="/wiki/File:OpenSCAD_offset_example.png" class="internal" title="Enlarge"></a>

Example 1: Result.

```scad
// Example 1
 
linear_extrude(height = 60, twist = 90, slices = 60) {
   difference() {
     offset(r = 10) {
      square(20, center = true);
     }
     offset(r = 8) {
       square(20, center = true);
     }
   }
}
```

```scad
// Example 2
 
module fillet(r) {
   offset(r = -r) {
     offset(delta = r) {
       children();
     }
   }
}
```
*/
module offset(delta, r=0, chamfer=false) { /* group */ }

module parent_module() { /* group */ }

/**
Using the `projection()` function, you can create 2d drawings from 3d
models, and export them to the dxf format. It works by projecting a 3D
model to the (x,y) plane, with z at 0. If `cut=true`, only points with
z=0 are considered (effectively cutting the object), with
`cut=false`(*the default*), points above and below the plane are
considered as well (creating a proper projection).

**Example**: Consider example002.scad, that comes with OpenSCAD.

![](https://upload.wikimedia.org/wikipedia/commons/2/2e/Openscad_projection_example_2x.png)

Then you can do a 'cut' projection, which gives you the 'slice' of the
x-y plane with z=0.

```scad
projection(cut = true) example002();
```

![](https://upload.wikimedia.org/wikipedia/commons/1/13/Openscad_projection_example_3x.png)

You can also do an 'ordinary' projection, which gives a sort of 'shadow'
of the object onto the xy plane.

```scad
projection(cut = false) example002();
```

![](https://upload.wikimedia.org/wikipedia/commons/5/5b/Openscad_example_projection_8x.png)

**Another Example**

You can also use projection to get a 'side view' of an object. Let's
take example002, and move it up, out of the X-Y plane, and rotate it:

```scad
translate([0,0,25]) rotate([90,0,0]) example002();
```

![](https://upload.wikimedia.org/wikipedia/commons/c/cd/Openscad_projection_example_4x.png)

Now we can get a side view with projection()

```scad
projection() translate([0,0,25]) rotate([90,0,0]) example002();
```

![](https://upload.wikimedia.org/wikipedia/commons/7/7d/Openscad_projection_example_5x.png)

Links:

-   <a href="http://svn.clifford.at/openscad/trunk/examples/example021.scad"     class="external text">example021.scad from Clifford Wolf's site</a>.
-   <a     href="http://www.gilesbathgate.com/2010/06/extracting-2d-mendel-outlines-using-openscad/"
    class="external text">More complicated example</a> from Giles
    Bathgate's blog

Retrieved from
"<https://en.wikibooks.org/w/index.php?title=OpenSCAD_User_Manual/3D_to_2D_Projection&oldid=3674389>"

[Category](https://en.wikibooks.org/wiki/Special:Categories "Special:Categories"):

-   [Book:OpenSCAD User
    Manual](https://en.wikibooks.org/wiki/Category:Book:OpenSCAD_User_Manual "Category:Book:OpenSCAD User Manual")

*/
module projection() { /* group */ }

/**
**Warning:** Using render, always calculates the CSG model for this tree
(even in OpenCSG preview mode). This can make previewing very slow and
OpenSCAD to appear to hang/freeze.

```scad
Usage example:
render(convexity = 1) { ... }
```

<table>
<tbody>
<tr class="odd">
<td>convexity</td>
<td>Integer. The convexity parameter specifies the maximum number of
front and back sides a ray intersecting the object might penetrate. This
parameter is only needed for correctly displaying the object in OpenCSG
preview mode and has no effect on the polyhedron rendering.</td>
</tr>
</tbody>
</table>

![](https://upload.wikimedia.org/wikipedia/commons/thumb/0/0c/Openscad_convexity.jpg/400px-Openscad_convexity.jpg)

This image shows a 2D shape with a convexity of 4, as the ray indicated
in red crosses the 2D shape a maximum of 4 times. The convexity of a 3D
shape would be determined in a similar way. Setting it to 10 should work
fine for most cases.

Retrieved from
"<https://en.wikibooks.org/w/index.php?title=OpenSCAD_User_Manual/CSG_Modelling&oldid=3830886>"

[Category](https://en.wikibooks.org/wiki/Special:Categories "Special:Categories"):

-   [Book:OpenSCAD User
    Manual](https://en.wikibooks.org/wiki/Category:Book:OpenSCAD_User_Manual "Category:Book:OpenSCAD User Manual")
*/
module render() { /* group */ }

/**
Modifies the size of the child object to match the given x,y, and z.

resize() is a CGAL operation, and like others such as render() operates
with full geometry, so even in preview this takes time to process.

Usage Example:

```scad
// resize the sphere to extend 30 in x, 60 in y, and 10 in the z directions.
resize(newsize=[30,60,10]) sphere(r=10);
```

![](https://upload.wikimedia.org/wikipedia/commons/thumb/d/d8/OpenSCAD_Resize_example_elipse.JPG/400px-OpenSCAD_Resize_example_elipse.JPG)

If x,y, or z is 0 then that dimension is left as-is.

```scad
// resize the 1x1x1 cube to 2x2x1
resize([2,2,0]) cube();
```

If the 'auto' parameter is set to true, it auto-scales any 0-dimensions
to match. For example.

```scad
// resize the 1x2x0.5 cube to 7x14x3.5
resize([7,0,0], auto=true) cube([1,2,0.5]);
```

The 'auto' parameter can also be used if you only wish to auto-scale a
single dimension, and leave the other as-is.

```scad
// resize to 10x8x1. Note that the z dimension is left alone.
resize([10,0,0], auto=[true,true,false]) cube([5,4,1]);
```
*/
module resize(newsize) { /* group */ }

/**
Rotates its child 'a' degrees about the axis of the coordinate system or
around an arbitrary axis. The argument names are optional if the
arguments are given in the same order as specified.

```scad
//Usage:
rotate(a = deg_a, v = [x, y, z]) { ... }  
// or
rotate(deg_a, [x, y, z]) { ... }
rotate(a = [deg_x, deg_y, deg_z]) { ... }
rotate([deg_x, deg_y, deg_z]) { ... }
```

The 'a' argument (deg_a) can be an array, as expressed in the later
usage above; when deg_a is an array, the 'v' argument is ignored. Where
'a' specifies *multiple axes* then the rotation is applied in the
following order: z, y, x. That means the code:

```scad
rotate(a=[ax,ay,az]) {...}
```

is equivalent to:

```scad
rotate(a=[0,0,az]) rotate(a=[0,ay,0]) rotate(a=[ax,0,0]) {...}
```

The optional argument 'v' is a vector and allows you to set an arbitrary
axis about which the object is rotated.

For example, to flip an object upside-down, you can rotate your object
180 degrees around the 'y' axis.

```scad
rotate(a=[0,180,0]) { ... }
```

This is frequently simplified to

```scad
rotate([0,180,0]) { ... }
```

When specifying a single axis the 'v' argument allows you to specify
which axis is the basis for rotation. For example, the equivalent to the
above, to rotate just around y

```scad
rotate(a=180, v=[0,1,0]) { ... }
```

When specifying a single axis, 'v' is a
<a href="https://en.wikipedia.org/wiki/Euler_vector" class="extiw" title="wikipedia:Euler vector">vector</a> defining an arbitrary axis for
rotation; this is different from the *multiple axis* above. For example,
rotate your object 45 degrees around the axis defined by the vector
[1,1,0],

```scad
rotate(a=45, v=[1,1,0]) { ... }
```

![](https://upload.wikimedia.org/wikipedia/commons/7/77/OpenSCAD_rotate%28%29_example.JPG)

Rotate with a *single scalar argument* rotates around the Z axis. This
is useful in 2D contexts where that is the only axis for rotation. For
example:

```scad
rotate(45) square(10);
```

![](https://upload.wikimedia.org/wikipedia/commons/b/b9/Example_2D_Rotate.JPG)

##### Rotation rule help

![](https://upload.wikimedia.org/wikipedia/commons/thumb/3/34/Right-hand_grip_rule.svg/220px-Right-hand_grip_rule.svg.png)

<a href="/wiki/File:Right-hand_grip_rule.svg" class="internal" title="Enlarge"></a>

Right-hand grip rule

For the case of:

```scad
rotate([a, b, c]) { ... };
```

"a" is a rotation about the X axis, from the +Y axis, toward the +Z
axis.  
"b" is a rotation about the Y axis, from the +Z axis, toward the +X
axis.  
"c" is a rotation about the Z axis, from the +X axis, toward the +Y
axis.

These are all cases of the
<a href="https://en.wikipedia.org/wiki/right-hand_rule" class="extiw" title="wikipedia:right-hand rule">Right Hand Rule</a>. Point your right
thumb along the positive axis, your fingers show the direction of
rotation.

Thus if "a" is fixed to zero, and "b" and "c" are manipulated
appropriately, this is the *spherical coordinate system*.  
So, to construct a cylinder from the origin to some other point (x,y,z):

```scad
x= 10; y = 10; z = 10; // point coordinates of end of cylinder
 
length = norm([x,y,z]);  // radial distance
b = acos(z/length); // inclination angle
c = atan2(y,x);     // azimuthal angle

rotate([0, b, c]) 
    cylinder(h=length, r=0.5);
%cube([x,y,z]); // corner of cube should coincide with end of cylinder
```

![](https://upload.wikimedia.org/wikipedia/commons/6/61/Example_xyz_rotation_in_OpenSCAD.JPG)
*/
module rotate(angles) { /* group */ }

/**
Rotational extrusion spins a 2D shape around the Z-axis to form a solid
which has rotational symmetry. One way to think of this operation is to
imagine a Potter's wheel placed on the X-Y plane with its axis of
rotation pointing up towards +Z. Then place the to-be-made object on
this virtual Potter's wheel (possibly extending down below the X-Y plane
towards -Z). The to-be-made object is the cross-section of the object on
the X-Y plane (keeping only the right half, X >= 0). That is the 2D
shape that will be fed to rotate_extrude() as the child in order to
generate this solid. Note that the object started on the X-Y plane but
is tilted up (rotated +90 degrees about the X-axis) to extrude.

Since a 2D shape is rendered by OpenSCAD on the X-Y plane, an
alternative way to think of this operation is as follows: spins a 2D
shape around the Y-axis to form a solid. The resultant solid is placed
so that its axis of rotation lies along the Z-axis.

Just like the linear_extrude, the extrusion is always performed on the
projection of the 2D polygon to the XY plane. Transformations like
rotate, translate, etc. applied to the 2D polygon before extrusion
modify the projection of the 2D polygon to the XY plane and therefore
also modify the appearance of the final 3D object.

-   A translation in Z of the 2D polygon has no effect on the result (as
    also the projection is not affected).
-   A translation in X increases the diameter of the final object.
-   A translation in Y results in a shift of the final object in Z
    direction.
-   A rotation about the X or Y axis distorts the cross section of the
    final object, as also the projection to the XY plane is distorted.

Don't get confused, as OpenSCAD renders 2D polygons with a certain
height in the Z direction, so the 2D object (with its height) appears to
have a bigger projection to the XY plane. But for the projection to the
XY plane and also for the later extrusion only the base polygon without
height is used.

It can not be used to produce a helix or screw threads. (These things
can be done with
[linear_extrude()](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/The_OpenSCAD_Language#Linear_Extrude "OpenSCAD User Manual/The OpenSCAD Language")
using the twist parameter.)

The 2D shape **must** lie completely on either the right (recommended)
or the left side of the Y-axis. More precisely speaking, **every**
vertex of the shape must have either x >= 0 or x <= 0. If the shape
spans the X axis a warning appears in the console windows and the
rotate_extrude() is ignored. If the 2D shape touches the Y axis, i.e. at
x=0, it **must** be a line that touches, not a point, as a point results
in a zero thickness 3D object, which is invalid and results in a CGAL
error. For OpenSCAD versions prior to 2016.xxxx, if the shape is in the
negative axis the resulting faces are oriented inside-out, which may
cause undesired effects.

#### Usage

```scad
rotate_extrude(angle = 360, convexity = 2) {...}
```

![](https://upload.wikimedia.org/wikipedia/commons/thumb/3/34/Right-hand_grip_rule.svg/220px-Right-hand_grip_rule.svg.png)

<a href="/wiki/File:Right-hand_grip_rule.svg" class="internal" title="Enlarge"></a>

Right-hand grip rule

You must use parameter names due to a backward compatibility issue.

**convexity** : If the extrusion fails for a non-trival 2D shape, try
setting the convexity parameter (the default is not 10, but 10 is a
"good" value to try). See explanation further down.

**angle** [Note: Requires version 2019.05] : Defaults to 360.
Specifies the number of degrees to sweep, starting at the positive X
axis. The direction of the sweep follows the
<a href="https://en.wikipedia.org/wiki/right-hand_rule" class="extiw" title="wikipedia:right-hand rule">Right Hand Rule</a>, hence a negative
angle sweeps clockwise.

**$fa** : minimum angle (in degrees) of each fragment.

**$fs** : minimum circumferential length of each fragment.

**$fn** : **fixed** number of fragments in 360 degrees. Values of 3 or
more override $fa and $fs

$fa, $fs and $fn must be named parameters. [click here for more
details,](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Other_Language_Features "OpenSCAD User Manual/Other Language Features").

#### Examples

![](https://upload.wikimedia.org/wikipedia/commons/thumb/7/7d/Rotate_extrude_wiki_2D.jpg/400px-Rotate_extrude_wiki_2D.jpg)

→![](https://upload.wikimedia.org/wikipedia/commons/thumb/b/be/Openscad_rotext_01.jpg/400px-Openscad_rotext_01.jpg)

A simple torus can be constructed using a rotational extrude.

```scad
rotate_extrude(convexity = 10)
translate([2, 0, 0])
circle(r = 1);
```

#### Mesh Refinement

![](https://upload.wikimedia.org/wikipedia/commons/thumb/c/c0/Rotate_extrude_wiki_2D_C.jpg/380px-Rotate_extrude_wiki_2D_C.jpg)

→![](https://upload.wikimedia.org/wikipedia/commons/thumb/5/5e/Openscad_rotext_02.jpg/400px-Openscad_rotext_02.jpg)

Increasing the number of fragments composing the 2D shape improves the
quality of the mesh, but takes longer to render.

```scad
rotate_extrude(convexity = 10)
translate([2, 0, 0])
circle(r = 1, $fn = 100);
```

![](https://upload.wikimedia.org/wikipedia/commons/thumb/c/c0/Rotate_extrude_wiki_2D_C.jpg/380px-Rotate_extrude_wiki_2D_C.jpg)

→![](https://upload.wikimedia.org/wikipedia/commons/thumb/4/4b/Openscad_rotext_03.jpg/400px-Openscad_rotext_03.jpg)

The number of fragments used by the extrusion can also be increased.

```scad
rotate_extrude(convexity = 10, $fn = 100)
translate([2, 0, 0])
circle(r = 1, $fn = 100);
```

Using the parameter angle (with OpenSCAD versions 2016.xx), a hook can
be modeled .

![](https://upload.wikimedia.org/wikipedia/commons/thumb/4/47/Hook.png/220px-Hook.png)

<a href="/wiki/File:Hook.png" class="internal" title="Enlarge"></a>

OpenSCAD - a hook

```scad
eps = 0.01;
translate([eps, 60, 0])
   rotate_extrude(angle=270, convexity=10)
       translate([40, 0]) circle(10);
rotate_extrude(angle=90, convexity=10)
   translate([20, 0]) circle(10);
translate([20, eps, 0])
   rotate([90, 0, 0]) cylinder(r=10, h=80+eps);
```

#### Extruding a Polygon

Extrusion can also be performed on polygons with points chosen by the
user.

Here is a simple polygon and its 200 step rotational extrusion. (Note it
has been rotated 90 degrees to show how the rotation appears; the
`rotate_extrude()` needs it flat).

```scad
rotate([90,0,0])        polygon( points=[[0,0],[2,1],[1,2],[1,3],[3,4],[0,5]] );
```

```scad
rotate_extrude($fn=200) polygon( points=[[0,0],[2,1],[1,2],[1,3],[3,4],[0,5]] );
```

![](https://upload.wikimedia.org/wikipedia/commons/thumb/9/95/Rotate_extrude_wiki_2D_B.jpg/300px-Rotate_extrude_wiki_2D_B.jpg)

→![](https://upload.wikimedia.org/wikipedia/commons/thumb/6/6c/Openscad_polygon_extrusion_1.png/300px-Openscad_polygon_extrusion_1.png)→![](https://upload.wikimedia.org/wikipedia/commons/thumb/e/ea/Openscad_polygon_extrusion_2.png/300px-Openscad_polygon_extrusion_2.png)

For more information on polygons, please see: [2D Primitives:
Polygon](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/2D_Primitives#polygon "OpenSCAD User Manual/2D Primitives").
*/
module rotate_extrude(angle=360, convexity=2) { /* group */ }

/**
Scales its child elements using the specified vector. The argument name
is optional.

```scad
Usage Example:
scale(v = [x, y, z]) { ... }
```

```scad
cube(10);
translate([15,0,0]) scale([0.5,1,2]) cube(10);
```

![](https://upload.wikimedia.org/wikipedia/commons/a/a7/OpenSCAD_scale%28%29_example.JPG)
*/
module scale(v) { /* group */ }

/**
Translates (moves) its child elements along the specified vector. The
argument name is optional.

```scad
Example:
translate(v = [x, y, z]) { ... }
```

```scad
cube(2,center = true); 
translate([5,0,0]) 
   sphere(1,center = true);
```

![](https://upload.wikimedia.org/wikipedia/commons/a/ad/OpenSCAD_translate%28%29_example.JPG)
*/
module translate(v) { /* group */ }

/**
Creates a union of all its child nodes. This is the **sum** of all
children (logical **or**).  
May be used with either 2D or 3D objects, but don't mix them.

![](https://upload.wikimedia.org/wikipedia/commons/thumb/1/1d/Openscad_union.jpg/400px-Openscad_union.jpg)

```scad
//Usage example:
union() {
    cylinder (h = 4, r=1, center = true, $fn=100);
    rotate ([90,0,0]) cylinder (h = 4, r=0.9, center = true, $fn=100);
}
```

Remark: union is implicit when not used. But it is mandatory, for
example, in difference to group first child nodes into one.

**Note:** It is mandatory for all unions, explicit or implicit, that
external faces to be merged not be coincident. Failure to follow this
rule results in a design with undefined behavior, and can result in a
render which is not manifold (with zero volume portions, or portions
inside out), which typically leads to a warning and sometimes removal of
a portion of the design from the rendered output. (This can also result
in [flickering effects during the
preview](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/FAQ#What_are_those_strange_flickering_artifacts_in_the_preview? "OpenSCAD User Manual/FAQ").)
This requirement is not a bug, but an intrinsic property of floating
point comparisons and the fundamental inability to exactly represent
irrational numbers such as those resulting from most rotations. As an
example, this is an invalid OpenSCAD program, and will at least lead to
a warning on most platforms:

```scad
// Invalid!
size = 10;
rotation = 17;
union() {
    rotate([rotation, 0, 0])
       cube(size);
    rotate([rotation, 0, 0])
       translate([0, 0, size])
       cube([2, 3, 4]);
}
```

The solution is to always use a small value called an epsilon when
merging adjacent faces like this to guarantee overlap. Note the 0.01 eps
value used in TWO locations, so that the external result is equivalent
to what was intended:

```scad
// Correct!
size = 10;
rotation = 17;
eps = 0.01;
union() {
    rotate([rotation, 0, 0])
       cube(size);
    rotate([rotation, 0, 0])
       translate([0, 0, size-eps])
       cube([2, 3, 4+eps]);
}
```
*/
module union() { /* group */ }

/**
Mathematical **absolute value** function. Returns the positive value of
a signed decimal number.

**Usage examples:**

```scad
abs(-5.0);  returns 5.0
abs(0);     returns 0.0
abs(8.0);   returns 8.0
```
*/
function abs(x) = 0;

/**
Mathematical **arccosine**, or **inverse cosine**, expressed in degrees.
See:
<a href="https://en.wikipedia.org/wiki/Inverse_trigonometric_functions" class="extiw" title="w:Inverse trigonometric functions">Inverse
trigonometric functions</a>
*/
function acos(x) = 0;

/**
Mathematical **arcsine**, or **inverse sine**, expressed in degrees.
See:
<a href="https://en.wikipedia.org/wiki/Inverse_trigonometric_functions" class="extiw" title="w:Inverse trigonometric functions">Inverse
trigonometric functions</a>
*/
function asin(x) = 0;

function assert(cond) = 0;

/**
Mathematical **arctangent**, or **inverse tangent**, function. Returns
the principal value of the arc tangent of x, expressed in degrees. See:
<a href="https://en.wikipedia.org/wiki/Inverse_trigonometric_functions" class="extiw" title="w:Inverse trigonometric functions">Inverse
trigonometric functions</a>
*/
function atan(x) = 0;

/**
Mathematical **two-argument atan** function atan2(y,x) that spans the
full 360 degrees. This function returns the full angle (0-360) made
between the x axis and the vector(x,y) expressed in degrees. atan can
not distinguish between y/x and -y/-x and returns angles from -90 to +90
See: <a href="https://en.wikipedia.org/wiki/Atan2" class="extiw" title="w:Atan2">atan2</a>

**Usage examples:**

```scad
atan2(5.0,-5.0);     //result: 135 degrees. atan() would give -45
atan2(y,x);          //angle between (1,0) and (x,y) = angle around z-axis
```

## Other Mathematical Functions
*/
function atan2(y, x) = 0;

/**
Mathematical **ceiling** function.

Returns the next highest integer value by rounding up value if
necessary.

See: <a href="https://en.wikipedia.org/wiki/Ceil_function" class="extiw" title="w:Ceil function">Ceil Function</a>

```scad
echo(ceil(4.4),ceil(-4.4));     // produces ECHO: 5, -4
```
*/
function ceil(x) = 0;

/**
[Note: Requires version 2015.03]

Convert numbers to a string containing character with the corresponding
code. OpenSCAD uses Unicode, so the number is interpreted as Unicode
code point. Numbers outside the valid code point range produce an empty
string.

**Parameters**

chr(Number)  
Convert one code point to a string of length 1 (number of bytes
depending on UTF-8 encoding) if the code point is valid.

<!-- -->

chr(Vector)  
Convert all code points given in the argument vector to a string.

<!-- -->

chr(Range)  
Convert all code points produced by the range argument to a string.

**Examples**

```scad
echo(chr(65), chr(97));      // ECHO: "A", "a"
echo(chr(65, 97));           // ECHO: "Aa"
echo(chr([66, 98]));         // ECHO: "Bb"
echo(chr([97 : 2 : 102]));   // ECHO: "ace"
echo(chr(-3));               // ECHO: ""
echo(chr(9786), chr(9788));  // ECHO: "☺", "☼"
echo(len(chr(9788)));        // ECHO: 1
```

Note: When used with echo() the output to the console for character
codes greater than 127 is platform dependent.
*/
function chr(x) = 0;

/**
[Note: Requires version 2015.03]

Return a new vector that is the result of appending the elements of the
supplied vectors.

Where an argument is a vector the elements of the vector are
individually appended to the result vector. Strings are distinct from
vectors in this case.

**Usage examples:**

```scad
echo(concat("a","b","c","d","e","f"));          // produces ECHO: ["a", "b", "c", "d", "e", "f"]
echo(concat(["a","b","c"],["d","e","f"]));      // produces ECHO: ["a", "b", "c", "d", "e", "f"]
echo(concat(1,2,3,4,5,6));                      // produces ECHO: [1, 2, 3, 4, 5, 6]
```

Vector of vectors

```scad
echo(concat([ [1],[2] ], [ [3] ]));             // produces ECHO: [[1], [2], [3]]
```

***Note:*** All vectors passed to the function lose one nesting level.
When adding something like a single element [x, y, z] tuples (which
are vectors, too), the tuple needs to be enclosed in a vector (i.e. an
extra set of brackets) before the concatenation. in the exmple below, a
fourth point is added to the polygon path, which used to resemble a
triangle, making it a square now:

```scad
polygon(concat([[0,0],[0,5],[5,5]], [[5,0]]));
```

Contrast with strings

```scad
echo(concat([1,2,3],[4,5,6]));                   // produces ECHO: [1, 2, 3, 4, 5, 6]
echo(concat("abc","def"));                       // produces ECHO: ["abc", "def"]
echo(str("abc","def"));                          // produces ECHO: "abcdef"
```
*/
function concat(args) = 0;

/**
Mathematical **cosine** function of degrees. See <a href="https://en.wikipedia.org/wiki/Cosine#Sine.2C_cosine_and_tangent"
class="extiw" title="w:Cosine">Cosine</a>

**Parameters**

Decimal. Angle in degrees.

<strong>Usage Example:</strong>
```scad
for(i=[0:36])
    translate([i*10,0,0])
        cylinder(r=5,h=cos(i*10)*50+60);
```
![](https://upload.wikimedia.org/wikipedia/commons/thumb/3/3f/OpenSCAD_Cos_Function.png/220px-OpenSCAD_Cos_Function.png)
<a href="/wiki/File:OpenSCAD_Cos_Function.png" class="internal" title="Enlarge"></a>

OpenSCAD Cos Function

*/
function cos(x) = 0;

/**
Calculates the cross product of two vectors in 3D or 2D space. If both
vectors are in the 3D, the result is a vector that is perpendicular to
both of the input vectors. If both vectors are in 2D space, their cross
product has the form [0,0,z] and the cross function returns just the z
value of the cross product:

*cross([x,y], [u,v]) = x\*v - y\*u*

Note that this is the determinant of the 2x2 matrix [[x,y],[u,v]].
Using any other types, vectors with lengths different from 2 or 3, or
vectors not of the same length produces 'undef'.

**Usage examples:**

```scad
echo(cross([2, 3, 4], [5, 6, 7]));     // produces ECHO: [-3, 6, -3]
echo(cross([2, 1, -3], [0, 4, 5]));    // produces ECHO: [17, -10, 8]
echo(cross([2, 1], [0, 4]));           // produces ECHO: 8
echo(cross([1, -3], [4, 5]));          // produces ECHO: 17
echo(cross([2, 1, -3], [4, 5]));       // produces ECHO: undef
echo(cross([2, 3, 4], "5"));           // produces ECHO: undef
```

For any two vectors *a* and *b* in 2D or in 3D, the following holds:

*cross(a,b) == -cross(b,a)*
*/
function cross(u, v) = 0;

function dxf_cross() = 0;

function dxf_dim() = 0;

/**
Mathematical **exp** function. Returns the base-e exponential function
of x, which is the number e raised to the power x. See:
<a href="https://en.wikipedia.org/wiki/Exponent" class="extiw" title="w:Exponent">Exponent</a>

```scad
echo(exp(1),exp(ln(3)*4));    // produces ECHO: 2.71828, 81
```
*/
function exp(x) = 0;

/**
Mathematical **floor** function. floor(x) = is the largest integer not
greater than x

See:
<a href="https://en.wikipedia.org/wiki/Floor_function" class="extiw" title="w:Floor function">Floor Function</a>

```scad
echo(floor(4.4),floor(-4.4));    // produces ECHO: 4, -5
```
*/
function floor(x) = 0;

/**
[Note: Requires version 2019.05]

```scad
echo("resulting in true:");
echo(is_bool(true));
echo(is_bool(false));
echo("resulting in false:");
echo(is_bool([]));
echo(is_bool([1]));
echo(is_bool("test"));
echo(is_bool(0.1));
echo(is_bool(1));
echo(is_bool(10));
echo(is_bool(0/0)); //nan
echo(is_bool((1/0)/(1/0)));  //nan
echo(is_bool(1/0));  //inf
echo(is_bool(-1/0));  //-inf
echo(is_bool(undef));
```
*/
function is_bool(x) = 0;

/**
[Note: Requires version 2019.05]

```scad
echo("returning true");
echo(is_list([]));
echo(is_list([1]));
echo(is_list([1,2]));
echo(is_list([true]));
echo(is_list([1,2,[5,6],"test"]));
echo("--------");
echo("returning false");
echo(is_list(1));
echo(is_list(1/0));
echo(is_list(((1/0)/(1/0))));
echo(is_list("test"));
echo(is_list(true));
echo(is_list(false));
echo("--------");
echo("causing warnings:");
echo(is_list());
echo(is_list(1,2));
```
*/
function is_list(x) = 0;

/**
[Note: Requires version 2019.05]

```scad
echo("a number is a number:");
echo(is_num(0.1));
echo(is_num(1));
echo(is_num(10));

echo("inf is a number:");
echo(is_num(+1/0)); //+inf
echo(is_num(-1/0)); //-inf

echo("nan is not a number:");
echo(is_num(0/0)); //nan
echo(is_num((1/0)/(1/0)));  //nan

echo("resulting in false:");
echo(is_num([]));
echo(is_num([1]));
echo(is_num("test"));
echo(is_num(false));
echo(is_num(undef));
```
*/
function is_num(x) = 0;

/**
[Note: Requires version 2019.05]

```scad
echo("resulting in true:");
echo(is_string(""));
echo(is_string("test"));
echo("resulting in false:");
echo(is_string(0.1));
echo(is_string(1));
echo(is_string(10));
echo(is_string([]));
echo(is_string([1]));
echo(is_string(false));
echo(is_string(0/0)); //nan
echo(is_string((1/0)/(1/0)));  //nan
echo(is_string(1/0));  //inf
echo(is_string(-1/0));  //-inf
echo(is_string(undef));
```
*/
function is_string(x) = 0;

/**
[Note: Requires version 2019.05]

is_undef accepts one parameter. If the parameter is undef, this function
returns true. If the parameter is not undef, it returns false. When
checking a variable (like \`is_undef(a)\`), it does the variable lookup
silently, meaning that is_undef(a) does not cause \`WARNING: Ignoring
unknown variable 'a'. \`

The alternative is code like this:

```scad
if(a==undef){
    //code goes here
}
```

or

```scad
b = (a==undef) ? true : false;
```

causes

```scad
WARNING: Ignoring unknown variable 'a'.
```

is_undef also works for special variables, allowing for things like
this:

```scad
exploded = is_undef($exploded) ? 0 : $exploded; // 1 for exploded view
```

#### legacy support

For older openscad version, is_undef can be emulated with

```scad
function is_undef ( a ) = (undef == a) ;
```

which of-course causes warning(s), but requires no changes to code
relying on is_undef().
*/
function is_undef(x) = 0;

/**
returns the number of characters in a text.

```scad
echo(len("Hello world"));    // 11
```
*/
/**
Mathematical **length** function. Returns the length of an array, a
vector or a string parameter.

**Usage examples:**

```scad
; len_str1=len(str1);
echo(str1,len_str1);

a=6; len_a=len(a);
echo(a,len_a);

array1=[1,2,3,4,5,6,7,8]; len_array1=len(array1);
echo(array1,len_array1);

array2=[[0,0],[0,1],[1,0],[1,1]]; len_array2=len(array2);
echo(array2,len_array2);

len_array2_2=len(array2[2]);
echo(array2[2],len_array2_2);
```

**Results:**

```scad
ECHO: "abcdef", 6
An error is raised
ECHO: [1, 2, 3, 4, 5, 6, 7, 8], 8
ECHO: [[0, 0], [0, 1], [1, 0], [1, 1]], 4
ECHO: [1, 0], 2
```

This function allows (e.g.) the parsing of an array, a vector or a
string.

**Usage examples:**

```scad
;
for (i=[0:len(str2)-1])
    echo(str("digit ",i+1,"  :  ",str2[i]));
```

**Results:**

```scad
ECHO: "digit 1  :  4"
ECHO: "digit 2  :  7"
ECHO: "digit 3  :  1"
ECHO: "digit 4  :  1"
```

Note that the len() function is not defined when a simple variable is
passed as the parameter.

This is useful when handling parameters to a module, similar to how
shapes can be defined as a single number, or as an [x,y,z] vector;
i.e. cube(5) or cube([5,5,5])

For example

```scad
module doIt(size) {
    if (len(size) == undef) {
        // size is a number, use it for x,y & z. (or could be undef)
        do([size,size,size]);
    } else { 
        // size is a vector, (could be a string but that would be stupid)
        do(size);
    }
}
 
doIt(5);    // equivalent to [5,5,5]
doIt([5,5,5]);  // similar to cube(5) v's cube([5,5,5])
```
*/
function len(x) = 0;

/**
Mathematical **natural logarithm**. See:
<a href="https://en.wikipedia.org/wiki/Natural_logarithm" class="extiw" title="w:Natural logarithm">Natural logarithm</a>
*/
function ln(x) = 0;

/**
Mathematical **logarithm** to the base 10. Example: log(1000) = 3. See:
<a href="https://en.wikipedia.org/wiki/Logarithm" class="extiw" title="w:Logarithm">Logarithm</a>
*/
function log(x) = 0;

/**
Look up value in table, and linearly interpolate if there's no exact
match. The first argument is the value to look up. The second is the
lookup table -- a vector of key-value pairs.

**Parameters**

key  
A lookup key

<key,value> array  

keys and values
<strong>Notes</strong>

There is a bug in which out-of-range keys return the first value in the
list. Newer versions of Openscad should use the top or bottom end of the
table as appropriate instead.</p></td>

<strong>Usage example:</strong>
- Create a 3D chart made from cylinders of different heights.

```scad
function get_cylinder_h(p) = lookup(p, [
        [ -200, 5 ],
        [ -50, 20 ],
        [ -20, 18 ],
        [ +80, 25 ],
        [ +150, 2 ]
]);
 
 for (i = [-100:5:+100]) {
    // echo(i, get_cylinder_h(i));
    translate([ i, 0, -30 ]) cylinder(r1 = 6, r2 = 2, h = get_cylinder_h(i)*3);
 }
```
![](https://upload.wikimedia.org/wikipedia/commons/thumb/0/04/OpenSCAD_Lookup_Function.png/220px-OpenSCAD_Lookup_Function.png)
<a href="/wiki/File:OpenSCAD_Lookup_Function.png" class="internal" title="Enlarge"></a>
OpenSCAD Lookup Function

*/
function lookup(key, vals) = 0;

/**
Returns the maximum of the parameters. If a single vector is given as
parameter, returns the maximum element of that vector.

**Parameters**

```scad
max(n,n{,n}...)
max(vector)
```

<n>  
Two or more decimals

<vector>  
Single vector of decimals [Note: Requires version 2014.06].

**Usage Example:**

```scad
max(3.0,5.0)
max(8.0,3.0,4.0,5.0)
max([8,3,4,5])
```

**Results:**

```scad
5
8
8
```
*/
function max(args) = 0;

/**
Returns the minimum of the parameters. If a single vector is given as
parameter, returns the minimum element of that vector.

**Parameters**

```scad
min(n,n{,n}...)
min(vector)
```

<n>  
Two or more decimals

<vector>  
Single vector of decimals [Note: Requires version 2014.06].

**Usage Example:**

```scad
min(3.0,5.0)
min(8.0,3.0,4.0,5.0)
min([8,3,4,5])
```

**Results:**

```scad
3
3
3
```

Looking for **mod** - it's not a function, see [modulo operator
(%)](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Mathematical_Operators "OpenSCAD User Manual/Mathematical Operators")
*/
function min(args) = 0;

/**
Returns the
<a href="https://en.wikipedia.org/wiki/Norm_(mathematics)" class="extiw" title="w:Norm (mathematics)">euclidean norm</a> of a vector. Note this
returns the actual numeric length while **len** returns the number of
elements in the vector or array.

**Usage examples:**

```scad
a=[1,2,3,4];
;
c=[];
;
e=[[1,2,3,4],[1,2,3],[1,2],[1]];
echo(norm(a)); //5.47723
echo(norm(b)); //undef
echo(norm(c)); //0
echo(norm(d)); //undef
echo(norm(e[0])); //5.47723
echo(norm(e[1])); //3.74166
echo(norm(e[2])); //2.23607
echo(norm(e[3])); //1
```

**Results:**

```scad
ECHO: 5.47723
ECHO: undef
ECHO: 0
ECHO: undef
ECHO: 5.47723
ECHO: 3.74166
ECHO: 2.23607
ECHO: 1
```
*/
function norm(v) = 0;

/**
[Note: Requires version 2019.05]

Convert a character to a number representing the
<a href="https://en.wikipedia.org/wiki/Unicode" class="external text">Unicode</a>
<a href="https://en.wikipedia.org/wiki/Code_point" class="external text">code point</a>. If the parameter is not a string,
the `ord()` returns `undef`.

**Parameters**

ord(String)  
Convert the first character of the given string to a Unicode code point.

**Examples**

```scad
echo(ord("a"));
// ECHO: 97

echo(ord("BCD"));
// ECHO: 66

echo([for (c = "Hello! 🙂") ord(c)]);
// ECHO: [72, 101, 108, 108, 111, 33, 32, 128578]

txt="1";
echo(ord(txt)-48,txt);
// ECHO: 1,"1" // only converts 1 character
```
*/
function ord(c) = 0;

/**
Mathematical **power** function.

As of version 2021.01 you can use the [exponentiation operator
`^`](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Mathematical_Operators "OpenSCAD User Manual/Mathematical Operators")
instead.

**Parameters**

<base>  
Decimal. Base.

<exponent>  
Decimal. Exponent.

**Usage examples:**

```scad
for (i = [0:5]) {
    translate([i*25,0,0]) {
        cylinder(h = pow(2,i)*5, r=10);
        echo (i, pow(2,i));
    }
}
```

```scad
echo(pow(10,2)); // means 10^2 or 10*10
// result: ECHO: 100

echo(pow(10,3)); // means 10^3 or 10*10*10
// result: ECHO: 1000

echo(pow(125,1/3)); // means 125^(0.333...), which calculates the cube root of 125
// result: ECHO: 5
```
*/
function pow(base, exp) = 0;

/**
Random number generator. Generates a constant vector of pseudo random
numbers, much like an array. The numbers are doubles not integers. When
generating only one number, you still call it with variable[0]

**Parameters**

min_value  
Minimum value of random number range

max_value  
Maximum value of random number range

value_count  
Number of random numbers to return as a vector

seed_value (optional)  
Seed value for random number generator for repeatable results. On
versions before late 2015, seed_value gets rounded to the nearest
integer

**Usage Examples:**

```scad
// get a single number
single_rand = rands(0,10,1)[0];
echo(single_rand);
```

```scad
// get a vector of 4 numbers
seed=42;
random_vect=rands(5,15,4,seed);
echo( "Random Vector: ",random_vect);
sphere(r=5);
for(i=[0:3]) {
    rotate(360*i/4) {
    translate([10+random_vect[i],0,0])
        sphere(r=random_vect[i]/2);
    }
}
// ECHO: "Random Vector: ", [8.7454, 12.9654, 14.5071, 6.83435]
```
*/
function rands(min, max, count, seed_value=0) = 0;

/**
The "round" operator returns the greatest or least integer part,
respectively, if the numeric input is positive or negative.

Some examples:  

```scad
round(x.5) = x+1.

round(x.49) = x.

round(-(x.5)) = -(x+1).

round(-(x.49)) = -x.

round(5.4); //-> 5

round(5.5); //-> 6

round(5.6); //-> 6

```
*/
function round(x) = 0;

function search() = 0;

/**
Mathematical **signum** function. Returns a unit value that extracts the
sign of a value see:
<a href="https://en.wikipedia.org/wiki/Sign_function" class="extiw" title="w:Sign function">Signum function</a>

**Parameters**

<x>  
Decimal. Value to find the sign of.

**Usage examples:**

```scad
sign(-5.0);
sign(0);
sign(8.0);
```

**Results:**

```scad
-1.0
0.0
1.0
```
*/
function sign(x) = 0;

/**
Mathematical **sine** function. See <a href="https://en.wikipedia.org/wiki/Trigonometric_functions#Sine.2C_cosine_and_tangent"
class="extiw" title="w:Trigonometric functions">Sine</a>

**Parameters**


Decimal. Angle in degrees.

```scad
for (i = [0:5]) {
    echo(360*i/6, sin(360*i/6)*80, cos(360*i/6)*80);
        translate([sin(360*i/6)*80, cos(360*i/6)*80, 0 ])
        cylinder(h = 200, r=10);
}
```

```scad
for(i=[0:36])
    translate([i*10,0,0])
        cylinder(r=5,h=sin(i*10)*50+60);
```
![](https://upload.wikimedia.org/wikipedia/commons/thumb/c/c8/OpenSCAD_Sin_Function.png/220px-OpenSCAD_Sin_Function.png)
<a href="/wiki/File:OpenSCAD_Sin_Function.png" class="internal" title="Enlarge"></a>

OpenSCAD Sin Function

*/
function sin(x) = 0;

/**
Mathematical **square root** function.

**Usage Examples:**

```scad
translate([sqrt(100),0,0])sphere(100);
```

## Infinities and NaNs

How does OpenSCAD deal with inputs like (1/0)? Basically, the behavior
is inherited from the language OpenSCAD was written in, the C++
language, and its floating point number types and the associated C math
library. This system allows representation of both positive and negative
infinity by the special values "Inf" or "-Inf". It also allow
representation of creatures like sqrt(-1) or 0/0 as "NaN", an
abbreviation for "Not A Number". Some very nice explanations can be
found on the web, for example the <a href="http://pubs.opengroup.org/onlinepubs/009695399/basedefs/math.h.html"
class="external text">Open Group's site on math.h</a> or <a href="https://en.wikipedia.org/wiki/IEEE_754-1985#Representation_of_non-numbers"
class="extiw" title="w:IEEE 754-1985">Wikipedia's page on the IEEE 754
number format</a>. However OpenSCAD is it's own language so it may not
exactly match everything that happens in C. For example, OpenSCAD uses
degrees instead of radians for trigonometric functions. Another example
is that sin() does not throw a "domain error" when the input is 1/0,
although it does return NaN.

Here are some examples of infinite input to OpenSCAD math functions and
the resulting output, taken from OpenSCAD's regression test system in
late 2015.

<table class="wikitable">
<tbody>
<tr class="odd">
<td>0/0: nan</td>
<td>sin(1/0): nan</td>
<td>asin(1/0): nan</td>
<td>ln(1/0): inf</td>
<td>round(1/0): inf</td>
</tr>
<tr class="even">
<td>-0/0: nan</td>
<td>cos(1/0): nan</td>
<td>acos(1/0): nan</td>
<td>ln(-1/0): nan</td>
<td>round(-1/0): -inf</td>
</tr>
<tr class="odd">
<td>0/-0: nan</td>
<td>tan(1/0): nan</td>
<td>atan(1/0): 90</td>
<td>log(1/0): inf</td>
<td>sign(1/0): 1</td>
</tr>
<tr class="even">
<td>1/0: inf</td>
<td>ceil(-1/0): -inf</td>
<td>atan(-1/0): -90</td>
<td>log(-1/0): nan</td>
<td>sign(-1/0): -1</td>
</tr>
<tr class="odd">
<td>1/-0: -inf</td>
<td>ceil(1/0): inf</td>
<td>atan2(1/0, -1/0): 135</td>
<td>max(-1/0, 1/0): inf</td>
<td>sqrt(1/0): inf</td>
</tr>
<tr class="even">
<td>-1/0: -inf</td>
<td>floor(-1/0): -inf</td>
<td>exp(1/0): inf</td>
<td>min(-1/0, 1/0): -inf</td>
<td>sqrt(-1/0): nan</td>
</tr>
<tr class="odd">
<td>-1/-0: inf</td>
<td>floor(1/0): inf</td>
<td>exp(-1/0): 0</td>
<td>pow(2, 1/0): inf</td>
<td>pow(2, -1/0): 0</td>
</tr>
</tbody>
</table>

Retrieved from
"<https://en.wikibooks.org/w/index.php?title=OpenSCAD_User_Manual/Mathematical_Functions&oldid=4070836>"

[Category](https://en.wikibooks.org/wiki/Special:Categories "Special:Categories"):

-   [Book:OpenSCAD User
    Manual](https://en.wikibooks.org/wiki/Category:Book:OpenSCAD_User_Manual "Category:Book:OpenSCAD User Manual")
*/
function sqrt(x) = 0;

/**
Convert all arguments to strings and concatenate.

**Usage examples:**

```scad
number=2;
echo ("This is ",number,3," and that's it.");
echo (str("This is ",number,3," and that's it."));
```

**Results:**

```scad
ECHO: "This is ", 2, 3, " and that's it."
ECHO: "This is 23 and that's it."
```
*/
function str(args) = 0;

/**
Mathematical **tangent** function. See <a href="https://en.wikipedia.org/wiki/Trigonometric_functions#Sine.2C_cosine_and_tangent"
class="extiw" title="w:Trigonometric functions">Tangent</a>

**Parameters**

  
Decimal. Angle in degrees.

<strong>Usage example:</strong>
```scad
for (i = [0:5]) {
    echo(360*i/6, tan(360*i/6)*80);
        translate([tan(360*i/6)*80, 0, 0 ])
        cylinder(h = 200, r=10);
}
```
*/
function tan(x) = 0;

function version() = 0;

function version_num() = 0;

