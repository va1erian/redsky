## 2024-04-13 - [HashMap Lookup Optimization]
**Learning:** Found an anti-pattern of using `contains_key` followed by `get().unwrap()` on the `image_cache` HashMap in the UI render loops (e.g., `make_post_inner_view`, `make_buffer_image_view`). This causes a double lookup on the map for every frame that renders an image.
**Action:** Always use pattern matching like `if let Some(texture) = self.image_cache.get(key)` or `match self.image_cache.get(key)` to perform a single lookup and avoid the extra hashing and `.unwrap()` overhead.
