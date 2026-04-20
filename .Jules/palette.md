## 2024-05-24 - Missing Affordances for Right-Click Context Menus
**Learning:** Context menus added via `context_menu()` in egui components have zero visual affordance by default. Users cannot easily discover that right-clicking elements (like interaction buttons) offers additional functionality (e.g. "Show Likers" / "Show Reposters").
**Action:** Always add a tooltip via `.on_hover_text()` to any element implementing a `.context_menu()` to provide clear instructions on how to access the hidden options.

## 2024-05-24 - Form Submission Keyboard Shortcuts
**Learning:** Keyboard-only users expect forms to be submittable via standard shortcuts (`Enter` for single-line forms like Login, `Cmd/Ctrl+Enter` for multi-line inputs like New Post). Without this, users must rely on mouse/tabbing to reach submit buttons.
**Action:** Always add keyboard event listeners `(ui.input(|i| i.key_pressed(egui::Key::Enter)))` or `(ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter)))` to forms, and add explicit tooltip hints for non-standard shortcuts.

## 2024-05-25 - Form Fields Placeholder UX
**Learning:** `egui::TextEdit` inputs lack placeholder text natively. Users won't inherently know expected input formats for forms (like bsky handles or passwords).
**Action:** Always add `.hint_text("...")` to `egui::TextEdit` components to provide clear placeholder text and improve discoverability of required field formats.

## 2024-05-25 - Empty States UX
**Learning:** Feeds or collections of items (like Notifications or Timelines) can appear broken or stuck if they render empty without explanation.
**Action:** Always check if a collection is empty (`if collection.is_empty() { ... }`) and render an explicit empty state message (e.g., `ui.label("No notifications yet.");`) to reassure the user.
