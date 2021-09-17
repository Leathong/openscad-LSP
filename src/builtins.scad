// Leaf modules.
module children(index) {}
module circle(rad) {}
module cube(size, center) {}
module cylinder(r) {}
module echo(msgn) {}
module import(file, center=false, dpi=96, convexity=1) {}
module polygon(pts) {}
module polyhedron(points, faces, convexity=1) {}
module sphere(rad) {}
module square(size) {}
module surface(file, center=false, invert=false, convexity=1) {}
module text(args) {}

// Group modules.
module color(c) { /* group */ }
module difference() { /* group */ }
module group() { /* group */ }
module hull() { /* group */ }
module intersection() { /* group */ }
module linear_extrude(height, center=false, convexity=10, twist=0, slices=20, scale=1.0) { /* group */ }
module minkowski() { /* group */ }
module mirror(v) { /* group */ }
module multmatrix(m) { /* group */ }
module offset(delta, r=0, chamfer=false) { /* group */ }
module parent_module() { /* group */ }
module projection() { /* group */ }
module render() { /* group */ }
module resize(newsize) { /* group */ }
module rotate(angles) { /* group */ }
module rotate_extrude(angle=360, convexity=2) { /* group */ }
module scale(v) { /* group */ }
module translate(v) { /* group */ }
module union() { /* group */ }

// Functions.
function abs(x) = 0;
function acos(x) = 0;
function asin(x) = 0;
function assert(cond) = 0;
function atan(x) = 0;
function atan2(y, x) = 0;
function ceil(x) = 0;
function chr(x) = 0;
function concat(args) = 0;
function cos(x) = 0;
function cross(u, v) = 0;
function dxf_cross() = 0;
function dxf_dim() = 0;
function exp(x) = 0;
function floor(x) = 0;
function is_bool(x) = 0;
function is_list(x) = 0;
function is_num(x) = 0;
function is_string(x) = 0;
function is_undef(x) = 0;
function len(x) = 0;
function ln(x) = 0;
function log(x) = 0;
function lookup(key, vals) = 0;
function max(args) = 0;
function min(args) = 0;
function norm(v) = 0;
function ord(c) = 0;
function pow(base, exp) = 0;
function rands(min, max, count, seed_value=0) = 0;
function round(x) = 0;
function search() = 0;
function sign(x) = 0;
function sin(x) = 0;
function sqrt(x) = 0;
function str(args) = 0;
function tan(x) = 0;
function version() = 0;
function version_num() = 0;
