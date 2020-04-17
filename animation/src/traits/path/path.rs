use super::point::*;
use super::curve::*;
use super::component::*;

use flo_canvas::*;
use flo_curves::geo::*;
use flo_curves::line::*;
use flo_curves::bezier::*;
use flo_curves::bezier::path::*;

use itertools::*;

use std::iter;
use std::sync::*;

///
/// Represents a vector path
///
#[derive(Clone, Debug)]
pub struct Path {
    pub elements: Arc<Vec<PathComponent>>
}

impl Path {
    ///
    /// Creates a new, empty path
    ///
    pub fn new() -> Path {
        Path { elements: Arc::new(vec![]) }
    }

    ///
    /// Creates a path that contains all of the specified paths merged into one
    ///
    pub fn from_paths<'a, PathIter: IntoIterator<Item=&'a Path>>(paths: PathIter) -> Path {
        // Only paths with > 2 elements can be merged
        let path    = paths.into_iter().filter(|path| path.elements().count() > 2);

        // Flatten the elements into a single path
        let path    = path.flat_map(|path| path.elements());

        // Create a single path from these elements
        let path    = Path::from_elements(path);

        path
    }

    ///
    /// Creates a path from an elements iterator
    ///
    pub fn from_elements<Elements: IntoIterator<Item=PathComponent>>(elements: Elements) -> Path {
        Path {
            elements: Arc::new(elements.into_iter().collect())
        }
    }

    ///
    /// Converts this path to a list of subpaths
    ///
    pub fn to_subpaths(&self) -> Vec<Path> {
        // Each subpath is divided by a move
        let mut paths           = vec![];
        let mut component_iter  = self.elements.iter();
        let mut initial_move    = component_iter.next();

        while let Some(move_component) = initial_move {
            // Start the subpath with the move
            let mut subpath = vec![*move_component];

            // Read until the next move or the end of the path
            loop {
                let next_item = component_iter.next();

                match next_item {
                    Some(PathComponent::Move(_)) => {
                        // This becomes the initial move for the next subpath
                        initial_move = next_item;
                        break;
                    }

                    None        => {
                        // Finished the paths
                        initial_move = None;
                        break;
                    }

                    Some(other) => { subpath.push(*other); }
                }
            }

            // Generate the subpath
            paths.push(Path { elements: Arc::new(subpath) });
        }

        // Return the subpaths
        paths
    }

    ///
    /// The number of elements in this path
    ///
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    ///
    /// Returns the elements that make up this path as an iterator
    ///
    pub fn elements<'a>(&'a self) -> impl 'a+Iterator<Item=PathComponent> {
        self.elements.iter()
            .map(|component| *component)
    }

    ///
    /// Returns references to the elements that make up this path as an iterator
    ///
    pub fn elements_ref<'a>(&'a self) -> impl 'a+Iterator<Item=&'a PathComponent> {
        self.elements.iter()
    }

    ///
    /// Creates a path from an existing collection of components without copying them
    ///
    pub fn from_elements_arc(elements: Arc<Vec<PathComponent>>) -> Path {
        Path {
            elements
        }
    }

    ///
    /// Creates a path from a drawing iterator
    ///
    pub fn from_drawing<Drawing: IntoIterator<Item=Draw>>(drawing: Drawing) -> Path {
        let elements = drawing.into_iter()
            .map(|draw| {
                use self::Draw::*;

                match draw {
                    Move(x, y)              => Some(PathComponent::Move(PathPoint::from((x, y)))),
                    Line(x, y)              => Some(PathComponent::Line(PathPoint::from((x, y)))),
                    BezierCurve(p, c1, c2)  => Some(PathComponent::Bezier(PathPoint::from(p), PathPoint::from(c1), PathPoint::from(c2))),
                    ClosePath               => Some(PathComponent::Close),

                    _                       => None
                }
            })
            .filter(|element| element.is_some())
            .map(|element| element.unwrap());

        Path {
            elements: Arc::new(elements.collect())
        }
    }

    ///
    /// Returns this path as an iterator of draw elements
    ///
    pub fn to_drawing<'a>(&'a self) -> impl 'a+Iterator<Item=Draw> {
        self.elements.iter()
            .map(|path| {
                use self::Draw::*;

                match path {
                    &PathComponent::Move(point)       => Move(point.x(), point.y()),
                    &PathComponent::Line(point)       => Line(point.x(), point.y()),
                    &PathComponent::Bezier(p, c1, c2) => BezierCurve(p.into(), c1.into(), c2.into()),
                    &PathComponent::Close             => ClosePath
                }
            })
    }

    ///
    /// Returns the curves in this path
    ///
    pub fn to_curves<'a>(&'a self) -> impl 'a+Iterator<Item=PathCurve> {
        self.elements.iter()
            .tuple_windows()
            .flat_map(|(prev, next)| {
                use self::PathComponent::*;

                let start_point = match prev {
                    Move(p)          => p,
                    Line(p)          => p,
                    Bezier(p, _, _)  => p,
                    Close            => { return None }
                };

                match next {
                    Move(_)                     => None,
                    Close                       => None,
                    Line(end_point)             => Some(line_to_bezier(&(*start_point, *end_point))),
                    Bezier(end_point, cp1, cp2) => Some(PathCurve::from_points(*start_point, (*cp1, *cp2), *end_point))
                }
            })
    }
}

impl From<(f32, f32)> for PathPoint {
    fn from((x, y): (f32, f32)) -> PathPoint {
        PathPoint::new(x, y)
    }
}

impl From<(f64, f64)> for PathPoint {
    fn from((x, y): (f64, f64)) -> PathPoint {
        PathPoint::new(x as f32, y as f32)
    }
}

impl From<Vec<PathComponent>> for Path {
    fn from(points: Vec<PathComponent>) -> Path {
        Path {
            elements: Arc::new(points)
        }
    }
}

///
/// Converts a drawing into a path (ignoring all the parts of a drawing
/// that cannot be simply converted)
///
impl From<Vec<Draw>> for Path {
    fn from(drawing: Vec<Draw>) -> Path {
        Path::from_drawing(drawing)
    }
}

///
/// Converts a path into a drawing
///
impl<'a> Into<Vec<Draw>> for &'a Path {
    fn into(self) -> Vec<Draw> {
        self.to_drawing().collect()
    }
}

impl Geo for Path {
    type Point = PathPoint;
}

impl BezierPath for Path {
    /// Type of an iterator over the points in this curve. This tuple contains the points ordered as a hull: ie, two control points followed by a point on the curve
    type PointIter = Box<dyn Iterator<Item=(Self::Point, Self::Point, Self::Point)>>;

    ///
    /// Retrieves the initial point of this path
    ///
    fn start_point(&self) -> Self::Point {
        use self::PathComponent::*;

        if self.elements.len() > 0 {
            match self.elements[0] {
                Move(p)         => p,
                Line(p)         => p,
                Bezier(p, _, _) => p,
                Close           => PathPoint::new(0.0, 0.0)
            }
        } else {
            PathPoint::new(0.0, 0.0)
        }
    }

