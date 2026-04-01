use super::widget::*;
use super::widget_data::*;
use super::super::gtk_action::*;

use flo_canvas::*;

use gtk;
use gtk::prelude::*;
use itertools::*;

use std::collections::HashMap;

///
/// The custom style manages custom widget style information
///
/// This is used so that we can things like font formatting and colour information for Flo Widgets to be things
/// other than their defaults (which Flo allows directly but GTK does via a style sheet... well, partly via a
/// style sheet)
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
        vec![ format!(".{} {{\n", self.class_name()) ].into_iter()
            .chain(self.styles.iter().map(|(name, defn)| format!(" {}: {};\n", name, defn)))
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
        style_context.add_provider(&self.style_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        style_context.add_class(&class_name);
    }

    ///
    /// Sets the value of a style property
    ///
    pub fn set_style(&mut self, key_name: &str, style: &str) {
        // Update the style and mark this as needing an update
        self.styles.insert(key_name.to_string(), style.to_string());
        self.need_refresh = true;
    }

    ///
    /// Returns the CSS for a color
    ///
    fn value_for_color(color: &Color) -> String {
        let (r, g, b, a)    = color.to_rgba_components();
        let (r, g, b)       = ((r*255.0).floor() as i32, (g*255.0).floor() as i32, (b*255.0).floor() as i32);

        format!("rgba({}, {}, {}, {})", r, g, b, a)
    }

    ///
    /// Updates the foreground colour of the style
    ///
    pub fn set_foreground(&mut self, foreground_color: &Color) {
        self.set_style("color", &Self::value_for_color(foreground_color));
    }

    ///
    /// Updates the background colour of the style
    ///
    pub fn set_background(&mut self, background_color: &Color) {
        self.set_style("background-color", &Self::value_for_color(background_color));
    }

    ///
    /// Sets the font size of this widget
    ///
    pub fn set_font_size(&mut self, pixels: f32) {
        self.set_style("font-size", &format!("{}px", pixels));
    }

    ///
    /// Sets the font weight of the widget
    ///
    pub fn set_font_weight(&mut self, weight: u32) {
        self.set_style("font-weight", &format!("{}", weight));
    }
}

///
/// Trait used to provide a custom style for a particular widget
///
pub trait CustomStyleForWidget {
    ///
    /// Retrieves the custom style for a widget
    ///
    fn get_custom_style(&self, widget: &dyn GtkUiWidget) -> WidgetDataEntry<CustomStyle>;

    ///
    /// Causes the custom style for a particular widget to be updated
    ///
    fn update_custom_style(&self, widget: &dyn GtkUiWidget);
}

impl CustomStyleForWidget for WidgetData {
    fn get_custom_style(&self, widget: &dyn GtkUiWidget) -> WidgetDataEntry<CustomStyle> {
        let widget_id = widget.id();

        if let Some(existing_style) = self.get_widget_data(widget_id) {
            // Just use whatever already exists if there's something
            existing_style
        } else {
            // This widget doesn't have a custom style yet

            // Create a new style and attach it to the widget
            let new_style = CustomStyle::new(widget_id);
            new_style.apply(widget.get_underlying());

            // Store as the style for this widget
            self.set_widget_data(widget_id, new_style);

            // Retrieve it straight back for the result of this function
            self.get_widget_data(widget_id).unwrap()
        }
    }

    fn update_custom_style(&self, widget: &dyn GtkUiWidget) {
        if let Some(existing_style) = self.get_widget_data::<CustomStyle>(widget.id()) {
            existing_style.borrow_mut().reload_if_needed();
        }
    }
}
