```toml
flo_curves = "0.2"
```

flo_curves
==========

`flo_curves` is a library providing routines for manipulating various types of curve, particularly Bezier curves. It provides
a grab-bag of algorithms, from the basis functions for generating points on a curve to collisions, fitting to points and even 
path arithmetic. It is built around traits, which makes it easy to use the provided algorithms with any data structure, though 
some defaults are provided.

`flo_curves` is designed as a support library for `flowbetween`, an animation tool I'm working on, but is also designed to work
stand-alone.

Examples
========

Creating a curve:

```Rust
use flo_curves::*;
use flo_curves::bezier;

let curve = bezier::Curve::from_points(Coord2(1.0, 2.0), (Coord2(2.0, 0.0), Coord2(3.0, 5.0)), Coord2(4.0, 2.0));
```

Finding a point on a curve:

```Rust
use flo_curves::bezier;

let pos = curve.point_at_pos(0.5);
```

Intersections:

```Rust
use flo_curves::bezier;

for (t1, t2) in bezier::curve_intersects_curve_clip(curve1, curve2) {
    let pos = curve1.point_at_pos(t1);
    println!("Intersection, curve1 t: {}, curve2 t: {}, position: {}, {}", t1, t2, pos.x(), pos.y());
}
```

Creating a path:

```Rust
use flo_curves::bezier;
use flo_curves::bezier::path::*;

let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
    .line_to(Coord2(5.0, 1.0))
    .line_to(Coord2(5.0, 5.0))
    .line_to(Coord2(1.0, 5.0))
    .line_to(Coord2(1.0, 1.0))
    .build();
```

Path artihmetic:

```Rust
use flo_curves::bezier::path::*;

let rectangle_with_hole = path_sub::<_,_,_, SimpleBezierPath>(&vec![rectangle], &vec![circle])
```
