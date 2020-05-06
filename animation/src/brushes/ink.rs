use super::super::traits::*;
use super::super::raycast::*;

use std::iter;
use std::ops::*;
use std::sync::*;

use flo_curves::*;
use flo_curves::bezier;
use flo_canvas::*;

// Minimum distance between points to use to fit to a curve
const MIN_DISTANCE: f64 = 2.0;

// Scaling used on the pressure for coordinate fitting
// (As we fit to a distance of 1.0, having a pressure of 0-1 means the curve fitter can use essentially any pressure)
const INK_PRESSURE_SCALE: f64 = 50.0;

///
/// The ink brush draws a solid line with width based on pressure
///
pub struct InkBrush {
    /// The blend mode that this brush will use
    blend_mode: BlendMode,

    /// Width at pressure 0%
    min_width: f32,

    /// Width at pressure 100%
    max_width: f32,

    // Distance to scale up at the start of the brush stroke
    scale_up_distance: f32
}

impl InkBrush {
    ///
    /// Creates a new ink brush with the default settings
    ///
    pub fn new(definition: &InkDefinition, drawing_style: BrushDrawingStyle) -> Self {
        use BrushDrawingStyle::*;

        let blend_mode = match drawing_style {
            Draw    => BlendMode::SourceOver,
            Erase   => BlendMode::DestinationOut
        };

        Self {
            blend_mode,
            min_width:          definition.min_width,
            max_width:          definition.max_width,
            scale_up_distance:  definition.scale_up_distance
        }
    }
}

///
/// Ink brush coordinate (used for curve fitting)
///
#[derive(Clone, Copy, PartialEq)]
struct InkCoord {
    x: f64,
    y: f64,
    pressure: f64
}

impl InkCoord {
    pub fn pressure(&self) -> f64 { self.pressure }
    pub fn set_pressure(&mut self, new_pressure: f64) {
        self.pressure = new_pressure;
    }

    pub fn to_coord2(&self) -> (Coord2, f64) {
        (Coord2(self.x, self.y), self.pressure)
    }
}

impl<'a> From<&'a RawPoint> for InkCoord {
    fn from(src: &'a RawPoint) -> Self {
        Self {
            x: src.position.0 as f64,
            y: src.position.1 as f64,
            pressure: (src.pressure as f64)*INK_PRESSURE_SCALE
        }
    }
}

impl<'a> From<&'a BrushPoint> for InkCoord {
    fn from(src: &'a BrushPoint) -> Self {
        Self {
            x: src.position.0 as f64,
            y: src.position.1 as f64,
            pressure: (src.width as f64)*INK_PRESSURE_SCALE
        }
    }
}

impl Add<InkCoord> for InkCoord {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            pressure: self.pressure + rhs.pressure
        }
    }
}

impl Sub<InkCoord> for InkCoord {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            pressure: self.pressure - rhs.pressure
        }
    }
}

impl Mul<f64> for InkCoord {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            pressure: self.pressure * rhs
        }
    }
}

impl Coordinate for InkCoord {
    #[inline]
    fn from_components(components: &[f64]) -> Self {
        Self { x: components[0], y: components[1], pressure: components[2] }
    }

    #[inline]
    fn origin() -> Self {
        Self { x: 0.0, y: 0.0, pressure: 0.0 }
    }

    #[inline]
    fn len() -> usize { 3 }

    #[inline]
    fn get(&self, index: usize) -> f64 {
        match index {
            0 => self.x,
            1 => self.y,
            2 => self.pressure,
            _ => panic!("InkCoord only has three components")
        }
    }

    fn from_biggest_components(p1: Self, p2: Self) -> Self {
        Self {
            x: f64::from_biggest_components(p1.x, p2.x),
            y: f64::from_biggest_components(p1.y, p2.y),
            pressure: f64::from_biggest_components(p1.pressure, p2.pressure)
        }
    }

    fn from_smallest_components(p1: Self, p2: Self) -> Self {
        Self {
            x: f64::from_smallest_components(p1.x, p2.x),
            y: f64::from_smallest_components(p1.y, p2.y),
            pressure: f64::from_smallest_components(p1.pressure, p2.pressure)
        }
    }

    #[inline]
    fn distance_to(&self, target: &Self) -> f64 {
        let dist_x = target.x-self.x;
        let dist_y = target.y-self.y;
        let dist_p = target.pressure-self.pressure;

        f64::sqrt(dist_x*dist_x + dist_y*dist_y + dist_p*dist_p)
    }

    #[inline]
    fn dot(&self, target: &Self) -> f64 {
        self.x*target.x + self.y*target.y + self.pressure*target.pressure
    }
}

