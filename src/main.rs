use std::thread;
use std::time::Duration;

mod editor;

fn main() {
    let mut my_editor = editor::editor::Editor::new().unwrap();
    my_editor.run();
}