    ///
    /// Retrieves an iterator over the points in this path
    ///
    /// Note that the bezier path trait doesn't support Move or Close operations so these will be ignored.
    /// Operations like `path_contains_point` may be unreliable for 'broken' paths as a result of this limitation.
    /// (Paths with a 'move' in them are really two seperate paths)
    ///
    fn points(&self) -> Self::PointIter {
        // Points as a set of bezier curves
        let mut points = vec![];

        // The last point, which is used to generate the set of points along a line
        let start_point     = self.start_point();
        let mut last_point  = start_point;

        // Convert each element to a point in the path
        for element in self.elements.iter().skip(1) {
            match element {
                PathComponent::Bezier(target, cp1, cp2) => {
                    points.push((*cp1, *cp2, *target));
                    last_point = *target;
                },

                PathComponent::Line(target)  |
                PathComponent::Move(target) => {
                    // Generate control points for a line
                    let diff = *target-last_point;
                    let cp1 = diff * 0.3;
                    let cp2 = diff * 0.7;
                    let cp1 = last_point + cp1;
                    let cp2 = last_point + cp2;

                    points.push((cp1, cp2, *target));
                    last_point = *target;
                },

                PathComponent::Close => {
                    // Line to the start point
                    let diff = start_point-last_point;
                    let cp1 = diff * 0.3;
                    let cp2 = diff * 0.7;
                    let cp1 = last_point + cp1;
                    let cp2 = last_point + cp2;

                    points.push((cp1, cp2, start_point));
                    last_point = start_point;
                }
            }
        }

        // Turn into an iterator
        Box::new(points.into_iter())
    }
}

