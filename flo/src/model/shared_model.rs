use flo_ui_files::*;

///
/// Represents the file model for FlowBetween animations
/// 
pub struct SharedModel {

}

impl<Anim: Animation+'static> FileModel for SharedModel {
    type SharedModel;
    type InstanceModel;
}