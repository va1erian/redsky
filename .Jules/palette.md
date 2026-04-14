## 2024-05-18 - Egui Tooltip Discoverability
**Learning:** In the `egui` framework, actions tied to context menus (like right-click options) lack visual affordances by default. Adding `.on_hover_text()` to the button element is an effective pattern to surface these hidden interactions without cluttering the UI.
**Action:** Whenever a `.context_menu()` is implemented in `egui`, immediately evaluate if a corresponding `.on_hover_text()` tooltip should be added to explain the interaction pattern.
