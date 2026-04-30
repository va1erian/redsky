## 2024-05-24 - Missing Affordances for Right-Click Context Menus
**Learning:** Context menus added via `context_menu()` in egui components have zero visual affordance by default. Users cannot easily discover that right-clicking elements (like interaction buttons) offers additional functionality (e.g. "Show Likers" / "Show Reposters").
**Action:** Always add a tooltip via `.on_hover_text()` to any element implementing a `.context_menu()` to provide clear instructions on how to access the hidden options.

## 2024-05-24 - Form Submission Keyboard Shortcuts
**Learning:** Keyboard-only users expect forms to be submittable via standard shortcuts (`Enter` for single-line forms like Login, `Cmd/Ctrl+Enter` for multi-line inputs like New Post). Without this, users must rely on mouse/tabbing to reach submit buttons.
**Action:** Always add keyboard event listeners `(ui.input(|i| i.key_pressed(egui::Key::Enter)))` or `(ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter)))` to forms, and add explicit tooltip hints for non-standard shortcuts.
## 2024-05-24 - Text Field Hinting
**Learning:** `egui::TextEdit::singleline` lacks context unless combined with labels or hint text. Users appreciate placeholder text indicating what kind of value is expected in text fields (e.g. format of a handle or kind of search).
**Action:** Instead of `ui.text_edit_singleline(...)`, use the builder pattern `ui.add(egui::TextEdit::singleline(...).hint_text("placeholder"))` to provide contextual hints for empty fields.
