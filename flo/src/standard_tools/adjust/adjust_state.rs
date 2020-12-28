use super::adjust_edge::*;
use super::adjust_control_point::*;

use crate::tools::*;
use crate::model::*;

use flo_curves::bezier::*;
use flo_binding::*;
use flo_animation::*;

use std::f32;
use std::f64;
use std::iter;
use std::sync::*;
use std::collections::{HashSet};

/// Proximity the pointer should be to a control point to interact with it
const MIN_DISTANCE: f64 = 4.0;

///
/// The current state of the input handler for the adjust tool
///
pub (super) struct AdjustToolState<Anim: 'static+EditableAnimation> {
    pub (super) input:                      ToolInputStream<()>, 
    pub (super) actions:                    ToolActionPublisher<()>,
    pub (super) flo_model:                  Arc<FloModel<Anim>>,
    pub (super) control_points:             BindRef<Arc<Vec<AdjustControlPoint>>>,
    pub (super) selected_control_points:    Binding<HashSet<AdjustControlPointId>>,
}

impl<Anim: 'static+EditableAnimation> AdjustToolState<Anim> {
    ///
    /// Finds the control point nearest to the specified position
    ///
    pub fn control_point_at_position(&self, x: f64, y: f64) -> Option<AdjustControlPointId> {
        let mut found_distance          = 1000.0;
        let mut found_control_point     = None;

        for cp in self.control_points.get().iter().rev() {
            if cp.control_point.is_control_point() { continue; }

            let (cp_x, cp_y)        = cp.control_point.position();
            let x_diff              = cp_x - x;
            let y_diff              = cp_y - y;
            let distance_squared    = (x_diff*x_diff) + (y_diff)*(y_diff);

            if distance_squared < found_distance && distance_squared < MIN_DISTANCE*MIN_DISTANCE {
                found_distance      = distance_squared;
                found_control_point = Some(AdjustControlPointId { owner: cp.owner, index: cp.index });
            }
        }

        found_control_point
    }

    ///
    /// The control points to drag at the specified position, if they're different to the selection
    ///
    pub fn drag_control_points(&self, x: f64, y: f64) -> Option<HashSet<AdjustControlPointId>> {
        const MAX_DISTANCE: f64         = MIN_DISTANCE * 2.0;
        let selected_control_points     = self.selected_control_points.get();

        if selected_control_points.len() == 1 {
            // If only one control point is selected, the user might drag the handles on either side
            let control_points  = self.control_points.get();
            let center_cp       = selected_control_points.iter().nth(0).cloned().unwrap();

            // Search for the center CP
            for cp_index in 0..control_points.len() {
                let cp = &control_points[cp_index];

                if cp.owner == center_cp.owner && cp.index == center_cp.index {
                    if cp.control_point.is_control_point() {
                        // Doesn't have handles
                        break;
                    }

                    // The left and right points might be the handles for this item
                    if cp_index > 0 && control_points[cp_index-1].control_point.is_control_point() {
                        // This CP is being dragged if it's within MAX_DISTANCE of the click
                        let (x2, y2) = control_points[cp_index-1].control_point.position();
                        let (dx, dy) = ((x-x2), (y-y2));

                        if (dx*dx) + (dy*dy) <= MAX_DISTANCE { return Some(iter::once((&control_points[cp_index-1]).into()).collect()); }
                    }

                    if cp_index < control_points.len()-1 && control_points[cp_index+1].control_point.is_control_point() {
                        // This CP is being dragged if it's within MAX_DISTANCE of the click
                        let (x2, y2) = control_points[cp_index+1].control_point.position();
                        let (dx, dy) = ((x-x2), (y-y2));

                        if (dx*dx) + (dy*dy) <= MAX_DISTANCE { return Some(iter::once((&control_points[cp_index+1]).into()).collect()); }
                    }

                    // Found the control point: don't look at the others
                    break;
                }
            }

            None
        } else {
            // Drag all of the selected control points, or select new control points if more than one is selected (or if 0 are selected)
            None
        }
    }

    ///
    /// Returns the bezier curve (and end point ID) corresponding to the specified start point
    ///
    pub fn curve_for_start_point(&self, control_points: &Vec<AdjustControlPoint>, control_point_index: usize) -> Option<(Curve<Coord2>, AdjustControlPointId)> {
        let cp_index        = control_point_index;

        // Get the initial control point for the curve
        let initial_point   = &control_points[cp_index];
        if initial_point.control_point.is_control_point() {
            // Must be a point on the curve and not a control point
            return None;
        }

        // Should be followed by two control points and an end point
        if cp_index+3 >= control_points.len() {
            return None;
        }

        let cp1         = &control_points[cp_index+1];
        let cp2         = &control_points[cp_index+2];
        let end_point   = &control_points[cp_index+3];

        // Check that these belong to the correct curve
        if !cp1.control_point.is_control_point() || !cp2.control_point.is_control_point() || end_point.control_point.is_control_point() {
            return None;
        }
        if cp1.owner != initial_point.owner || cp2.owner != initial_point.owner || end_point.owner != initial_point.owner {
            return None;
        }

        // Generate a curve from these control points
        let initial_point   = initial_point.control_point.position();
        let cp1             = cp1.control_point.position();
        let cp2             = cp2.control_point.position();
        let end_point_id    = end_point.into();
        let end_point       = end_point.control_point.position();

        let initial_point   = Coord2(initial_point.0, initial_point.1);
        let cp1             = Coord2(cp1.0, cp1.1);
        let cp2             = Coord2(cp2.0, cp2.1);
        let end_point       = Coord2(end_point.0, end_point.1);

        Some((Curve::from_points(initial_point, (cp1, cp2), end_point), end_point_id))
    }

    ///
    /// Returns the t value and distance to the closest point on the curve
    ///
    fn closest_t<C: BezierCurve<Point=Coord2>>(curve: &C, point: &Coord2) -> Option<(f64, f64)> {
        // Raycast to try to find the closet points (horizontally, vertically and at a 45 degree angle)
        let rays = vec![
                (Coord2(point.x(), point.y()), Coord2(point.x()-1.0, point.y())), 
                (Coord2(point.x(), point.y()), Coord2(point.x(), point.y()-1.0)), 
                (Coord2(point.x(), point.y()), Coord2(point.x()-1.0, point.y()-1.0)), 
            ];

        let mut closest = None;

        // Try each ray in turn
        for ray in rays {
            // Find the intersections of the curve with this ray
            let intersections = curve_intersects_ray(curve, &ray);

            // See if any of these intersection points are closer than the closest we've found so far
            for (curve_t, _line_t, intersection_point) in intersections.into_iter() {
                let distance = intersection_point.distance_to(point);

                if let Some((_closest_t, closest_distance)) = closest {
                    if distance < closest_distance {
                        closest = Some((curve_t, distance));
                    }
                } else {
                    closest = Some((curve_t, distance));
                }
            }
        }

        closest
    }

    ///
    /// Finds the curve (and the point) closest to the specified position
    ///
    pub fn curve_at_position(&self, x: f64, y: f64) -> Option<AdjustEdgePoint> {
        // Fetch the control points
        let control_points                          = self.control_points.get();
        let mut closest: Option<AdjustEdgePoint>   = None;

        // Check all of the curves generated by the control points to find the closest
        for cp_index in 0..control_points.len() {
            if let Some((curve, end_point_id)) = self.curve_for_start_point(&*control_points, cp_index) {
                // This control point represents a curve
                if let Some((closest_t, closest_distance)) = Self::closest_t(&curve, &Coord2(x, y)) {
                    // We've found a point close to the curve
                    if let Some(closest_point) = &closest {
                        if closest_point.distance > closest_distance {
                            // Found a new closest point
                            closest = Some(AdjustEdgePoint { start_point: (&control_points[cp_index]).into(), end_point: end_point_id, t: closest_t, distance: closest_distance });
                        }
                    } else {
                        // Found the first closest point
                        closest = Some(AdjustEdgePoint { start_point: (&control_points[cp_index]).into(), end_point: end_point_id, t: closest_t, distance: closest_distance });
                    }
                }
            }
        }

        if let Some(closest) = closest {
            if closest.distance <= MIN_DISTANCE*2.0 {
                Some(closest)
            } else {
                // Too far away
                None
            }
        } else {
            // No match
            None
        }
    }

    ///
    /// Returns the adjusted control points for the element the AdjustEdgePoint belongs to when transformed by the given offset
    ///
    pub fn adjusted_control_points_for_curve_drag(&self, adjust_curve: &AdjustEdgePoint, dx: f64, dy: f64) -> Vec<(f32, f32)> {
        // Fetch the element that's being adjusted
        let frame       = self.flo_model.frame().frame.get();
        let frame       = if let Some(frame) = frame { frame } else { return vec![] };
        let element     = frame.element_with_id(adjust_curve.start_point.owner);
        let element     = if let Some(element) = element { element } else { return vec![]; };

        let properties  = frame.apply_properties_for_element(&element, Arc::new(VectorProperties::default()));

        // Get the curve for this control point
        let control_points  = element.control_points(&properties);
        let control_points  = control_points.into_iter()
            .enumerate()
            .map(|(index, cp)| AdjustControlPoint { owner: element.id(), index: index, control_point: cp })
            .collect();

        let curve           = self.curve_for_start_point(&control_points, adjust_curve.start_point.index);
        let curve           = if let Some((curve, _)) = curve { curve } else { return vec![]; };

        // Transform the curve
        let curve           = move_point::<_, _, Curve<_>>(&curve, adjust_curve.t, &Coord2(dx, dy));

        // Work out the adjusted control points
        let (Coord2(cp1x, cp1y), Coord2(cp2x, cp2y)) = curve.control_points();
        let mut result      = vec![];

        for cp_index in 0..control_points.len() {
            if cp_index == adjust_curve.start_point.index+1 {
                // Should be cp1
                result.push((cp1x as f32, cp1y as f32));
            } else if cp_index == adjust_curve.start_point.index+2 {
                // Should be cp2
                result.push((cp2x as f32, cp2y as f32));
            } else {
                // Point is preserved
                let (x, y) = control_points[cp_index].control_point.position();
                result.push((x as f32, y as f32));
            }
        }

        result
    }

    ///
    /// Finds the element at the specified position
    ///
    pub fn element_at_position(&self, x: f64, y: f64) -> Option<ElementId> {
        // Find the elements at this point
        let frame       = self.flo_model.frame();
        let elements    = frame.elements_at_point((x as f32, y as f32));

        // Search for an element to select
        let mut selected_element = None;
        for elem in elements {
            match elem {
                ElementMatch::InsidePath(element) => {
                    selected_element = Some(element);
                    break;
                }

                ElementMatch::OnlyInBounds(element) => {
                    if selected_element.is_none() { selected_element = Some(element); }
                }
            }
        }

        selected_element
    }
}
