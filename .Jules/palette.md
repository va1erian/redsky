## 2024-05-24 - Missing Affordances for Right-Click Context Menus
**Learning:** Context menus added via `context_menu()` in egui components have zero visual affordance by default. Users cannot easily discover that right-clicking elements (like interaction buttons) offers additional functionality (e.g. "Show Likers" / "Show Reposters").
**Action:** Always add a tooltip via `.on_hover_text()` to any element implementing a `.context_menu()` to provide clear instructions on how to access the hidden options.

## 2024-05-24 - Form Submission Keyboard Shortcuts
**Learning:** Keyboard-only users expect forms to be submittable via standard shortcuts (`Enter` for single-line forms like Login, `Cmd/Ctrl+Enter` for multi-line inputs like New Post). Without this, users must rely on mouse/tabbing to reach submit buttons.
**Action:** Always add keyboard event listeners `(ui.input(|i| i.key_pressed(egui::Key::Enter)))` or `(ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter)))` to forms, and add explicit tooltip hints for non-standard shortcuts.
## 2025-04-27 - Add hint text to form inputs
**Learning:** In egui, `ui.text_edit_singleline(&mut self.text)` lacks accessibility/guidance for empty inputs. Replacing it with the builder pattern `ui.add(egui::TextEdit::singleline(&mut self.text).hint_text("..."))` significantly improves discoverability without cluttering the UI. This is a highly reusable UX pattern for this app.
**Action:** Always prefer the `egui::TextEdit` builder pattern with `hint_text` over the shorthand `ui.text_edit_singleline` when dealing with search boxes, login forms, or other empty inputs that lack explicit inline context.
