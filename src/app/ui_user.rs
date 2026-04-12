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
            .size(Size::exact(150.0))
            .size(Size::remainder())
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    self.make_maybe_user_profile_view(
                        ui,
                        username,
                        self.user_infos_cache.get(username),
                    );

                    let mut current_view = self.user_view_states.get(username).cloned().unwrap_or(UserViewState::Posts);
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut current_view, UserViewState::Posts, "Posts");
                        ui.selectable_value(&mut current_view, UserViewState::Media, "Media");
                    });
                    self.user_view_states.insert(username.to_string(), current_view.clone());

                    ui.separator();
                });
                strip.strip(|builder| {
                    builder.sizes(Size::remainder(), 1).horizontal(|mut strip| {
                        strip.cell(|ui| match posts {
                            Some(posts) => {
                                let current_view = self.user_view_states.get(username).unwrap();
                                match current_view {
                                    UserViewState::Posts => {
                                        self.make_post_view(ui, username, posts);
                                    }
                                    UserViewState::Media => {
                                        self.make_user_media_view(ui, username, posts);
                                    }
                                }
                            }
                            None => {
                                self.make_placeholder_post_view(ui, username);
                            }
                        });
                    });
                });
            });
    }

                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               

    fn make_user_media_view(&mut self, ui: &mut Ui, username: &str, posts: &mut Vec<FeedItem>) {
        let mut current_size = self.media_image_sizes.get(username).cloned().unwrap_or(200.0);

        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut current_size, 50.0..=500.0).text("Image Size"));
        });
        self.media_image_sizes.insert(username.to_string(), current_size);

        let scroll_offset_y;
        let content_size_y;

        let scroll_area = egui::ScrollArea::vertical();
        let scroll_output = scroll_area.show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (_idx, item) in posts.iter_mut().enumerate() {
                    // Rehydration check (MUST happen before matching if we need data)
                    let mut rehydrate_uri = None;
                    if let FeedItem::Dehydrated { uri } = item {
                        // Dehydrated items will just trigger rehydration immediately in Media View
                        // as we don't do complex virtualization here anymore
                        rehydrate_uri = Some(uri.clone());
                    }
                    if let Some(uri) = rehydrate_uri {
                        if let Some(post) = self.post_cache.remove(&uri) {
                            *item = FeedItem::Full(post);
                            self.post_cache_order.retain(|u| u != &uri);
                        }
                    }

                    match item {
                        FeedItem::Full(post) => {
                            for embed in &post.embeds {
                                if !self.image_cache.contains_key(&embed.thumbnail_url) {
                                    self.request_image(&embed.thumbnail_url);
                                }

                                if let Some(texture) = self.image_cache.get(&embed.thumbnail_url).unwrap_or(&None) {
                                    let img_view = ui.add(egui::Image::new(texture).max_width(current_size).max_height(current_size));
                                    let sensing_img = img_view.interact(egui::Sense::click());

                                    if sensing_img.clicked() {
                                        self.post_ui_message(RedskyUiMsg::ShowBigImageView {
                                            img_uri: embed.url.clone(),
                                        });
                                    }
                                } else {
                                    ui.spinner();
                                }
                            }
                        }
                        FeedItem::Dehydrated { uri: _ } => {}
                    }
                }
            });
        });

        scroll_offset_y = scroll_output.state.offset.y;
        content_size_y = scroll_output.content_size.y;

        // Infinite Scroll Check
        let viewport_height = ui.available_height();
        if (scroll_offset_y + viewport_height) > content_size_y * 0.8 && content_size_y > 0.0 {
            if username == "Your timeline" {
                if let Some(cursor) = self.timeline_cursor.clone() {
                    self.post_message(BskyActorMsg::GetTimeline {
                        cursor: Some(cursor),
                    });
                    self.timeline_cursor = None; // Avoid duplicate requests
                }
            } else if username != "Thread" {
                if let Some(cursor) = self.user_cursors.get(username).cloned().flatten() {
                    self.post_message(BskyActorMsg::GetUserPosts {
                        username: username.to_string(),
                        cursor: Some(cursor),
                    });
                    self.user_cursors.insert(username.to_string(), None); // Avoid duplicate requests
                }
            }
        }

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
