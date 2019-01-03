use super::action::*;

use flo_ui::session::*;

///
/// Represents the type
///
pub struct AppState {

}

impl AppState {
    ///
    /// Creates a new AppState
    ///
    pub fn new() -> AppState {
        AppState {
            
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
        vec![]
    }
}