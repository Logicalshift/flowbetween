extern crate gtk;
extern crate flo_gtk_ui;

use flo_gtk_ui::*;

use std::thread;
use std::time::Duration;

fn main() {
    let mut gtk_thread  = GtkThread::new();
    let window0         = WindowId::Assigned(0);

    gtk_thread.perform_actions(vec![
        GtkAction::Window(window0, vec![
            GtkWindowAction::New(gtk::WindowType::Toplevel),
            GtkWindowAction::SetTitle("Hello".to_string()),
            GtkWindowAction::SetDefaultSize(1024, 768),
            GtkWindowAction::ShowAll
        ]),

        GtkAction::Widget(WidgetId::Assigned(0), vec![
            GtkWidgetAction::New(GtkWidgetType::Generic),
            GtkWidgetAction::Content(WidgetContent::SetText(String::from("Hello, world"))),
            GtkWidgetAction::SetRoot(window0)
        ])
    ]);

    println!("Hello");
    thread::sleep(Duration::from_millis(20000));
}
