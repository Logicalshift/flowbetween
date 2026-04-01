use super::viewmodel::*;
use super::super::gtk_action::*;

///
/// Represents a control within the UI hierarchy
///
pub struct GtkControl {
    /// The ID of the widget that is displaying the UI for this control
    pub widget_id: WidgetId,

    /// The name of the controller that this control and its children belong to
    pub controller: Option<String>,

    /// Controls stored underneath this one in the UI tree
    pub child_controls: Vec<GtkControl>
}

impl GtkControl {
    ///
    /// Creates a new GTK control with a particular widget ID
    ///
    pub fn new(widget_id: WidgetId, controller: Option<String>) -> GtkControl {
        GtkControl {
            widget_id:      widget_id,
            controller:     controller,
            child_controls: vec![]
        }
    }

    ///
    /// If this control has a child at the specified index, this will return a reference to it
    ///
    pub fn child_at_index<'a>(&'a self, index: u32) -> Option<&'a GtkControl> {
        self.child_controls.get(index as usize)
    }

    ///
    /// If this control has a child at the specified index, this will return a reference to it
    ///
    pub fn child_at_index_mut<'a>(&'a mut self, index: u32) -> Option<&'a mut GtkControl> {
        self.child_controls.get_mut(index as usize)
    }

    ///
    /// Removes this control and any child controls from the viewmodel
    ///
    pub fn delete_from_viewmodel(&self, viewmodel: &mut GtkSessionViewModel) {
        for child_control in self.child_controls.iter() {
            child_control.delete_from_viewmodel(viewmodel);
        }

        viewmodel.delete_widget(self.widget_id);
    }

    ///
    /// Returns all of the widget IDs in the tree rooted at this control
    ///
    pub fn tree_ids(&self) -> Vec<WidgetId> {
        vec![ self.widget_id ]
            .into_iter()
            .chain(self.child_controls.iter()
                .flat_map(|control| control.tree_ids())
            )
            .collect()
    }

    ///
    /// Creates the actions for deleting this control and any child controls
    ///
    pub fn delete_actions(&self) -> Vec<GtkAction> {
        let mut actions = vec![];

        // Tell all the child controls to generate their delete actions
        for child_control in self.child_controls.iter() {
            actions.extend(child_control.delete_actions());
        }

        // Generate our delete action
        actions.push(GtkAction::Widget(self.widget_id, vec![GtkWidgetAction::Delete]));

        // Control is gone
        actions
    }
}
