use crate::scenery::document::*;

use flo_scene::*;
use futures::prelude::*;
use serde::*;

///
/// Commands for controlling the main flowbetween program
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FlowBetween {
    CreateEmptyDocument(DocumentId),
}

impl SceneMessage for FlowBetween { 
    fn default_target() -> StreamTarget {
        SubProgramId::called("flowbetween::flowbetween").into()
    }
}

///
/// Runs the main flowbetween program
///
pub async fn flowbetween(events: InputStream<FlowBetween>, _context: SceneContext) {
    let mut events = events;

    while let Some(evt) = events.next().await {
        use FlowBetween::*;

        match evt {
            CreateEmptyDocument(_document_id) => {

            }
        }
    }
}
