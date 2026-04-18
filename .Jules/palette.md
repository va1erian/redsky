## 2024-05-24 - Missing Affordances for Right-Click Context Menus
**Learning:** Context menus added via `context_menu()` in egui components have zero visual affordance by default. Users cannot easily discover that right-clicking elements (like interaction buttons) offers additional functionality (e.g. "Show Likers" / "Show Reposters").
**Action:** Always add a tooltip via `.on_hover_text()` to any element implementing a `.context_menu()` to provide clear instructions on how to access the hidden options.

## 2024-05-24 - Form Submission Keyboard Shortcuts
**Learning:** Keyboard-only users expect forms to be submittable via standard shortcuts (`Enter` for single-line forms like Login, `Cmd/Ctrl+Enter` for multi-line inputs like New Post). Without this, users must rely on mouse/tabbing to reach submit buttons.
**Action:** Always add keyboard event listeners `(ui.input(|i| i.key_pressed(egui::Key::Enter)))` or `(ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter)))` to forms, and add explicit tooltip hints for non-standard shortcuts.
## 2024-05-18 - Hint text for egui TextEdit components
**Learning:** In immediate-mode GUI frameworks like egui, text inputs lack default placeholder or hint text, which can lead to poor UX discoverability. Since there are no native placeholder attributes in rust egui by default, using `egui::TextEdit::hint_text("...")` provides the needed context for users to understand what input is expected, greatly improving usability without cluttering the UI with additional labels.
**Action:** When adding or encountering `egui::TextEdit` components (both single and multiline) that don't have clear accompanying context, always append `.hint_text("...")` to provide immediate inline guidance.