///
/// Bezier curve using InkCoords
///
#[derive(Clone, Copy)]
struct InkCurve {
    pub start_point:    InkCoord,
    pub end_point:      InkCoord,
    pub control_points: (InkCoord, InkCoord)
}

impl InkCurve {
    ///
    /// Creates an ink curve from brush points
    ///
    pub fn from_brush_points(last_point: &BrushPoint, next_point: &BrushPoint) -> Self {
        Self {
            start_point:    InkCoord { x: last_point.position.0 as f64, y: last_point.position.1 as f64, pressure: last_point.width as f64 },
            end_point:      InkCoord { x: next_point.position.0 as f64, y: next_point.position.1 as f64, pressure: next_point.width as f64 },
            control_points: (
                InkCoord { x: next_point.cp1.0 as f64, y: next_point.cp1.1 as f64, pressure: next_point.width as f64 },
                InkCoord { x: next_point.cp2.0 as f64, y: next_point.cp2.1 as f64, pressure: next_point.width as f64 }
            )
        }
    }

    ///
    /// Converts to a pair of offset curves
    ///
    pub fn to_offset_curves(&self, min_width: f64, max_width: f64) -> (Vec<bezier::Curve<Coord2>>, Vec<bezier::Curve<Coord2>>) {
        // Fetch the coordinates for the offset curve
        let (start, start_pressure) = self.start_point().to_coord2();
        let (end, end_pressure)     = self.end_point().to_coord2();
        let cp1                     = self.control_points.0.to_coord2().0;
        let cp2                     = self.control_points.1.to_coord2().0;

        // Create the top and bottom offsets
        let start_offset    = start_pressure*(max_width-min_width) + min_width;
        let end_offset      = end_pressure*(max_width-min_width) + min_width;
        let base_curve      = bezier::Curve::from_points(start, (cp1, cp2), end);

        let offset_up       = bezier::offset(&base_curve, start_offset, end_offset);
        let offset_down     = bezier::offset(&base_curve, -start_offset, -end_offset);

        (offset_up, offset_down)
    }
}

impl Geo for InkCurve {
    type Point = InkCoord;
}

impl BezierCurveFactory for InkCurve {
    fn from_points(start: InkCoord, control_points: (InkCoord, InkCoord), end: InkCoord) -> Self {
        Self {
            start_point:    start,
            end_point:      end,
            control_points: control_points
        }
    }
}

impl BezierCurve for InkCurve {
    #[inline]
    fn start_point(&self) -> InkCoord {
        self.start_point
    }

    #[inline]
    fn end_point(&self) -> InkCoord {
        self.end_point
    }

    #[inline]
    fn control_points(&self) -> (InkCoord, InkCoord) {
        self.control_points
    }
}

impl Brush for InkBrush {
    fn brush_points_for_raw_points(&self, points: &[RawPoint]) -> Vec<BrushPoint> {
        // Nothing to draw if there are no points in the brush stroke (or only one point)
        if points.len() <= 2 {
            return vec![];
        }

        // Convert points to ink points
        let ink_points: Vec<_> = points.iter().map(|point| InkCoord::from(point)).collect();

        // Average points that are very close together so we don't overdo
        // the curve fitting
        let mut averaged_points = vec![];
        let mut last_point      = ink_points[0];
        averaged_points.push(last_point);

        for point in ink_points.iter().skip(1) {
            // If the distance between this point and the last one is below a
            // threshold, average them together
            let distance = last_point.distance_to(point);

            if distance < MIN_DISTANCE {
                // Average this point with the previous average
                // TODO: (We should really total up the number of points we're
                // averaging over)
                let num_averaged    = averaged_points.len();
                let current_average = averaged_points[num_averaged-1];
                let averaged_point  = (current_average + last_point) * 0.5;

                // Update the earlier point (and don't update last_point: we'll
                // keep averaging until we find a new point far enough away)
                averaged_points[num_averaged-1] = averaged_point;
            } else {
                // Keep this point
                averaged_points.push(*point);

                // Update the last point
                last_point = *point;
            }
        }

        // Smooth out the points to remove any jitteryness
        let mut ink_points = InkCoord::smooth(&averaged_points, &[0.1, 0.25, 0.3, 0.25, 0.1]);

        // Scale up the pressure at the start of the brush stroke
        let mut distance    = 0.0;
        let mut last_point  = ink_points[0];
        let scale_up_distance = self.scale_up_distance as f64;
        for point in ink_points.iter_mut() {
            // Add to the distance
            distance += last_point.distance_to(point);
            last_point = *point;

            // Scale the pressure by the distance
            if distance > scale_up_distance { break; }

            let pressure = point.pressure();
            point.set_pressure(pressure * (distance/scale_up_distance));
        }

        /*
         * -- TODO: needs its own config option
         *
        // Scale down the pressure at the end of the brush stroke
        last_point  = *ink_points.last().unwrap();
        distance    = 0.0;
        for index in (0..ink_points.len()).rev() {
            let point = &mut ink_points[index];

            distance += last_point.distance_to(point);
            last_point = *point;

            // Scale the pressure by the distance
            if distance > scale_up_distance { break; }

            let pressure = point.pressure();
            point.set_pressure(pressure * (distance/scale_up_distance));
        }
        */

        // Fit these points to a curve
        let curve = InkCurve::fit_from_points(&ink_points, 1.0);

        // Turn into brush points
        let mut brush_points = vec![];

        if let Some(curve) = curve {
            // First point is the start point, the control points don't matter for this
            let start = curve[0].start_point();
            brush_points.push(BrushPoint {
                position:   (start.x as f32, start.y as f32),
                cp1:        (0.0, 0.0),
                cp2:        (0.0, 0.0),
                width:      (start.pressure/INK_PRESSURE_SCALE) as f32
            });

            // Convert the remaining curve segments
            for segment in curve {
                let end             = segment.end_point();
                let (cp1, cp2)      = segment.control_points();

                brush_points.push(BrushPoint {
                    position:   (end.x as f32, end.y as f32),
                    cp1:        (cp1.x as f32, cp1.y as f32),
                    cp2:        (cp2.x as f32, cp2.y as f32),
                    width:      (end.pressure/INK_PRESSURE_SCALE) as f32
                });
            }
        }

        brush_points
    }

