## 2024-05-18 - [Performance] Defer pretty-printing of raw JSON to UI on demand
**Learning:** In an immediate mode GUI framework like egui, putting expensive operations like `serde_json::from_str` or `to_string_pretty` inside the render loop causes catastrophic performance issues because it runs every frame. The initial attempt to fix a performance issue by offloading work to the UI layer resulted in a major regression.
**Action:** When shifting work from the background data processing layer (e.g., parsing feed posts) to the UI layer, always ensure the expensive work is done exactly once in the event/message handler and the result is cached in the application state. Never do the parsing/formatting directly inside the egui `show` closure.

## 2025-03-08 - Prevent egui unbounded repaints from off-screen animations
**Learning:** In egui, rendering animated elements (like `ui.spinner()`) continuously triggers immediate-mode repaints. If these elements are placed in long, scrollable lists (like infinite scrolling feeds) but are not currently visible (off-screen placeholders), they still trigger UI repaints at 60 FPS, resulting in massive unbounded CPU overhead for dehydrated items.
**Action:** Always check visibility (`ui.is_rect_visible(rect)`) on the allocated rect for placeholders before rendering any animated element (such as a spinner) in long lists.
