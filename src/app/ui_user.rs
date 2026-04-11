impl RedskyApp {

    fn make_maybe_user_profile_view(
        &self,
        ui: &mut Ui,
        username: &str,
        maybe_profile: Option<&UserProfile>,
    ) {
        match maybe_profile {
            Some(profile) => {
                ui.horizontal(|ui| {
                    ui.set_max_height(120f32);
                    match self.image_cache.get(&profile.avatar_uri) {
                        Some(Some(texture)) => {
                            ui.vertical(|ui| {
                                ui.set_max_width(120f32);
                                ui.set_max_height(120f32);
                                ui.add(
                                    egui::Image::new(texture).max_width(120.0).max_height(120.0),
                                );
                            });
                        }
                        Some(None) => {
                            ui.vertical(|ui| {
                                ui.set_max_width(120f32);
                                ui.set_max_height(120f32);
                                ui.spinner();
                            });
                        }
                        None => {
                            self.post_message(BskyActorMsg::LoadImage {
                                url: profile.avatar_uri.clone(),
                            });
                        }
                    }
                    ui.vertical(|ui| {
                        ui.set_max_height(120f32);
                        ui.heading(&profile.display_name);
                        ui.small(&profile.handle);
                        ui.label(&profile.bio);
                        ui.label(format!(
                            "{} post(s), {} follower(s), {} follow(s)",
                            &profile.post_count, &profile.follower_count, &profile.follow_count
                        ));
                    });
                    ui.allocate_space(ui.available_size());
                });
            }
            None => {
                ui.label(username);
                ui.spinner();
            }
        }
    }

                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          

    fn make_maybe_user_post_view(
        &mut self,
        ui: &mut Ui,
        username: &str,
        posts: &mut Option<Vec<FeedItem>>,
    ) {
        StripBuilder::new(ui)
            .size(Size::exact(120.0))
            .size(Size::remainder())
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    self.make_maybe_user_profile_view(
                        ui,
                        username,
                        self.user_infos_cache.get(username),
                    );
                    ui.separator();
                });
                strip.strip(|builder| {
                    builder.sizes(Size::remainder(), 1).horizontal(|mut strip| {
                        strip.cell(|ui| match posts {
                            Some(posts) => {
                                self.make_post_view(ui, username, posts);
                            }
                            None => {
                                self.make_placeholder_post_view(ui, username);
                            }
                        });
                    });
                });
            });
    }

                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               

    fn make_user_timelines_views(&mut self, ctx: &egui::Context) {
        let mut to_drop = Vec::new();
        let mut to_download = Vec::new();

        let usernames: Vec<String> = self.user_posts.keys().cloned().collect();
        for username in usernames {
            if username == self.login {
                continue;
            }

            let mut posts = self.user_posts.get_mut(&username).unwrap().take();

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of(&username),
                egui::ViewportBuilder::default()
                    .with_title(format!("Posts of {}", username.clone()))
                    .with_inner_size([400.0, 600.0]),
                |ui, _| {
                    egui::Panel::top("menu_bar").show_inside(ui, |ui| {
                        egui::MenuBar::new().ui(ui, |ui| {
                            ui.menu_button("Actions", |ui| {
                                if ui.button("Download All Images").clicked() {
                                    to_download.push(username.clone());
                                    ui.close();
                                }
                            });
                        });
                    });
                    egui::CentralPanel::default().show_inside(ui, |ui| {
                        self.make_maybe_user_post_view(ui, &username, &mut posts);
                    });

                    if ui.ctx().input(|i| i.viewport().close_requested()) {
                        to_drop.push(username.clone());
                    }
                },
            );
            self.user_posts.insert(username, posts);
        }

        for username in to_drop {
            self.ui_tx
                .send(RedskyUiMsg::DropUserPostsMsg { username })
                .unwrap();
        }

        for username in to_download {
            let ui_tx = self.ui_tx.clone();
            let ctx = ctx.clone();
            std::thread::spawn(move || {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    let path_str = path.display().to_string();
                    let _ = ui_tx.send(RedskyUiMsg::StartDownloadJob {
                        username,
                        path: path_str,
                    });
                    ctx.request_repaint();
                }
            });
        }
    }
}