impl BezierPathFactory for Path {
    ///
    /// Creates a new instance of this path from a set of points
    ///
    fn from_points<FromIter: IntoIterator<Item=(Self::Point, Self::Point, Self::Point)>>(start_point: Self::Point, points: FromIter) -> Self {
        let elements = points.into_iter()
            .map(|(cp1, cp2, target)| PathComponent::Bezier(target, cp1, cp2));
        let elements = iter::once(PathComponent::Move(start_point))
            .chain(elements);

        Path {
            elements: Arc::new(elements.collect())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn remove_interior_points_from_path() {
        use self::PathComponent::*;

        let path = vec![Path { elements: Arc::new(vec![Move(PathPoint { position: (632.2469, 700.9489) }), Bezier(PathPoint { position: (636.50836, 736.8372) }, PathPoint { position: (635.87396, 713.1276) }, PathPoint { position: (634.9844, 724.8036) }), Bezier(PathPoint { position: (636.5715, 737.8543) }, PathPoint { position: (636.5508, 737.1728) }, PathPoint { position: (636.5288, 737.5175) }), Bezier(PathPoint { position: (637.372, 750.5799) }, PathPoint { position: (637.0104, 741.34863) }, PathPoint { position: (637.16614, 746.5887) }), Bezier(PathPoint { position: (637.3977, 752.5185) }, PathPoint { position: (637.3946, 751.0081) }, PathPoint { position: (637.38806, 751.6552) }), Bezier(PathPoint { position: (644.0849, 755.8308) }, PathPoint { position: (633.1192, 754.11414) }, PathPoint { position: (639.5689, 756.6728) }), Bezier(PathPoint { position: (644.19867, 754.36053) }, PathPoint { position: (644.1641, 754.98047) }, PathPoint { position: (644.1864, 754.67413) }), Bezier(PathPoint { position: (644.2086, 754.16125) }, PathPoint { position: (644.20123, 754.29443) }, PathPoint { position: (644.206, 754.228) }), Bezier(PathPoint { position: (644.2215, 751.2548) }, PathPoint { position: (644.2247, 753.1409) }, PathPoint { position: (644.2261, 752.1555) }), Bezier(PathPoint { position: (644.45715, 744.7793) }, PathPoint { position: (644.224, 748.8276) }, PathPoint { position: (644.2046, 746.9262) }), Bezier(PathPoint { position: (641.36017, 741.81305) }, PathPoint { position: (648.3836, 744.23425) }, PathPoint { position: (643.29285, 738.7249) }), Bezier(PathPoint { position: (641.6927, 782.1316) }, PathPoint { position: (642.6393, 748.0454) }, PathPoint { position: (643.9271, 782.0206) }), Bezier(PathPoint { position: (643.7223, 782.1798) }, PathPoint { position: (642.8214, 783.06555) }, PathPoint { position: (645.9634, 783.143) }), Bezier(PathPoint { position: (646.3471, 761.6446) }, PathPoint { position: (642.66925, 776.1191) }, PathPoint { position: (643.24994, 766.54584) }), Bezier(PathPoint { position: (646.70874, 761.53534) }, PathPoint { position: (646.0383, 762.0672) }, PathPoint { position: (646.3682, 761.80475) }), Bezier(PathPoint { position: (643.1715, 762.14465) }, PathPoint { position: (644.1248, 761.1655) }, PathPoint { position: (641.0635, 760.8419) }), Bezier(PathPoint { position: (643.1819, 762.21313) }, PathPoint { position: (643.1719, 762.1477) }, PathPoint { position: (643.17847, 762.19104) }), Bezier(PathPoint { position: (638.2301, 795.27673) }, PathPoint { position: (643.86633, 769.5984) }, PathPoint { position: (642.7505, 789.9731) }), Bezier(PathPoint { position: (638.85767, 794.72217) }, PathPoint { position: (639.1027, 794.62427) }, PathPoint { position: (638.98035, 794.673) }), Bezier(PathPoint { position: (638.1717, 794.9965) }, PathPoint { position: (638.63153, 794.8128) }, PathPoint { position: (638.4038, 794.9035) }), Bezier(PathPoint { position: (645.45856, 767.8199) }, PathPoint { position: (638.3428, 795.2742) }, PathPoint { position: (644.6337, 777.1467) }), Bezier(PathPoint { position: (646.0075, 765.4529) }, PathPoint { position: (645.7012, 766.7749) }, PathPoint { position: (645.9087, 765.9209) }), Bezier(PathPoint { position: (647.0966, 760.214) }, PathPoint { position: (646.3751, 763.7076) }, PathPoint { position: (646.7394, 761.9524) }), Bezier(PathPoint { position: (648.446, 753.7522) }, PathPoint { position: (647.5424, 758.0451) }, PathPoint { position: (647.9867, 755.90405) }), Bezier(PathPoint { position: (644.3893, 749.47437) }, PathPoint { position: (652.64514, 750.74603) }, PathPoint { position: (647.7916, 749.95404) }), Bezier(PathPoint { position: (641.45935, 771.6988) }, PathPoint { position: (645.33276, 755.0831) }, PathPoint { position: (643.25696, 763.2509) }), Bezier(PathPoint { position: (639.52216, 780.95984) }, PathPoint { position: (640.70874, 774.97974) }, PathPoint { position: (639.956, 778.2863) }), Bezier(PathPoint { position: (639.3949, 782.05225) }, PathPoint { position: (639.49023, 781.1691) }, PathPoint { position: (639.43176, 781.6622) }), Bezier(PathPoint { position: (639.6422, 788.78516) }, PathPoint { position: (637.63666, 785.9545) }, PathPoint { position: (639.2575, 789.9833) }), Bezier(PathPoint { position: (642.2427, 789.4523) }, PathPoint { position: (640.9656, 789.9968) }, PathPoint { position: (644.292, 791.5906) }), Bezier(PathPoint { position: (642.27344, 788.3878) }, PathPoint { position: (642.2577, 789.2343) }, PathPoint { position: (642.2846, 788.71204) }), Bezier(PathPoint { position: (643.0503, 772.7711) }, PathPoint { position: (642.0669, 783.21735) }, PathPoint { position: (642.5938, 778.21106) }), Bezier(PathPoint { position: (643.5235, 767.25977) }, PathPoint { position: (643.2073, 770.9551) }, PathPoint { position: (643.3929, 769.0887) }), Bezier(PathPoint { position: (643.73376, 764.4581) }, PathPoint { position: (643.5761, 766.5219) }, PathPoint { position: (643.6471, 765.58673) }), Bezier(PathPoint { position: (641.64185, 710.2857) }, PathPoint { position: (645.10504, 753.14685) }, PathPoint { position: (647.04663, 717.76825) }), Bezier(PathPoint { position: (636.49, 711.1563) }, PathPoint { position: (636.7783, 710.50946) }, PathPoint { position: (636.7587, 710.91833) }), Bezier(PathPoint { position: (635.5495, 714.0062) }, PathPoint { position: (635.7036, 711.7706) }, PathPoint { position: (635.5378, 713.2535) }), Bezier(PathPoint { position: (635.52673, 714.4867) }, PathPoint { position: (635.4977, 712.09265) }, PathPoint { position: (635.52606, 713.5063) }), Line(PathPoint { position: (639.098, 715.4459) }), Bezier(PathPoint { position: (639.9312, 715.1056) }, PathPoint { position: (639.837, 713.6776) }, PathPoint { position: (639.67847, 715.2541) }), Bezier(PathPoint { position: (639.9312, 715.1056) }, PathPoint { position: (639.80084, 713.9517) }, PathPoint { position: (639.6351, 715.4346) }), Bezier(PathPoint { position: (641.39453, 714.15936) }, PathPoint { position: (640.1495, 714.9192) }, PathPoint { position: (640.8361, 714.71277) }), Bezier(PathPoint { position: (637.88934, 764.0092) }, PathPoint { position: (641.60767, 719.1665) }, PathPoint { position: (638.4246, 750.5007) }), Bezier(PathPoint { position: (637.67487, 766.8423) }, PathPoint { position: (637.8036, 765.12506) }, PathPoint { position: (637.72845, 766.09174) }), Bezier(PathPoint { position: (637.2579, 772.2711) }, PathPoint { position: (637.54645, 768.64197) }, PathPoint { position: (637.417, 770.4258) }), Bezier(PathPoint { position: (636.6079, 788.61206) }, PathPoint { position: (636.79565, 777.4734) }, PathPoint { position: (636.3957, 783.20404) }), Bezier(PathPoint { position: (636.60645, 789.2139) }, PathPoint { position: (636.61523, 788.75525) }, PathPoint { position: (636.61755, 788.81494) }), Bezier(PathPoint { position: (640.5524, 794.2531) }, PathPoint { position: (634.46796, 789.1854) }, PathPoint { position: (637.68787, 793.2979) }), Bezier(PathPoint { position: (644.87463, 782.6135) }, PathPoint { position: (643.1684, 792.68353) }, PathPoint { position: (646.117, 783.74805) }), Bezier(PathPoint { position: (644.9533, 781.8391) }, PathPoint { position: (644.9351, 782.05304) }, PathPoint { position: (644.94507, 781.87787) }), Bezier(PathPoint { position: (646.8134, 772.91797) }, PathPoint { position: (645.36383, 779.2981) }, PathPoint { position: (646.0396, 776.33136) }), Bezier(PathPoint { position: (649.786, 748.61017) }, PathPoint { position: (648.66156, 765.3553) }, PathPoint { position: (650.78, 755.1044) }), Bezier(PathPoint { position: (643.1162, 752.6146) }, PathPoint { position: (646.3384, 747.8475) }, PathPoint { position: (640.46545, 748.3663) }), Bezier(PathPoint { position: (641.7667, 759.11865) }, PathPoint { position: (642.654, 754.77997) }, PathPoint { position: (642.2096, 756.96326) }), Bezier(PathPoint { position: (640.68976, 764.33264) }, PathPoint { position: (641.40765, 760.8658) }, PathPoint { position: (641.0556, 762.5961) }), Bezier(PathPoint { position: (640.1838, 766.59705) }, PathPoint { position: (640.6022, 764.7494) }, PathPoint { position: (640.43756, 765.50085) }), Bezier(PathPoint { position: (637.9215, 799.777) }, PathPoint { position: (637.4878, 772.4575) }, PathPoint { position: (634.0093, 796.4725) }), Bezier(PathPoint { position: (640.81964, 799.6173) }, PathPoint { position: (640.3659, 799.79974) }, PathPoint { position: (640.59357, 799.7079) }), Bezier(PathPoint { position: (641.18665, 799.4699) }, PathPoint { position: (640.9423, 799.5681) }, PathPoint { position: (641.0644, 799.51886) }), Bezier(PathPoint { position: (649.3676, 761.2433) }, PathPoint { position: (647.55554, 792.8747) }, PathPoint { position: (650.90564, 768.0388) }), Bezier(PathPoint { position: (649.36664, 761.23566) }, PathPoint { position: (649.3682, 761.2475) }, PathPoint { position: (649.3711, 761.2653) }), Bezier(PathPoint { position: (644.4576, 755.9601) }, PathPoint { position: (651.3629, 761.7767) }, PathPoint { position: (647.61487, 756.77313) }), Bezier(PathPoint { position: (641.8184, 757.4125) }, PathPoint { position: (642.48016, 756.89044) }, PathPoint { position: (642.1489, 757.15106) }), Bezier(PathPoint { position: (637.6301, 783.68835) }, PathPoint { position: (637.28265, 763.3824) }, PathPoint { position: (635.7778, 778.01166) }), Bezier(PathPoint { position: (643.0133, 788.31854) }, PathPoint { position: (635.51465, 783.23224) }, PathPoint { position: (639.78516, 787.83264) }), Bezier(PathPoint { position: (647.0385, 738.4141) }, PathPoint { position: (649.20667, 786.6304) }, PathPoint { position: (652.2765, 743.06934) }), Bezier(PathPoint { position: (637.91504, 744.0158) }, PathPoint { position: (641.2722, 735.0977) }, PathPoint { position: (634.1425, 743.2423) }), Bezier(PathPoint { position: (637.66223, 751.2858) }, PathPoint { position: (637.6356, 746.4281) }, PathPoint { position: (637.63794, 749.1239) }), Bezier(PathPoint { position: (637.6631, 753.9846) }, PathPoint { position: (637.6663, 752.22925) }, PathPoint { position: (637.67645, 753.0867) }), Bezier(PathPoint { position: (637.6596, 754.1043) }, PathPoint { position: (637.66266, 753.9716) }, PathPoint { position: (637.66223, 754.03815) }), Bezier(PathPoint { position: (637.63354, 755.03284) }, PathPoint { position: (637.64734, 754.4176) }, PathPoint { position: (637.6455, 754.7256) }), Bezier(PathPoint { position: (643.8777, 752.4591) }, PathPoint { position: (641.7748, 755.7511) }, PathPoint { position: (648.18933, 754.4743) }), Bezier(PathPoint { position: (643.82465, 750.251) }, PathPoint { position: (643.8726, 751.71405) }, PathPoint { position: (643.85175, 750.79736) }), Bezier(PathPoint { position: (642.3367, 737.1246) }, PathPoint { position: (643.62244, 746.23627) }, PathPoint { position: (642.7793, 740.59534) }), Bezier(PathPoint { position: (642.14453, 736.1245) }, PathPoint { position: (642.29407, 736.788) }, PathPoint { position: (642.187, 736.4604) }), Bezier(PathPoint { position: (633.20514, 700.6628) }, PathPoint { position: (640.6148, 724.007) }, PathPoint { position: (636.82635, 712.7576) })]) }];
        path_remove_interior_points::<_, Path>(&path, 0.01);
    }

    #[test]
    fn to_subpaths_simple() {
        use self::PathComponent::*;

        let path = Path { elements: Arc::new(vec![Move(PathPoint { position: (632.2469, 700.9489) }), Bezier(PathPoint { position: (636.50836, 736.8372) }, PathPoint { position: (635.87396, 713.1276) }, PathPoint { position: (634.9844, 724.8036) }), Bezier(PathPoint { position: (636.5715, 737.8543) }, PathPoint { position: (636.5508, 737.1728) }, PathPoint { position: (636.5288, 737.5175) }), Bezier(PathPoint { position: (637.372, 750.5799) }, PathPoint { position: (637.0104, 741.34863) }, PathPoint { position: (637.16614, 746.5887) }), Bezier(PathPoint { position: (637.3977, 752.5185) }, PathPoint { position: (637.3946, 751.0081) }, PathPoint { position: (637.38806, 751.6552) }), Bezier(PathPoint { position: (644.0849, 755.8308) }, PathPoint { position: (633.1192, 754.11414) }, PathPoint { position: (639.5689, 756.6728) }), Bezier(PathPoint { position: (644.19867, 754.36053) }, PathPoint { position: (644.1641, 754.98047) }, PathPoint { position: (644.1864, 754.67413) }), Bezier(PathPoint { position: (644.2086, 754.16125) }, PathPoint { position: (644.20123, 754.29443) }, PathPoint { position: (644.206, 754.228) }), Bezier(PathPoint { position: (644.2215, 751.2548) }, PathPoint { position: (644.2247, 753.1409) }, PathPoint { position: (644.2261, 752.1555) }), Bezier(PathPoint { position: (644.45715, 744.7793) }, PathPoint { position: (644.224, 748.8276) }, PathPoint { position: (644.2046, 746.9262) }), Bezier(PathPoint { position: (641.36017, 741.81305) }, PathPoint { position: (648.3836, 744.23425) }, PathPoint { position: (643.29285, 738.7249) }), Bezier(PathPoint { position: (641.6927, 782.1316) }, PathPoint { position: (642.6393, 748.0454) }, PathPoint { position: (643.9271, 782.0206) }), Bezier(PathPoint { position: (643.7223, 782.1798) }, PathPoint { position: (642.8214, 783.06555) }, PathPoint { position: (645.9634, 783.143) }), Bezier(PathPoint { position: (646.3471, 761.6446) }, PathPoint { position: (642.66925, 776.1191) }, PathPoint { position: (643.24994, 766.54584) }), Bezier(PathPoint { position: (646.70874, 761.53534) }, PathPoint { position: (646.0383, 762.0672) }, PathPoint { position: (646.3682, 761.80475) }), Bezier(PathPoint { position: (643.1715, 762.14465) }, PathPoint { position: (644.1248, 761.1655) }, PathPoint { position: (641.0635, 760.8419) }), Bezier(PathPoint { position: (643.1819, 762.21313) }, PathPoint { position: (643.1719, 762.1477) }, PathPoint { position: (643.17847, 762.19104) }), Bezier(PathPoint { position: (638.2301, 795.27673) }, PathPoint { position: (643.86633, 769.5984) }, PathPoint { position: (642.7505, 789.9731) }), Bezier(PathPoint { position: (638.85767, 794.72217) }, PathPoint { position: (639.1027, 794.62427) }, PathPoint { position: (638.98035, 794.673) }), Bezier(PathPoint { position: (638.1717, 794.9965) }, PathPoint { position: (638.63153, 794.8128) }, PathPoint { position: (638.4038, 794.9035) }), Bezier(PathPoint { position: (645.45856, 767.8199) }, PathPoint { position: (638.3428, 795.2742) }, PathPoint { position: (644.6337, 777.1467) }), Bezier(PathPoint { position: (646.0075, 765.4529) }, PathPoint { position: (645.7012, 766.7749) }, PathPoint { position: (645.9087, 765.9209) }), Bezier(PathPoint { position: (647.0966, 760.214) }, PathPoint { position: (646.3751, 763.7076) }, PathPoint { position: (646.7394, 761.9524) }), Bezier(PathPoint { position: (648.446, 753.7522) }, PathPoint { position: (647.5424, 758.0451) }, PathPoint { position: (647.9867, 755.90405) }), Bezier(PathPoint { position: (644.3893, 749.47437) }, PathPoint { position: (652.64514, 750.74603) }, PathPoint { position: (647.7916, 749.95404) }), Bezier(PathPoint { position: (641.45935, 771.6988) }, PathPoint { position: (645.33276, 755.0831) }, PathPoint { position: (643.25696, 763.2509) }), Bezier(PathPoint { position: (639.52216, 780.95984) }, PathPoint { position: (640.70874, 774.97974) }, PathPoint { position: (639.956, 778.2863) }), Bezier(PathPoint { position: (639.3949, 782.05225) }, PathPoint { position: (639.49023, 781.1691) }, PathPoint { position: (639.43176, 781.6622) }), Bezier(PathPoint { position: (639.6422, 788.78516) }, PathPoint { position: (637.63666, 785.9545) }, PathPoint { position: (639.2575, 789.9833) }), Bezier(PathPoint { position: (642.2427, 789.4523) }, PathPoint { position: (640.9656, 789.9968) }, PathPoint { position: (644.292, 791.5906) }), Bezier(PathPoint { position: (642.27344, 788.3878) }, PathPoint { position: (642.2577, 789.2343) }, PathPoint { position: (642.2846, 788.71204) }), Bezier(PathPoint { position: (643.0503, 772.7711) }, PathPoint { position: (642.0669, 783.21735) }, PathPoint { position: (642.5938, 778.21106) }), Bezier(PathPoint { position: (643.5235, 767.25977) }, PathPoint { position: (643.2073, 770.9551) }, PathPoint { position: (643.3929, 769.0887) }), Bezier(PathPoint { position: (643.73376, 764.4581) }, PathPoint { position: (643.5761, 766.5219) }, PathPoint { position: (643.6471, 765.58673) }), Bezier(PathPoint { position: (641.64185, 710.2857) }, PathPoint { position: (645.10504, 753.14685) }, PathPoint { position: (647.04663, 717.76825) }), Bezier(PathPoint { position: (636.49, 711.1563) }, PathPoint { position: (636.7783, 710.50946) }, PathPoint { position: (636.7587, 710.91833) }), Bezier(PathPoint { position: (635.5495, 714.0062) }, PathPoint { position: (635.7036, 711.7706) }, PathPoint { position: (635.5378, 713.2535) }), Bezier(PathPoint { position: (635.52673, 714.4867) }, PathPoint { position: (635.4977, 712.09265) }, PathPoint { position: (635.52606, 713.5063) }), Line(PathPoint { position: (639.098, 715.4459) }), Bezier(PathPoint { position: (639.9312, 715.1056) }, PathPoint { position: (639.837, 713.6776) }, PathPoint { position: (639.67847, 715.2541) }), Bezier(PathPoint { position: (639.9312, 715.1056) }, PathPoint { position: (639.80084, 713.9517) }, PathPoint { position: (639.6351, 715.4346) }), Bezier(PathPoint { position: (641.39453, 714.15936) }, PathPoint { position: (640.1495, 714.9192) }, PathPoint { position: (640.8361, 714.71277) }), Bezier(PathPoint { position: (637.88934, 764.0092) }, PathPoint { position: (641.60767, 719.1665) }, PathPoint { position: (638.4246, 750.5007) }), Bezier(PathPoint { position: (637.67487, 766.8423) }, PathPoint { position: (637.8036, 765.12506) }, PathPoint { position: (637.72845, 766.09174) }), Bezier(PathPoint { position: (637.2579, 772.2711) }, PathPoint { position: (637.54645, 768.64197) }, PathPoint { position: (637.417, 770.4258) }), Bezier(PathPoint { position: (636.6079, 788.61206) }, PathPoint { position: (636.79565, 777.4734) }, PathPoint { position: (636.3957, 783.20404) }), Bezier(PathPoint { position: (636.60645, 789.2139) }, PathPoint { position: (636.61523, 788.75525) }, PathPoint { position: (636.61755, 788.81494) }), Bezier(PathPoint { position: (640.5524, 794.2531) }, PathPoint { position: (634.46796, 789.1854) }, PathPoint { position: (637.68787, 793.2979) }), Bezier(PathPoint { position: (644.87463, 782.6135) }, PathPoint { position: (643.1684, 792.68353) }, PathPoint { position: (646.117, 783.74805) }), Bezier(PathPoint { position: (644.9533, 781.8391) }, PathPoint { position: (644.9351, 782.05304) }, PathPoint { position: (644.94507, 781.87787) }), Bezier(PathPoint { position: (646.8134, 772.91797) }, PathPoint { position: (645.36383, 779.2981) }, PathPoint { position: (646.0396, 776.33136) }), Bezier(PathPoint { position: (649.786, 748.61017) }, PathPoint { position: (648.66156, 765.3553) }, PathPoint { position: (650.78, 755.1044) }), Bezier(PathPoint { position: (643.1162, 752.6146) }, PathPoint { position: (646.3384, 747.8475) }, PathPoint { position: (640.46545, 748.3663) }), Bezier(PathPoint { position: (641.7667, 759.11865) }, PathPoint { position: (642.654, 754.77997) }, PathPoint { position: (642.2096, 756.96326) }), Bezier(PathPoint { position: (640.68976, 764.33264) }, PathPoint { position: (641.40765, 760.8658) }, PathPoint { position: (641.0556, 762.5961) }), Bezier(PathPoint { position: (640.1838, 766.59705) }, PathPoint { position: (640.6022, 764.7494) }, PathPoint { position: (640.43756, 765.50085) }), Bezier(PathPoint { position: (637.9215, 799.777) }, PathPoint { position: (637.4878, 772.4575) }, PathPoint { position: (634.0093, 796.4725) }), Bezier(PathPoint { position: (640.81964, 799.6173) }, PathPoint { position: (640.3659, 799.79974) }, PathPoint { position: (640.59357, 799.7079) }), Bezier(PathPoint { position: (641.18665, 799.4699) }, PathPoint { position: (640.9423, 799.5681) }, PathPoint { position: (641.0644, 799.51886) }), Bezier(PathPoint { position: (649.3676, 761.2433) }, PathPoint { position: (647.55554, 792.8747) }, PathPoint { position: (650.90564, 768.0388) }), Bezier(PathPoint { position: (649.36664, 761.23566) }, PathPoint { position: (649.3682, 761.2475) }, PathPoint { position: (649.3711, 761.2653) }), Bezier(PathPoint { position: (644.4576, 755.9601) }, PathPoint { position: (651.3629, 761.7767) }, PathPoint { position: (647.61487, 756.77313) }), Bezier(PathPoint { position: (641.8184, 757.4125) }, PathPoint { position: (642.48016, 756.89044) }, PathPoint { position: (642.1489, 757.15106) }), Bezier(PathPoint { position: (637.6301, 783.68835) }, PathPoint { position: (637.28265, 763.3824) }, PathPoint { position: (635.7778, 778.01166) }), Bezier(PathPoint { position: (643.0133, 788.31854) }, PathPoint { position: (635.51465, 783.23224) }, PathPoint { position: (639.78516, 787.83264) }), Bezier(PathPoint { position: (647.0385, 738.4141) }, PathPoint { position: (649.20667, 786.6304) }, PathPoint { position: (652.2765, 743.06934) }), Bezier(PathPoint { position: (637.91504, 744.0158) }, PathPoint { position: (641.2722, 735.0977) }, PathPoint { position: (634.1425, 743.2423) }), Bezier(PathPoint { position: (637.66223, 751.2858) }, PathPoint { position: (637.6356, 746.4281) }, PathPoint { position: (637.63794, 749.1239) }), Bezier(PathPoint { position: (637.6631, 753.9846) }, PathPoint { position: (637.6663, 752.22925) }, PathPoint { position: (637.67645, 753.0867) }), Bezier(PathPoint { position: (637.6596, 754.1043) }, PathPoint { position: (637.66266, 753.9716) }, PathPoint { position: (637.66223, 754.03815) }), Bezier(PathPoint { position: (637.63354, 755.03284) }, PathPoint { position: (637.64734, 754.4176) }, PathPoint { position: (637.6455, 754.7256) }), Bezier(PathPoint { position: (643.8777, 752.4591) }, PathPoint { position: (641.7748, 755.7511) }, PathPoint { position: (648.18933, 754.4743) }), Bezier(PathPoint { position: (643.82465, 750.251) }, PathPoint { position: (643.8726, 751.71405) }, PathPoint { position: (643.85175, 750.79736) }), Bezier(PathPoint { position: (642.3367, 737.1246) }, PathPoint { position: (643.62244, 746.23627) }, PathPoint { position: (642.7793, 740.59534) }), Bezier(PathPoint { position: (642.14453, 736.1245) }, PathPoint { position: (642.29407, 736.788) }, PathPoint { position: (642.187, 736.4604) }), Bezier(PathPoint { position: (633.20514, 700.6628) }, PathPoint { position: (640.6148, 724.007) }, PathPoint { position: (636.82635, 712.7576) })]) };

        assert!(path.to_subpaths().len() == 1);
    }

    #[test]
    fn to_subpaths_with_extra_move() {
        use self::PathComponent::*;

        let path = Path { elements: Arc::new(vec![
            Move(PathPoint { position: (632.2469, 700.9489) }), Bezier(PathPoint { position: (636.50836, 736.8372) }, PathPoint { position: (635.87396, 713.1276) }, PathPoint { position: (634.9844, 724.8036) }), Bezier(PathPoint { position: (636.5715, 737.8543) }, PathPoint { position: (636.5508, 737.1728) }, PathPoint { position: (636.5288, 737.5175) }), Bezier(PathPoint { position: (637.372, 750.5799) }, PathPoint { position: (637.0104, 741.34863) }, PathPoint { position: (637.16614, 746.5887) }), Bezier(PathPoint { position: (637.3977, 752.5185) }, PathPoint { position: (637.3946, 751.0081) }, PathPoint { position: (637.38806, 751.6552) }), Bezier(PathPoint { position: (644.0849, 755.8308) }, PathPoint { position: (633.1192, 754.11414) }, PathPoint { position: (639.5689, 756.6728) }), Bezier(PathPoint { position: (644.19867, 754.36053) }, PathPoint { position: (644.1641, 754.98047) }, PathPoint { position: (644.1864, 754.67413) }), Bezier(PathPoint { position: (644.2086, 754.16125) }, PathPoint { position: (644.20123, 754.29443) }, PathPoint { position: (644.206, 754.228) }), Bezier(PathPoint { position: (644.2215, 751.2548) }, PathPoint { position: (644.2247, 753.1409) }, PathPoint { position: (644.2261, 752.1555) }), Bezier(PathPoint { position: (644.45715, 744.7793) }, PathPoint { position: (644.224, 748.8276) }, PathPoint { position: (644.2046, 746.9262) }), Bezier(PathPoint { position: (641.36017, 741.81305) }, PathPoint { position: (648.3836, 744.23425) }, PathPoint { position: (643.29285, 738.7249) }), Bezier(PathPoint { position: (641.6927, 782.1316) }, PathPoint { position: (642.6393, 748.0454) }, PathPoint { position: (643.9271, 782.0206) }), Bezier(PathPoint { position: (643.7223, 782.1798) }, PathPoint { position: (642.8214, 783.06555) }, PathPoint { position: (645.9634, 783.143) }), Bezier(PathPoint { position: (646.3471, 761.6446) }, PathPoint { position: (642.66925, 776.1191) }, PathPoint { position: (643.24994, 766.54584) }), Bezier(PathPoint { position: (646.70874, 761.53534) }, PathPoint { position: (646.0383, 762.0672) }, PathPoint { position: (646.3682, 761.80475) }), Bezier(PathPoint { position: (643.1715, 762.14465) }, PathPoint { position: (644.1248, 761.1655) }, PathPoint { position: (641.0635, 760.8419) }), Bezier(PathPoint { position: (643.1819, 762.21313) }, PathPoint { position: (643.1719, 762.1477) }, PathPoint { position: (643.17847, 762.19104) }), Bezier(PathPoint { position: (638.2301, 795.27673) }, PathPoint { position: (643.86633, 769.5984) }, PathPoint { position: (642.7505, 789.9731) }), Bezier(PathPoint { position: (638.85767, 794.72217) }, PathPoint { position: (639.1027, 794.62427) }, PathPoint { position: (638.98035, 794.673) }), Bezier(PathPoint { position: (638.1717, 794.9965) }, PathPoint { position: (638.63153, 794.8128) }, PathPoint { position: (638.4038, 794.9035) }), Bezier(PathPoint { position: (645.45856, 767.8199) }, PathPoint { position: (638.3428, 795.2742) }, PathPoint { position: (644.6337, 777.1467) }), Bezier(PathPoint { position: (646.0075, 765.4529) }, PathPoint { position: (645.7012, 766.7749) }, PathPoint { position: (645.9087, 765.9209) }), Bezier(PathPoint { position: (647.0966, 760.214) }, PathPoint { position: (646.3751, 763.7076) }, PathPoint { position: (646.7394, 761.9524) }), Bezier(PathPoint { position: (648.446, 753.7522) }, PathPoint { position: (647.5424, 758.0451) }, PathPoint { position: (647.9867, 755.90405) }), Bezier(PathPoint { position: (644.3893, 749.47437) }, PathPoint { position: (652.64514, 750.74603) }, PathPoint { position: (647.7916, 749.95404) }), Bezier(PathPoint { position: (641.45935, 771.6988) }, PathPoint { position: (645.33276, 755.0831) }, PathPoint { position: (643.25696, 763.2509) }), Bezier(PathPoint { position: (639.52216, 780.95984) }, PathPoint { position: (640.70874, 774.97974) }, PathPoint { position: (639.956, 778.2863) }), Bezier(PathPoint { position: (639.3949, 782.05225) }, PathPoint { position: (639.49023, 781.1691) }, PathPoint { position: (639.43176, 781.6622) }), Bezier(PathPoint { position: (639.6422, 788.78516) }, PathPoint { position: (637.63666, 785.9545) }, PathPoint { position: (639.2575, 789.9833) }), Bezier(PathPoint { position: (642.2427, 789.4523) }, PathPoint { position: (640.9656, 789.9968) }, PathPoint { position: (644.292, 791.5906) }), Bezier(PathPoint { position: (642.27344, 788.3878) }, PathPoint { position: (642.2577, 789.2343) }, PathPoint { position: (642.2846, 788.71204) }), Bezier(PathPoint { position: (643.0503, 772.7711) }, PathPoint { position: (642.0669, 783.21735) }, PathPoint { position: (642.5938, 778.21106) }), Bezier(PathPoint { position: (643.5235, 767.25977) }, PathPoint { position: (643.2073, 770.9551) }, PathPoint { position: (643.3929, 769.0887) }), Bezier(PathPoint { position: (643.73376, 764.4581) }, PathPoint { position: (643.5761, 766.5219) }, PathPoint { position: (643.6471, 765.58673) }), Bezier(PathPoint { position: (641.64185, 710.2857) }, PathPoint { position: (645.10504, 753.14685) }, PathPoint { position: (647.04663, 717.76825) }), Bezier(PathPoint { position: (636.49, 711.1563) }, PathPoint { position: (636.7783, 710.50946) }, PathPoint { position: (636.7587, 710.91833) }), Bezier(PathPoint { position: (635.5495, 714.0062) }, PathPoint { position: (635.7036, 711.7706) }, PathPoint { position: (635.5378, 713.2535) }), Bezier(PathPoint { position: (635.52673, 714.4867) }, PathPoint { position: (635.4977, 712.09265) }, PathPoint { position: (635.52606, 713.5063) }), Line(PathPoint { position: (639.098, 715.4459) }), Bezier(PathPoint { position: (639.9312, 715.1056) }, PathPoint { position: (639.837, 713.6776) }, PathPoint { position: (639.67847, 715.2541) }), Bezier(PathPoint { position: (639.9312, 715.1056) }, PathPoint { position: (639.80084, 713.9517) }, PathPoint { position: (639.6351, 715.4346) }), Bezier(PathPoint { position: (641.39453, 714.15936) }, PathPoint { position: (640.1495, 714.9192) }, PathPoint { position: (640.8361, 714.71277) }), Bezier(PathPoint { position: (637.88934, 764.0092) }, PathPoint { position: (641.60767, 719.1665) }, PathPoint { position: (638.4246, 750.5007) }), Bezier(PathPoint { position: (637.67487, 766.8423) }, PathPoint { position: (637.8036, 765.12506) }, PathPoint { position: (637.72845, 766.09174) }), Bezier(PathPoint { position: (637.2579, 772.2711) }, PathPoint { position: (637.54645, 768.64197) }, PathPoint { position: (637.417, 770.4258) }), Bezier(PathPoint { position: (636.6079, 788.61206) }, PathPoint { position: (636.79565, 777.4734) }, PathPoint { position: (636.3957, 783.20404) }), Bezier(PathPoint { position: (636.60645, 789.2139) }, PathPoint { position: (636.61523, 788.75525) }, PathPoint { position: (636.61755, 788.81494) }), Bezier(PathPoint { position: (640.5524, 794.2531) }, PathPoint { position: (634.46796, 789.1854) }, PathPoint { position: (637.68787, 793.2979) }), Bezier(PathPoint { position: (644.87463, 782.6135) }, PathPoint { position: (643.1684, 792.68353) }, PathPoint { position: (646.117, 783.74805) }), Bezier(PathPoint { position: (644.9533, 781.8391) }, PathPoint { position: (644.9351, 782.05304) }, PathPoint { position: (644.94507, 781.87787) }), Bezier(PathPoint { position: (646.8134, 772.91797) }, PathPoint { position: (645.36383, 779.2981) }, PathPoint { position: (646.0396, 776.33136) }), Bezier(PathPoint { position: (649.786, 748.61017) }, PathPoint { position: (648.66156, 765.3553) }, PathPoint { position: (650.78, 755.1044) }), Bezier(PathPoint { position: (643.1162, 752.6146) }, PathPoint { position: (646.3384, 747.8475) }, PathPoint { position: (640.46545, 748.3663) }), Bezier(PathPoint { position: (641.7667, 759.11865) }, PathPoint { position: (642.654, 754.77997) }, PathPoint { position: (642.2096, 756.96326) }), Bezier(PathPoint { position: (640.68976, 764.33264) }, PathPoint { position: (641.40765, 760.8658) }, PathPoint { position: (641.0556, 762.5961) }), Bezier(PathPoint { position: (640.1838, 766.59705) }, PathPoint { position: (640.6022, 764.7494) }, PathPoint { position: (640.43756, 765.50085) }), Bezier(PathPoint { position: (637.9215, 799.777) }, PathPoint { position: (637.4878, 772.4575) }, PathPoint { position: (634.0093, 796.4725) }), Bezier(PathPoint { position: (640.81964, 799.6173) }, PathPoint { position: (640.3659, 799.79974) }, PathPoint { position: (640.59357, 799.7079) }), Bezier(PathPoint { position: (641.18665, 799.4699) }, PathPoint { position: (640.9423, 799.5681) }, PathPoint { position: (641.0644, 799.51886) }), Bezier(PathPoint { position: (649.3676, 761.2433) }, PathPoint { position: (647.55554, 792.8747) }, PathPoint { position: (650.90564, 768.0388) }), Bezier(PathPoint { position: (649.36664, 761.23566) }, PathPoint { position: (649.3682, 761.2475) }, PathPoint { position: (649.3711, 761.2653) }), Bezier(PathPoint { position: (644.4576, 755.9601) }, PathPoint { position: (651.3629, 761.7767) }, PathPoint { position: (647.61487, 756.77313) }), Bezier(PathPoint { position: (641.8184, 757.4125) }, PathPoint { position: (642.48016, 756.89044) }, PathPoint { position: (642.1489, 757.15106) }), Bezier(PathPoint { position: (637.6301, 783.68835) }, PathPoint { position: (637.28265, 763.3824) }, PathPoint { position: (635.7778, 778.01166) }), Bezier(PathPoint { position: (643.0133, 788.31854) }, PathPoint { position: (635.51465, 783.23224) }, PathPoint { position: (639.78516, 787.83264) }), Bezier(PathPoint { position: (647.0385, 738.4141) }, PathPoint { position: (649.20667, 786.6304) }, PathPoint { position: (652.2765, 743.06934) }), Bezier(PathPoint { position: (637.91504, 744.0158) }, PathPoint { position: (641.2722, 735.0977) }, PathPoint { position: (634.1425, 743.2423) }), Bezier(PathPoint { position: (637.66223, 751.2858) }, PathPoint { position: (637.6356, 746.4281) }, PathPoint { position: (637.63794, 749.1239) }), Bezier(PathPoint { position: (637.6631, 753.9846) }, PathPoint { position: (637.6663, 752.22925) }, PathPoint { position: (637.67645, 753.0867) }), Bezier(PathPoint { position: (637.6596, 754.1043) }, PathPoint { position: (637.66266, 753.9716) }, PathPoint { position: (637.66223, 754.03815) }), Bezier(PathPoint { position: (637.63354, 755.03284) }, PathPoint { position: (637.64734, 754.4176) }, PathPoint { position: (637.6455, 754.7256) }), Bezier(PathPoint { position: (643.8777, 752.4591) }, PathPoint { position: (641.7748, 755.7511) }, PathPoint { position: (648.18933, 754.4743) }), Bezier(PathPoint { position: (643.82465, 750.251) }, PathPoint { position: (643.8726, 751.71405) }, PathPoint { position: (643.85175, 750.79736) }), Bezier(PathPoint { position: (642.3367, 737.1246) }, PathPoint { position: (643.62244, 746.23627) }, PathPoint { position: (642.7793, 740.59534) }), Bezier(PathPoint { position: (642.14453, 736.1245) }, PathPoint { position: (642.29407, 736.788) }, PathPoint { position: (642.187, 736.4604) }), Bezier(PathPoint { position: (633.20514, 700.6628) }, PathPoint { position: (640.6148, 724.007) }, PathPoint { position: (636.82635, 712.7576) }),
            Move(PathPoint { position: (632.2469, 700.9489) }), Bezier(PathPoint { position: (636.50836, 736.8372) }, PathPoint { position: (635.87396, 713.1276) }, PathPoint { position: (634.9844, 724.8036) }), Bezier(PathPoint { position: (636.5715, 737.8543) }, PathPoint { position: (636.5508, 737.1728) }, PathPoint { position: (636.5288, 737.5175) }), Bezier(PathPoint { position: (637.372, 750.5799) }, PathPoint { position: (637.0104, 741.34863) }, PathPoint { position: (637.16614, 746.5887) }), Bezier(PathPoint { position: (637.3977, 752.5185) }, PathPoint { position: (637.3946, 751.0081) }, PathPoint { position: (637.38806, 751.6552) }), Bezier(PathPoint { position: (644.0849, 755.8308) }, PathPoint { position: (633.1192, 754.11414) }, PathPoint { position: (639.5689, 756.6728) }), Bezier(PathPoint { position: (644.19867, 754.36053) }, PathPoint { position: (644.1641, 754.98047) }, PathPoint { position: (644.1864, 754.67413) }), Bezier(PathPoint { position: (644.2086, 754.16125) }, PathPoint { position: (644.20123, 754.29443) }, PathPoint { position: (644.206, 754.228) }), Bezier(PathPoint { position: (644.2215, 751.2548) }, PathPoint { position: (644.2247, 753.1409) }, PathPoint { position: (644.2261, 752.1555) }), Bezier(PathPoint { position: (644.45715, 744.7793) }, PathPoint { position: (644.224, 748.8276) }, PathPoint { position: (644.2046, 746.9262) }), Bezier(PathPoint { position: (641.36017, 741.81305) }, PathPoint { position: (648.3836, 744.23425) }, PathPoint { position: (643.29285, 738.7249) }), Bezier(PathPoint { position: (641.6927, 782.1316) }, PathPoint { position: (642.6393, 748.0454) }, PathPoint { position: (643.9271, 782.0206) }), Bezier(PathPoint { position: (643.7223, 782.1798) }, PathPoint { position: (642.8214, 783.06555) }, PathPoint { position: (645.9634, 783.143) }), Bezier(PathPoint { position: (646.3471, 761.6446) }, PathPoint { position: (642.66925, 776.1191) }, PathPoint { position: (643.24994, 766.54584) }), Bezier(PathPoint { position: (646.70874, 761.53534) }, PathPoint { position: (646.0383, 762.0672) }, PathPoint { position: (646.3682, 761.80475) }), Bezier(PathPoint { position: (643.1715, 762.14465) }, PathPoint { position: (644.1248, 761.1655) }, PathPoint { position: (641.0635, 760.8419) }), Bezier(PathPoint { position: (643.1819, 762.21313) }, PathPoint { position: (643.1719, 762.1477) }, PathPoint { position: (643.17847, 762.19104) }), Bezier(PathPoint { position: (638.2301, 795.27673) }, PathPoint { position: (643.86633, 769.5984) }, PathPoint { position: (642.7505, 789.9731) }), Bezier(PathPoint { position: (638.85767, 794.72217) }, PathPoint { position: (639.1027, 794.62427) }, PathPoint { position: (638.98035, 794.673) }), Bezier(PathPoint { position: (638.1717, 794.9965) }, PathPoint { position: (638.63153, 794.8128) }, PathPoint { position: (638.4038, 794.9035) }), Bezier(PathPoint { position: (645.45856, 767.8199) }, PathPoint { position: (638.3428, 795.2742) }, PathPoint { position: (644.6337, 777.1467) }), Bezier(PathPoint { position: (646.0075, 765.4529) }, PathPoint { position: (645.7012, 766.7749) }, PathPoint { position: (645.9087, 765.9209) }), Bezier(PathPoint { position: (647.0966, 760.214) }, PathPoint { position: (646.3751, 763.7076) }, PathPoint { position: (646.7394, 761.9524) }), Bezier(PathPoint { position: (648.446, 753.7522) }, PathPoint { position: (647.5424, 758.0451) }, PathPoint { position: (647.9867, 755.90405) }), Bezier(PathPoint { position: (644.3893, 749.47437) }, PathPoint { position: (652.64514, 750.74603) }, PathPoint { position: (647.7916, 749.95404) }), Bezier(PathPoint { position: (641.45935, 771.6988) }, PathPoint { position: (645.33276, 755.0831) }, PathPoint { position: (643.25696, 763.2509) }), Bezier(PathPoint { position: (639.52216, 780.95984) }, PathPoint { position: (640.70874, 774.97974) }, PathPoint { position: (639.956, 778.2863) }), Bezier(PathPoint { position: (639.3949, 782.05225) }, PathPoint { position: (639.49023, 781.1691) }, PathPoint { position: (639.43176, 781.6622) }), Bezier(PathPoint { position: (639.6422, 788.78516) }, PathPoint { position: (637.63666, 785.9545) }, PathPoint { position: (639.2575, 789.9833) }), Bezier(PathPoint { position: (642.2427, 789.4523) }, PathPoint { position: (640.9656, 789.9968) }, PathPoint { position: (644.292, 791.5906) }), Bezier(PathPoint { position: (642.27344, 788.3878) }, PathPoint { position: (642.2577, 789.2343) }, PathPoint { position: (642.2846, 788.71204) }), Bezier(PathPoint { position: (643.0503, 772.7711) }, PathPoint { position: (642.0669, 783.21735) }, PathPoint { position: (642.5938, 778.21106) }), Bezier(PathPoint { position: (643.5235, 767.25977) }, PathPoint { position: (643.2073, 770.9551) }, PathPoint { position: (643.3929, 769.0887) }), Bezier(PathPoint { position: (643.73376, 764.4581) }, PathPoint { position: (643.5761, 766.5219) }, PathPoint { position: (643.6471, 765.58673) }), Bezier(PathPoint { position: (641.64185, 710.2857) }, PathPoint { position: (645.10504, 753.14685) }, PathPoint { position: (647.04663, 717.76825) }), Bezier(PathPoint { position: (636.49, 711.1563) }, PathPoint { position: (636.7783, 710.50946) }, PathPoint { position: (636.7587, 710.91833) }), Bezier(PathPoint { position: (635.5495, 714.0062) }, PathPoint { position: (635.7036, 711.7706) }, PathPoint { position: (635.5378, 713.2535) }), Bezier(PathPoint { position: (635.52673, 714.4867) }, PathPoint { position: (635.4977, 712.09265) }, PathPoint { position: (635.52606, 713.5063) }), Line(PathPoint { position: (639.098, 715.4459) }), Bezier(PathPoint { position: (639.9312, 715.1056) }, PathPoint { position: (639.837, 713.6776) }, PathPoint { position: (639.67847, 715.2541) }), Bezier(PathPoint { position: (639.9312, 715.1056) }, PathPoint { position: (639.80084, 713.9517) }, PathPoint { position: (639.6351, 715.4346) }), Bezier(PathPoint { position: (641.39453, 714.15936) }, PathPoint { position: (640.1495, 714.9192) }, PathPoint { position: (640.8361, 714.71277) }), Bezier(PathPoint { position: (637.88934, 764.0092) }, PathPoint { position: (641.60767, 719.1665) }, PathPoint { position: (638.4246, 750.5007) }), Bezier(PathPoint { position: (637.67487, 766.8423) }, PathPoint { position: (637.8036, 765.12506) }, PathPoint { position: (637.72845, 766.09174) }), Bezier(PathPoint { position: (637.2579, 772.2711) }, PathPoint { position: (637.54645, 768.64197) }, PathPoint { position: (637.417, 770.4258) }), Bezier(PathPoint { position: (636.6079, 788.61206) }, PathPoint { position: (636.79565, 777.4734) }, PathPoint { position: (636.3957, 783.20404) }), Bezier(PathPoint { position: (636.60645, 789.2139) }, PathPoint { position: (636.61523, 788.75525) }, PathPoint { position: (636.61755, 788.81494) }), Bezier(PathPoint { position: (640.5524, 794.2531) }, PathPoint { position: (634.46796, 789.1854) }, PathPoint { position: (637.68787, 793.2979) }), Bezier(PathPoint { position: (644.87463, 782.6135) }, PathPoint { position: (643.1684, 792.68353) }, PathPoint { position: (646.117, 783.74805) }), Bezier(PathPoint { position: (644.9533, 781.8391) }, PathPoint { position: (644.9351, 782.05304) }, PathPoint { position: (644.94507, 781.87787) }), Bezier(PathPoint { position: (646.8134, 772.91797) }, PathPoint { position: (645.36383, 779.2981) }, PathPoint { position: (646.0396, 776.33136) }), Bezier(PathPoint { position: (649.786, 748.61017) }, PathPoint { position: (648.66156, 765.3553) }, PathPoint { position: (650.78, 755.1044) }), Bezier(PathPoint { position: (643.1162, 752.6146) }, PathPoint { position: (646.3384, 747.8475) }, PathPoint { position: (640.46545, 748.3663) }), Bezier(PathPoint { position: (641.7667, 759.11865) }, PathPoint { position: (642.654, 754.77997) }, PathPoint { position: (642.2096, 756.96326) }), Bezier(PathPoint { position: (640.68976, 764.33264) }, PathPoint { position: (641.40765, 760.8658) }, PathPoint { position: (641.0556, 762.5961) }), Bezier(PathPoint { position: (640.1838, 766.59705) }, PathPoint { position: (640.6022, 764.7494) }, PathPoint { position: (640.43756, 765.50085) }), Bezier(PathPoint { position: (637.9215, 799.777) }, PathPoint { position: (637.4878, 772.4575) }, PathPoint { position: (634.0093, 796.4725) }), Bezier(PathPoint { position: (640.81964, 799.6173) }, PathPoint { position: (640.3659, 799.79974) }, PathPoint { position: (640.59357, 799.7079) }), Bezier(PathPoint { position: (641.18665, 799.4699) }, PathPoint { position: (640.9423, 799.5681) }, PathPoint { position: (641.0644, 799.51886) }), Bezier(PathPoint { position: (649.3676, 761.2433) }, PathPoint { position: (647.55554, 792.8747) }, PathPoint { position: (650.90564, 768.0388) }), Bezier(PathPoint { position: (649.36664, 761.23566) }, PathPoint { position: (649.3682, 761.2475) }, PathPoint { position: (649.3711, 761.2653) }), Bezier(PathPoint { position: (644.4576, 755.9601) }, PathPoint { position: (651.3629, 761.7767) }, PathPoint { position: (647.61487, 756.77313) }), Bezier(PathPoint { position: (641.8184, 757.4125) }, PathPoint { position: (642.48016, 756.89044) }, PathPoint { position: (642.1489, 757.15106) }), Bezier(PathPoint { position: (637.6301, 783.68835) }, PathPoint { position: (637.28265, 763.3824) }, PathPoint { position: (635.7778, 778.01166) }), Bezier(PathPoint { position: (643.0133, 788.31854) }, PathPoint { position: (635.51465, 783.23224) }, PathPoint { position: (639.78516, 787.83264) }), Bezier(PathPoint { position: (647.0385, 738.4141) }, PathPoint { position: (649.20667, 786.6304) }, PathPoint { position: (652.2765, 743.06934) }), Bezier(PathPoint { position: (637.91504, 744.0158) }, PathPoint { position: (641.2722, 735.0977) }, PathPoint { position: (634.1425, 743.2423) }), Bezier(PathPoint { position: (637.66223, 751.2858) }, PathPoint { position: (637.6356, 746.4281) }, PathPoint { position: (637.63794, 749.1239) }), Bezier(PathPoint { position: (637.6631, 753.9846) }, PathPoint { position: (637.6663, 752.22925) }, PathPoint { position: (637.67645, 753.0867) }), Bezier(PathPoint { position: (637.6596, 754.1043) }, PathPoint { position: (637.66266, 753.9716) }, PathPoint { position: (637.66223, 754.03815) }), Bezier(PathPoint { position: (637.63354, 755.03284) }, PathPoint { position: (637.64734, 754.4176) }, PathPoint { position: (637.6455, 754.7256) }), Bezier(PathPoint { position: (643.8777, 752.4591) }, PathPoint { position: (641.7748, 755.7511) }, PathPoint { position: (648.18933, 754.4743) }), Bezier(PathPoint { position: (643.82465, 750.251) }, PathPoint { position: (643.8726, 751.71405) }, PathPoint { position: (643.85175, 750.79736) }), Bezier(PathPoint { position: (642.3367, 737.1246) }, PathPoint { position: (643.62244, 746.23627) }, PathPoint { position: (642.7793, 740.59534) }), Bezier(PathPoint { position: (642.14453, 736.1245) }, PathPoint { position: (642.29407, 736.788) }, PathPoint { position: (642.187, 736.4604) }), Bezier(PathPoint { position: (633.20514, 700.6628) }, PathPoint { position: (640.6148, 724.007) }, PathPoint { position: (636.82635, 712.7576) })
        ]) };

        assert!(path.to_subpaths().len() == 2);
    }
}
