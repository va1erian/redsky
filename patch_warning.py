import re

content = open("src/app/ui_post.rs").read()
content = content.replace("ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect),", "ui.allocate_ui_at_rect(rect,")
# I should change allocate_new_ui back to allocate_ui_at_rect and just let the warning be, OR use scope_builder
# Let's check egui docs for scope_builder
content = content.replace(
    "ui.allocate_ui_at_rect(rect, |ui| {",
    "ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {"
)
open("src/app/ui_post.rs", "w").write(content)
