## 2024-06-25 - Avoid redundant HashMap lookups in egui render loops
**Learning:** Calling `contains_key` followed by `get(...).unwrap()` on a HashMap in a hot path (like the egui UI render loop which executes every frame) wastes performance with redundant hash calculations and memory lookups. The codebase's `image_cache` was accessed this way, blocking the UI thread minimally but consistently.
**Action:** Always use the single-lookup pattern `if let Some(value) = hash_map.get(key) { ... }` (or `match`) instead of checking for containment first, especially in per-frame rendering operations.
