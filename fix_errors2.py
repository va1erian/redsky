import re

content = open("src/app/ui_post.rs").read()

content = content.replace(
    """                            let visible_idx = (scroll_output.state.offset.y / 200.0) as i32;
                            if (idx as i32 - visible_idx).abs() <= 50 {""",
    """                            let visible = ui.is_rect_visible(ui.cursor());
                            if visible {"""
)
content = content.replace("ui.allocate_ui_at_rect(rect,", "ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect),")

open("src/app/ui_post.rs", "w").write(content)

content = open("src/app/ui_user.rs").read()
content = content.replace(
    "let scroll_offset_y = scroll_output.state.offset.y;",
    "let _scroll_offset_y = scroll_output.state.offset.y;"
).replace(
    "let content_size_y = scroll_output.content_size.y;",
    "let _content_size_y = scroll_output.content_size.y;"
)
open("src/app/ui_user.rs", "w").write(content)
