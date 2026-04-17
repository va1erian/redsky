import re

content = open("src/bsky_actor/actor_methods_mock.rs").read()

content = content.replace(
    """        Ok(RedskyUiMsg::RefreshBookmarksMsg { posts: vec![] })""",
    """        Ok(RedskyUiMsg::RefreshBookmarksMsg {
            posts: vec![],
            cursor: None,
            append: false,
        })"""
).replace(
    """        Ok(RedskyUiMsg::RefreshNotificationsMsg { notifications: vec![] })""",
    """        Ok(RedskyUiMsg::RefreshNotificationsMsg {
            notifications: vec![],
            cursor: None,
            append: false,
        })"""
)

open("src/bsky_actor/actor_methods_mock.rs", "w").write(content)

content = open("src/app/ui_post.rs").read()
content = content.replace(
    "ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {",
    "ui.allocate_ui_at_rect(rect, |ui| {"
)
# I'll just change back to allocate_ui_at_rect to let the warning pass if they allow deprecated there, wait CI uses `RUSTFLAGS: -D warnings` which means warnings are errors! I need to use `scope_builder`.
# Let's fix using scope_builder:
content = content.replace(
    "ui.allocate_ui_at_rect(rect, |ui| {",
    "ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {"
)
content = content.replace(
    "ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {",
    "ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {"
)
open("src/app/ui_post.rs", "w").write(content)
