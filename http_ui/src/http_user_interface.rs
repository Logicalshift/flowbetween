use super::event::*;
use super::update::*;
use super::htmlcontrol::*;

use ui::*;
use ui::session::*;
use binding::*;
use futures::*;
use futures::stream;

use std::sync::*;

///
/// Converts a core user interface into a HTTP user interface
/// 
pub struct HttpUserInterface<CoreUi> {
    /// The core UI is the non-platform specific implementation of the user interface
    core_ui: Arc<CoreUi>,

    /// A binding ref for the UI tree (we need this for converting controller paths)
    ui_tree: BindRef<Control>,

    /// The base path of the instance (where URIs are generated relative to)
    base_path: String
}

impl<CoreUi: CoreUserInterface> HttpUserInterface<CoreUi> {
    ///
    /// Creates a new HTTP UI that will translate requests for the specified core UI
    /// 
    pub fn new(ui: Arc<CoreUi>, base_path: String) -> HttpUserInterface<CoreUi> {
        let ui_tree = ui.ui_tree();

        HttpUserInterface {
            core_ui:    ui,
            ui_tree:    ui_tree,
            base_path:  base_path
        }
    }

    ///
    /// Retrieves the underlying non-platform specific UI object
    /// 
    pub fn core(&self) -> Arc<CoreUi> {
        Arc::clone(&self.core_ui)
    }

    ///
    /// Converts an event from the HTTP side of things into a UI event
    /// 
    fn http_event_to_core_event(http_event: Event) -> UiEvent {
        use Event::*;

        match http_event {
            NewSession  => UiEvent::Tick,
            UiRefresh   => UiEvent::Tick,
            Tick        => UiEvent::Tick,

            Action(controller_path, action_name, action_parameter) => UiEvent::Action(controller_path, action_name, action_parameter)
        }
    }

    ///
    /// Generates a new UI update (transforms a set of updates into the 'new HTML' update)
    /// 
    fn new_ui_update(old_updates: Vec<Update>) -> Vec<Update> {
        let mut new_updates = vec![];

        // Convert the updates in the old update
        for update in old_updates {
            match update {
                Update::UpdateHtml(html_diff) => {
                    ()
                },

                // Viewmodel updates should all wind up rolled into the new HTML update if we generate it
                // (Nothing will get generated normally)
                Update::UpdateViewModel(_) => (),

                // Everything else is left as-is
                update => new_updates.push(update)
            }
        }

        new_updates
    }

    ///
    /// Maps a core UI diff into a HTML diff
    /// 
    fn map_core_ui_diff(ui_diff: UiDiff, ui_tree: &Control, base_path: &str) -> HtmlDiff {
        // Fetch the properties of this difference
        let address         = ui_diff.address;
        let new_ui          = ui_diff.new_ui;
        let controller_path = html_controller_path_for_address(ui_tree, &address);
        let html            = new_ui.to_html_subcomponent(base_path, &controller_path);

        // Turn into a HTML diff
        HtmlDiff::new(address, &new_ui, html.to_string())
    }

    ///
    /// Maps a single core update to a HTTP update
    /// 
    fn map_core_update(core_update: UiUpdate, base_path: &str, ui_tree: &Control) -> Vec<Update> {
        use self::UiUpdate::*;

        match core_update {
            Start => vec![],

            UpdateUi(core_diffs) => {
                // Map the UI differences
                vec![Update::UpdateHtml(core_diffs.into_iter()
                    .map(|core_ui_diff| Self::map_core_ui_diff(core_ui_diff, &ui_tree, &base_path))
                    .collect()
                )]
            },
            
            UpdateCanvas(canvas_diffs) => unimplemented!() /* vec![Update::UpdateCanvas(canvas_diffs)] */,

            UpdateViewModel(view_model_diffs) => vec![Update::UpdateViewModel(view_model_diffs)]
        }
    }

    ///
    /// Converts updates from the core into HTTP updates
    /// 
    fn core_updates_to_http_updates(core_update: Vec<UiUpdate>, base_path: &str, ui_tree: &Control) -> Vec<Update> {
        use self::UiUpdate::*;

        let is_start    = core_update.len() > 0 && core_update[0] == Start;
        let base_update = core_update.into_iter()
                .flat_map(|core_update| Self::map_core_update(core_update, base_path, ui_tree).into_iter())
                .collect();

        if is_start {
            // Generate the new UI HTML update
            Self::new_ui_update(base_update)            
        } else {
            // Convert each update individually
            base_update
        }
    }
}

impl<CoreUi: CoreUserInterface> UserInterface<Event, Vec<Update>, ()> for HttpUserInterface<CoreUi> {
    type EventSink = Box<Sink<SinkItem=Event, SinkError=()>>;
    type UpdateStream = Box<Stream<Item=Vec<Update>, Error=()>>;

    fn get_input_sink(&self) -> Self::EventSink {
        // Get the core event sink
        let core_sink   = self.core_ui.get_input_sink();

        // Create a sink that turns HTTP events into core events
        let mapped_sink = core_sink.with_flat_map(|http_event| {
            let core_event = Self::http_event_to_core_event(http_event);
            stream::once(Ok(core_event))
        });

        // This new sink is our result
        Box::new(mapped_sink)
    }

    fn get_updates(&self) -> Self::UpdateStream {
        // Fetch the updates from the core
        let core_updates = self.core_ui.get_updates();

        // Fetch the extra components we need to map events from this object
        let ui_tree     = BindRef::clone(&self.ui_tree);
        let base_path   = self.base_path.clone();

        // Turn into HTTP updates
        let mapped_updates = core_updates.map(move |core_updates| {
            let ui_tree = ui_tree.get();

            Self::core_updates_to_http_updates(core_updates, &base_path, &ui_tree)
        });

        // These are the results
        Box::new(mapped_updates)
    }
}
