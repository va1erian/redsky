import re

content = open("src/app/ui_settings.rs").read()

content = content.replace(
    """                        if settings_changed {
                            self.settings.save();""",
    """                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut self.settings.allow_dehydration, "Enable post dehydration").changed() {
                                settings_changed = true;
                            }
                        });

                        if settings_changed {
                            self.settings.save();"""
)

open("src/app/ui_settings.rs", "w").write(content)
