use super::action::*;

use flo_ui::session::*;

use futures::*;

struct AppState {

}

///
/// Pipes UI updates to a Cocoa UI action sink
///
pub fn pipe_ui_updates<UiStream, CocoaSink>(ui_stream: UiStream, cocoa_sink: CocoaSink) -> impl Future<Item=()>
where   UiStream:   Stream<Item=Vec<UiUpdate>, Error=()>,
        CocoaSink:  Sink<SinkItem=Vec<AppAction>, SinkError=()> {
    // Create the state struction
    let mut state = AppState {

    };

    // Map the stream
    ui_stream
        .map(move |updates| {
            updates.into_iter()
                .flat_map(|update| state.map_update(update))
                .collect::<Vec<_>>()
        })
        .forward(cocoa_sink)
        .map(|_| ())
}

impl AppState {
    ///
    /// Changes a UI update into one or more AppActions
    ///
    fn map_update(&mut self, update: UiUpdate) -> Vec<AppAction> {
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