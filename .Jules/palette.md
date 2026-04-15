## 2024-04-15 - Context Menu Discoverability in egui
**Learning:** Custom context menus in desktop UI frameworks like egui (`.context_menu()`) lack the visual affordances that users might expect from explicit dropdown buttons or native browser context menus. Because they trigger on right-click without any visual hint, users often miss these secondary interactions.
**Action:** Always add hover tooltips (`.on_hover_text()`) to any interactive element that implements a context menu, explicitly suggesting "Right-click to..." to bridge the discoverability gap.
