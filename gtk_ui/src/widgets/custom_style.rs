use super::super::gtk_action::*;

use gtk;
use gtk::prelude::*;
use itertools::*;

use std::collections::HashMap;

///
/// The custom style manages custom widget style information
/// 
/// This is used so that we can things like font formatting and colour information for Flo Widgets to be things
/// other than their defaults (which Flo allows directly but GTK does extremely tediously via a style sheet)
/// 
pub struct CustomStyle {
    /// The widget ID that this custom style is for
    widget_id: WidgetId,

    /// A reference to the CSS provider where this style is set
    style_provider: gtk::CssProvider,

    /// The styles defined for this widget
    styles: HashMap<String, String>,

    /// True if the styles need to be reloaded into the style provider
    need_refresh: bool
}

impl CustomStyle {
    ///
    /// Creates a new custom style
    /// 
    pub fn new(widget_id: WidgetId) -> CustomStyle {
        CustomStyle {
            widget_id:      widget_id,
            style_provider: gtk::CssProvider::new(),
            styles:         HashMap::new(),
            need_refresh:   false
        }
    }

    ///
    /// Retrieves the class name this style will use within the style provider
    /// 
    fn class_name(&self) -> String {
        match self.widget_id {
            WidgetId::Unassigned    => "unassigned".to_string(),
            WidgetId::Assigned(id)  => format!("widget-{}", id)
        }
    }

    ///
    /// Returns the style sheet as a string
    ///
    fn style_sheet(&self) -> String {
        vec![ format!("{} {{\n", self.class_name()) ].into_iter()
            .chain(self.styles.iter().map(|(name, defn)| format!(" {}: {};", name, defn)))
            .chain(vec![ "}\n".to_string() ].into_iter())
            .join("")
    }

    ///
    /// Reloads the styles so that the attached widget will show the changes
    /// 
    pub fn reload_if_needed(&mut self) {
        if self.need_refresh {
            // Refresh the stylesheet
            let style_sheet = self.style_sheet();
            self.style_provider.load_from_data(style_sheet.as_bytes()).unwrap();

            // No longer need a refresh
            self.need_refresh = false;
        }
    }

    ///
    /// Applies this custom style to a widget
    /// 
    pub fn apply(&self, widget: &gtk::Widget) {
        let class_name      = self.class_name();
        let style_context   = widget.get_style_context();

        // We use a custom style provider and class name when we map our widget
        style_context.as_ref().map(|style_context| style_context.add_provider(&self.style_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION));
        style_context.map(|style_context| style_context.add_class(&class_name));
    }
}
