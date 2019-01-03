use super::action::*;
use super::view_state::*;

use flo_ui::*;
use flo_ui::session::*;

///
/// Represents the type
///
pub struct AppState {
    /// The root view
    root_view: Option<ViewState>,

    /// The ID that will be assigned to the next view we create
    next_view_id: usize
}

impl AppState {
    ///
    /// Creates a new AppState
    ///
    pub fn new() -> AppState {
        AppState {
            root_view:      None,
            next_view_id:   0
        }
    }

    ///
    /// Changes a UI update into one or more AppActions
    ///
    pub fn map_update(&mut self, update: UiUpdate) -> Vec<AppAction> {
        match update {
            UiUpdate::Start                     => { self.start() }
            UiUpdate::UpdateUi(differences)     => { self.update_ui(differences) }
            UiUpdate::UpdateCanvas(differences) => { vec![] }
            UiUpdate::UpdateViewModel(updates)  => { vec![] }
        }
    }

    ///
    /// Processes the 'start' update
    ///
    fn start(&mut self) -> Vec<AppAction> {
        vec![
            AppAction::CreateWindow(0)
        ]
    }

    ///
    /// Maps a UiDiff into the AppActions required to carry it out
    ///
    fn update_ui(&mut self, differences: Vec<UiDiff>) -> Vec<AppAction> {
        differences.into_iter()
            .flat_map(|diff| self.update_ui_from_diff(diff))
            .collect()
    }

    ///
    /// Returns the actions required to perform a single UI diff
    ///
    fn update_ui_from_diff(&mut self, difference: UiDiff) -> Vec<AppAction> {
        // The difference specifies a view to replace
        let view_to_replace = self.root_view.as_ref().and_then(|root_view| root_view.get_state_at_address(&difference.address));

        // Create the replacement view states
        let (view_state, actions) = self.create_view(&difference.new_ui);

        actions
    }

    ///
    /// Creates a view (and subviews) from a UI control
    ///
    fn create_view(&mut self, control: &Control) -> (ViewState, Vec<AppAction>) {
        // Create a new view state
        let view_id             = self.next_view_id;
        self.next_view_id       += 1;
        let mut view_state      = ViewState::new(view_id);

        // Initialise from the control
        let mut setup_actions   = view_state.set_up_from_control(control);

        // Also set up any subcomponents
        for subcomponent in control.subcomponents().unwrap_or(&vec![]) {
            // Create the view for the subcomponent
            let (subcomponent_view, subcomponent_actions) = self.create_view(subcomponent);

            // Add to the setup actions
            setup_actions.extend(subcomponent_actions);

            // Add as a subview
            setup_actions.push(AppAction::View(view_id, ViewAction::AddSubView(subcomponent_view.id())));

            // Add as a child control of our view state
            view_state.add_child_state(subcomponent_view);
        }

        (view_state, setup_actions)
    }
}
