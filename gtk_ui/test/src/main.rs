extern crate gtk;
extern crate flo_gtk_ui;

use flo_gtk_ui::*;

use std::thread;
use std::time::Duration;

fn main() {
    let mut gtk_thread  = GtkThread::new();
    let window0         = WindowId::Assigned(0);

    gtk_thread.perform_actions(vec![
        GtkAction::Window(window0, GtkWindowAction::New(gtk::WindowType::Toplevel)),
        GtkAction::Window(window0, GtkWindowAction::SetTitle("Hello".to_string())),
        GtkAction::Window(window0, GtkWindowAction::SetDefaultSize(1024, 768)),
        GtkAction::Window(window0, GtkWindowAction::ShowAll)
    ]);

    println!("Hello");
    thread::sleep(Duration::from_millis(20000));
}
