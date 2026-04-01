extern crate gtk;
extern crate flo_ui;
extern crate flo_gtk_ui;

use flo_ui::*;
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

        GtkAction::Widget(WidgetId::Assigned(1), vec![
            GtkWidgetAction::New(GtkWidgetType::Generic),
            GtkWidgetAction::Content(WidgetContent::SetText(String::from("Something else"))),
            GtkWidgetAction::Layout(WidgetLayout::BoundingBox(Bounds {
                x1: Position::At(100.0),
                y1: Position::At(100.0),
                x2: Position::At(400.0),
                y2: Position::At(120.0)
            }))
        ]),

        GtkAction::Widget(WidgetId::Assigned(2), vec![
            GtkWidgetAction::New(GtkWidgetType::CanvasRender),
            GtkWidgetAction::Layout(WidgetLayout::BoundingBox(Bounds {
                x1: Position::Start,
                y1: Position::Start,
                x2: Position::End,
                y2: Position::End
            }))
        ]),

        GtkAction::Widget(WidgetId::Assigned(0), vec![
            GtkWidgetAction::New(GtkWidgetType::Generic),
            GtkWidgetAction::Content(WidgetContent::SetText(String::from("Hello, world"))),
            GtkWidgetAction::Content(WidgetContent::SetChildren(vec![WidgetId::Assigned(1), WidgetId::Assigned(2)])),
            GtkWidgetAction::SetRoot(window0)
        ])
    ]);

    println!("Hello");
    thread::sleep(Duration::from_millis(20000));
}
