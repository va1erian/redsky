## 2024-05-15 - Improve Context Menu Discoverability
**Learning:** Context menus tied to primary interaction buttons (like Like/Repost) are often completely undiscoverable to users without explicit visual hints, as there is no standard affordance for "right-click here for more options".
**Action:** Add tooltip text via `.on_hover_text()` to any button that implements `.context_menu()` to ensure users are aware of the secondary actions available.
