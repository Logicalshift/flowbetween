use super::vector::*;
use super::element::*;
use super::group_type::*;
use super::properties::*;
use super::control_point::*;
use super::super::edit::*;
use super::super::path::*;
use super::super::motion::*;

use flo_canvas::*;
use flo_curves::bezier::path::*;

use std::sync::*;
use std::time::Duration;

///
/// Represents an element consisting of a group of other elements
///
#[derive(Clone, Debug)]
pub struct GroupElement {
    /// The ID assigned to this element
    id: ElementId,

    /// The type of this group
    group_type: GroupType,

    /// The elements that make up this group
    grouped_elements: Arc<Vec<Vector>>,
}

impl GroupElement {
    ///
    /// Creates a new group from a set of elements
    ///
    pub fn new(id: ElementId, group_type: GroupType, grouped_elements: Arc<Vec<Vector>>) -> GroupElement {
        GroupElement {
            id:                 id,
            group_type:         group_type,
            grouped_elements:   grouped_elements
        }
    }

    ///
    /// Renders the contents of this group in 'normal' mode
    ///
    fn render_normal(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties, when: Duration) {
        // Properties update internally to the group
        let mut properties = Arc::new(properties.clone());

        for elem in self.grouped_elements.iter() {
            properties = elem.update_properties(properties);
            properties.render(gc, elem.clone(), when);
        }
    }

    ///
    /// Returns the added path for this element
    ///
    fn added_path(&self, properties: &VectorProperties) -> Vec<Path> {
        // Get the paths for this rendering
        let paths = self.grouped_elements.iter()
            .flat_map(|elem| elem.to_path(properties))
            .map(|path| path_remove_interior_points::<_, Path>(&path, 0.01))
            .collect::<Vec<_>>();

        // Render if there are more than one path
        if paths.len() > 0 {
            // Add the paths into a single path
            let paths = path_add_chain(&paths, 0.01);
            paths
        } else {
            vec![]
        }
    }

    ///
    /// Renders the contents of this group in 'added' mode
    ///
    fn render_added(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties) {
        let paths = self.added_path(properties);

        gc.draw_list(properties.brush.prepare_to_render(&properties.brush_properties));
        paths.into_iter()
            .for_each(|path| gc.draw_list(properties.brush.render_path(&properties.brush_properties, &path)));
    }
}

impl VectorElement for GroupElement {
    ///
    /// The ID of this element
    /// 
    fn id(&self) -> ElementId {
        self.id
    }

    ///
    /// Retrieves the paths for this element, if there are any
    /// 
    fn to_path(&self, properties: &VectorProperties) -> Option<Vec<Path>> {
        match self.group_type {
            GroupType::Normal   => Some(self.grouped_elements.iter().flat_map(|elem| elem.to_path(properties)).flatten().collect()),
            GroupType::Added    => Some(self.added_path(properties))
        }
    }

    ///
    /// Renders this vector element
    /// 
    fn render(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties, when: Duration) {
        match self.group_type {
            GroupType::Normal   => self.render_normal(gc, properties, when),
            GroupType::Added    => self.render_added(gc, properties)
        }
    }

    ///
    /// Returns the properties to use for future elements
    /// 
    fn update_properties(&self, properties: Arc<VectorProperties>) -> Arc<VectorProperties> { 
        // Groups do not update properties
        properties
    }

    ///
    /// Returns a new element that is this element transformed along a motion at a particular moment
    /// in time.
    /// 
    fn motion_transform(&self, motion: &Motion, when: Duration) -> Vector {
        let new_elements = self.grouped_elements.iter()
            .map(|old_elem| old_elem.motion_transform(motion, when))
            .collect();

        Vector::Group(GroupElement::new(self.id, self.group_type, Arc::new(new_elements)))
    }

    ///
    /// Fetches the control points for this element
    /// 
    fn control_points(&self) -> Vec<ControlPoint> {
        vec![]
    }

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    /// 
    /// The vector here specifies the updated position for each control point in control_points
    /// 
    fn with_adjusted_control_points(&self, new_positions: Vec<(f32, f32)>) -> Vector {
        Vector::Group(self.clone())
    }
}