    fn prepare_to_render<'a>(&'a self, properties: &BrushProperties) -> Box<dyn 'a+Iterator<Item=Draw>> {
        Box::new(vec![
            Draw::BlendMode(self.blend_mode),
            Draw::FillColor(properties.color.with_alpha(properties.opacity))
        ].into_iter())
    }

    fn render_brush<'a>(&'a self, properties: &'a BrushProperties, points: &'a Vec<BrushPoint>, transform: Arc<Vec<Transformation>>) -> Box<dyn 'a+Iterator<Item=Draw>> {
        let size_ratio = properties.size / self.max_width;

        // Nothing to do if there are too few points
        if points.len() < 2 {
            return Box::new(iter::empty());
        }

        // Create an ink curve from the brush points
        let mut curve       = vec![];
        let mut last_point  = &points[0];

        for brush_point in points.iter().skip(1) {
            curve.push(InkCurve::from_brush_points(last_point, brush_point));
            last_point = brush_point;
        }

        // Transform the ink curve if needed
        if transform.len() > 0 {
            for index in 0..curve.len() {
                for transform in transform.iter() {
                    curve[index] = transform.transform_curve(&curve[index]);
                }
            }
        }

        // Draw a variable width line for this curve
        let (upper_curves, lower_curves): (Vec<_>, Vec<_>) = curve.into_iter()
            .map(|ink_curve| ink_curve.to_offset_curves((self.min_width*size_ratio) as f64, (self.max_width*size_ratio) as f64))
            .unzip();

        // Upper portion
        let Coord2(x, y) = upper_curves[0][0].start_point();
        let preamble = vec![
            Draw::NewPath,
            Draw::Move(x as f32, y as f32)
        ];

        let upper_curves = upper_curves.into_iter()
            .flat_map(|curve_list|  curve_list.into_iter())
            .map(|curve_section|    Draw::from(&curve_section));

        // Lower portion (reverse everything)
        let Coord2(x, y)    = {
            let last_section    = &lower_curves[lower_curves.len()-1];
            let last_curve      = &last_section[last_section.len()-1];
            last_curve.end_point()
        };

        let end_cap = Draw::Line(x as f32, y as f32);

        let lower_curves = lower_curves.into_iter()
            .rev()
            .flat_map(|curve_list|  curve_list.into_iter().rev())
            .map(|curve_section|    Draw::from(&curve_section.reverse::<bezier::Curve<_>>()));

        // Finish up
        let finish = Draw::Fill;

        // Assemble the final set of instructions
        let draw_brush = preamble.into_iter()
            .chain(upper_curves)
            .chain(iter::once(end_cap))
            .chain(lower_curves)
            .chain(iter::once(finish));

        Box::new(draw_brush)
    }

    ///
    /// Retrieves the definition for this brush
    ///
    fn to_definition(&self) -> (BrushDefinition, BrushDrawingStyle) {
        let definition = BrushDefinition::Ink(InkDefinition {
            min_width:          self.min_width,
            max_width:          self.max_width,
            scale_up_distance:  self.scale_up_distance
        });

        let drawing_style = match self.blend_mode {
            BlendMode::DestinationOut   => BrushDrawingStyle::Erase,
            _                           => BrushDrawingStyle::Draw
        };

        (definition, drawing_style)
    }

    ///
    /// Attempts to combine this brush stroke with the specified vector element. Returns the combined element if successful
    ///
    fn combine_with(&self, combined_element: &Vector, combined_element_properties: &VectorProperties, next_element: &Vector, next_element_properties: &VectorProperties) -> CombineResult {
        // The ink brush always combines into a group: retrieve that as the combined element here
        let combined_element = match combined_element {
            Vector::Group(group_element)    => group_element.clone(),
            other                           => {
                let mut other = other.clone();
                other.set_id(ElementId::Unassigned);
                GroupElement::new(ElementId::Unassigned, GroupType::Added, Arc::new(vec![other]))
            }
        };

        if combined_element_properties.brush_properties == next_element_properties.brush_properties {
            match next_element {
                Vector::BrushStroke(_) | Vector::Path(_) => {
                    // Add to brush strokes or paths if possible
                    let src_path = next_element.to_path(next_element_properties, PathConversion::RemoveInteriorPoints);
                    let tgt_path = combined_element.to_path(combined_element_properties, PathConversion::RemoveInteriorPoints);

                    // Try to combine with the path
                    if let (Some(src_path), Some(tgt_path)) = (src_path, tgt_path) {
                        let src_path = src_path.into_iter().flat_map(|p| p.to_subpaths()).collect();
                        let tgt_path = tgt_path.into_iter().flat_map(|p| p.to_subpaths()).collect();

                        let combined = combine_paths(&src_path, &tgt_path, 0.01);
                        if let Some(mut combined) = combined {
                            // Managed to combine the two brush strokes/paths into one: add to the group. Note that the 'next' element is combined underneath the current element.
                            let previous_elements   = combined_element.elements().cloned();
                            let grouped_elements    = iter::once(next_element.clone()).chain(previous_elements).collect();

                            let mut grouped         = GroupElement::new(ElementId::Unassigned, GroupType::Added, Arc::new(grouped_elements));

                            // In checking for an overlap we will have calculated most of the combined path: finish the job and set it as the hint
                            combined.set_exterior_by_adding();
                            combined.heal_exterior_gaps();

                            let combined_path = combined.exterior_paths();
                            let combined_path = vec![Path::from_paths(&combined_path)];
                            grouped.set_hint_path(Arc::new(combined_path));

                            CombineResult::NewElement(Vector::Group(grouped))
                        } else {
                            // Elements do not overlap
                            CombineResult::NoOverlap
                        }
                    } else {
                        // No source path
                        CombineResult::NoOverlap
                    }
                },

                Vector::Group(group) => {
                    // We can add this path to a group if it's already an 'add' group
                    if group.group_type() != GroupType::Added {
                        // Can't currently combine with non-added groups
                        // TODO: we *can* by creating an added subgroup, but for now we won't
                        CombineResult::UnableToCombineFurther
                    } else {
                        // Combine if the path for this group will add up
                        let src_path = group.to_path(next_element_properties, PathConversion::RemoveInteriorPoints);
                        let tgt_path = combined_element.to_path(combined_element_properties, PathConversion::RemoveInteriorPoints);

                        // Try to combine with the path
                        if let (Some(src_path), Some(tgt_path)) = (src_path, tgt_path) {
                            let src_path = src_path.into_iter().flat_map(|p| p.to_subpaths()).collect();
                            let tgt_path = tgt_path.into_iter().flat_map(|p| p.to_subpaths()).collect();

                            let combined = combine_paths(&src_path, &tgt_path, 0.01);
                            if let Some(mut combined) = combined {
                                // Managed to combine the two brush strokes/paths into one: add to the group
                                let previous_elements   = combined_element.elements().cloned();
                                let grouped_elements    = group.elements().cloned().chain(previous_elements).collect();

                                let mut grouped         = GroupElement::new(ElementId::Unassigned, GroupType::Added, Arc::new(grouped_elements));

                                // In checking for an overlap we will have calculated most of the combined path: finish the job and set it as the hint
                                combined.set_exterior_by_adding();
                                combined.heal_exterior_gaps();

                                let combined_path = combined.exterior_paths();
                                let combined_path = vec![Path::from_paths(&combined_path)];
                                grouped.set_hint_path(Arc::new(combined_path));

                                CombineResult::NewElement(Vector::Group(grouped))
                            } else {
                                // Elements do not overlap
                                CombineResult::NoOverlap
                            }
                        } else {
                            // No source path
                            CombineResult::NoOverlap
                        }
                    }
                }

                // TODO: extend groups if they are of GroupType::Added and we overlap them
                Vector::Transformed(_) => {
                    CombineResult::UnableToCombineFurther
                },

                // Ignore
                _ => { CombineResult::NoOverlap }
            }
        } else {
            // When the properties change, we can't combine any further
            CombineResult::UnableToCombineFurther
        }
    }
}
