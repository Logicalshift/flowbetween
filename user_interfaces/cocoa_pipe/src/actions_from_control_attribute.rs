use super::action::*;
use super::actions_from::*;

use flo_ui::*;

impl ActionsFrom<ViewAction> for ControlAttribute {
    fn actions_from(&self) -> Vec<ViewAction> { 
        vec![]
    }
}
