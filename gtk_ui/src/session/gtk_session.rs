use super::gtk_user_interface::*;
use super::super::gtk_action::*;

use flo_ui::*;
use flo_ui::session::*;

use gtk;
use gtk::prelude::*;
use futures::*;
use std::sync::*;

///
/// Core data structures associated with a Gtk session
/// 
struct GtkSessionCore {
    /// The GTK user interface
    gtk_ui: GtkUserInterface
}

///
/// The Gtk session object represents a session running with Gtk
/// 
pub struct GtkSession {
    core: Arc<Mutex<GtkSessionCore>>
}

impl GtkSession {
    ///
    /// Creates a new session connecting a core UI to a Gtk UI
    /// 
    pub fn new<Ui: CoreUserInterface>(core_ui: Ui, gtk_ui: GtkUserInterface) -> GtkSession {
        // Get the GTK event streams
        let mut gtk_action_sink     = gtk_ui.get_input_sink();
        let mut gtk_event_stream    = gtk_ui.get_updates();

        // Create the main window (always ID 0)
        Self::create_main_window(&mut gtk_action_sink);

        // Create the core
        let core = GtkSessionCore {
            gtk_ui: gtk_ui
        };
        let core = Arc::new(Mutex::new(core));

        // Finish up by creating the new session
        GtkSession {
            core: core
        }
    }

    ///
    /// Creates a GTK session from a core controller
    /// 
    pub fn from<CoreController: Controller+'static>(controller: CoreController, gtk_ui: GtkUserInterface) -> GtkSession {
        let session = UiSession::new(controller);
        Self::new(session, gtk_ui)
    }

    ///
    /// Creates the main window (ID 0) to run our session in
    /// 
    fn create_main_window<S: Sink<SinkItem=Vec<GtkAction>, SinkError=()>>(action_sink: &mut S) {
        use self::GtkAction::*;
        use self::GtkWindowAction::*;    

        // Create window 0, which will be the main window where the UI will run
        action_sink.start_send(vec![
            Window(WindowId::Assigned(0), vec![
                New(gtk::WindowType::Toplevel),
                SetPosition(gtk::WindowPosition::Center),
                SetDefaultSize(1920, 1080),             // TODO: make configurable (?)
                SetTitle("FlowBetween".to_string()),    // TODO: make configurable
                ShowAll
            ])
        ]).unwrap();
    }
}
