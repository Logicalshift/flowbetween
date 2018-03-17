use super::gtk_action::*;
use super::gtk_event::*;

use flo_ui::*;

pub trait GtkUserInterface : UserInterface<Vec<GtkAction>, Vec<GtkEvent>, ()> {
}
