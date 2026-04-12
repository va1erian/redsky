impl RedskyApp {
    fn make_settings_window(&mut self, ctx: &egui::Context) {
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("__settings"),
            egui::ViewportBuilder::default()
                .with_title("Settings")
                .with_inner_size([300.0, 200.0]),
            |ui, _| {
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading("Settings");
                        ui.separator();

                        let mut settings_changed = false;

                        ui.horizontal(|ui| {
                            ui.label("Theme:");
                            egui::ComboBox::from_id_salt("theme_combo")
                                .selected_text(format!("{:?}", self.settings.theme))
                                .show_ui(ui, |ui| {
                                    if ui.selectable_value(&mut self.settings.theme, AppTheme::System, "System").changed() {
                                        settings_changed = true;
                                    }
                                    if ui.selectable_value(&mut self.settings.theme, AppTheme::Light, "Light").changed() {
                                        settings_changed = true;
                                    }
                                    if ui.selectable_value(&mut self.settings.theme, AppTheme::Dark, "Dark").changed() {
                                        settings_changed = true;
                                    }
                                });
                        });

                        ui.horizontal(|ui| {
                            ui.label("Max Image Size in Timelines:");
                            if ui.add(egui::Slider::new(&mut self.settings.max_image_size, 100.0..=2560.0).text("px")).changed() {
                                settings_changed = true;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Global UI Zoom Factor:");
                            if ui.add(egui::Slider::new(&mut self.settings.zoom_factor, 0.5..=3.0).text("x")).changed() {
                                settings_changed = true;
                            }
                        });

                        if settings_changed {
                            self.settings.save();

                            // Apply theme immediately
                            match self.settings.theme {
                                AppTheme::System => {
                                    ctx.set_visuals(egui::Visuals::light()); // Fallback if system theme not supported by egui without extra logic
                                },
                                AppTheme::Light => {
                                    ctx.set_visuals(egui::Visuals::light());
                                },
                                AppTheme::Dark => {
                                    ctx.set_visuals(egui::Visuals::dark());
                                },
                            }
                        }
                    });
                });

                if ui.ctx().input(|i| i.viewport().close_requested()) {
                    self.is_settings_window_open = false;
                }
            },
        );
    }
}
