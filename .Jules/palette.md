## 2024-05-24 - Missing Affordances for Right-Click Context Menus
**Learning:** Context menus added via `context_menu()` in egui components have zero visual affordance by default. Users cannot easily discover that right-clicking elements (like interaction buttons) offers additional functionality (e.g. "Show Likers" / "Show Reposters").
**Action:** Always add a tooltip via `.on_hover_text()` to any element implementing a `.context_menu()` to provide clear instructions on how to access the hidden options.

## 2024-05-24 - Form Submission Keyboard Shortcuts
**Learning:** Keyboard-only users expect forms to be submittable via standard shortcuts (`Enter` for single-line forms like Login, `Cmd/Ctrl+Enter` for multi-line inputs like New Post). Without this, users must rely on mouse/tabbing to reach submit buttons.
**Action:** Always add keyboard event listeners `(ui.input(|i| i.key_pressed(egui::Key::Enter)))` or `(ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter)))` to forms, and add explicit tooltip hints for non-standard shortcuts.

## 2024-05-24 - Add placeholder texts to form fields
**Learning:** Replaced  with  to natively support placeholder texts in egui.
**Action:** Use hint_text via ui.add builder pattern in the future to improve form discoverability for standard egui inputs.

## 2024-05-24 - Add placeholder texts to form fields
**Learning:** In egui, placeholder texts are not natively supported through convenience methods like `ui.text_edit_singleline`. We must use the builder pattern `ui.add(egui::TextEdit::singleline(...).hint_text(...))` to implement standard input placeholders.
**Action:** Always use the `ui.add` builder pattern with `.hint_text()` when defining inputs to ensure users have clear, contextually relevant placeholder guidance.
