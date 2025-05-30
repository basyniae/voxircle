use egui::{InputState, Key, KeyboardShortcut, Modifiers};

pub fn shortcuts(i: &mut InputState) {
    if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::NONE, Key::Q))
        || i.consume_shortcut(&KeyboardShortcut::new(Modifiers::NONE, Key::Escape))
    {
        std::process::exit(0);
    }
}
