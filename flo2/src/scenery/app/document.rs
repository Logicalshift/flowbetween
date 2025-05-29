use flo_scene::*;

use serde::*;

///
/// The actions that can be performed on a document in the main app scene
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DocumentRequest {

}

impl SceneMessage for DocumentRequest {

}
