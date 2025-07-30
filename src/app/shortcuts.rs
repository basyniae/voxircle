use egui::InputState;
use egui::Key;
use egui::KeyboardShortcut;
use egui::Modifiers;

pub fn shortcuts(i: &mut InputState) {
    // Quit (does not work in web)
    #[cfg(not(target_arch = "wasm32"))]
    {
        if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::NONE, Key::Q))
            || i.consume_shortcut(&KeyboardShortcut::new(Modifiers::NONE, Key::Escape))
        {
            std::process::exit(0);
        }
    }

    // have to add alternate for web as otherwise my ide forgets the imports are used (stupidly)
    #[cfg(target_arch = "wasm32")]
    {
        if false {
            if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::NONE, Key::Q))
                || i.consume_shortcut(&KeyboardShortcut::new(Modifiers::NONE, Key::Escape))
            {
                std::process::exit(0);
            }
        }
    }
}
