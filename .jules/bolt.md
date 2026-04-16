## 2026-04-16 - Replaced Map Double-Lookups with Single Matches
**Learning:** Found an anti-pattern in the UI rendering loop where 'contains_key' followed by 'get().unwrap()' (a double hash lookup) caused excessive HashMap overhead.
**Action:** Replaced these cases across 'ui_post.rs', 'ui_widgets.rs', and 'ui_user.rs' with 'if let Some' or 'match' expressions. NLL ensures this is lifetime-safe even when a 'None' branch needs mutable borrowing afterwards.
