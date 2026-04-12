impl RedskyApp {

    fn make_post_context_menu_item<F>(&self, ui: &mut egui::Ui, text: &str, post: &Post, msg_constructor: F)
    where
        F: FnOnce(StrongRef) -> BskyActorMsg,
    {
        if ui.button(text).clicked() {
            self.post_message(msg_constructor(StrongRef {
                uri: post.uri.clone(),
                cid: post.cid.clone(),
            }));
            ui.close();
        }
    }

    fn make_placeholder_post_view(&mut self, ui: &mut Ui, username: &str) {
        ui.vertical(|ui| {
            ui.heading(username);
            ui.separator();
            ui.spinner();
        });
    }

                                                                                                                                                                                                                 

    fn make_buffer_image_view(
        &self,
        ui: &mut Ui,
        _image_uri: &String,
        img_data: &Option<egui::TextureHandle>,
        full_view_uri: Option<&String>,
    ) {
        if let Some(texture) = img_data {
            let max_width = if full_view_uri.is_some() {
                self.settings.max_image_size.min(ui.available_width())
            } else {
                ui.available_width()
            };
            let img_view = ui.add(egui::Image::new(texture).max_width(max_width));
            let sensing_img = img_view.interact(egui::Sense::click());

            if sensing_img.clicked() {
                if let Some(uri) = full_view_uri {
                    dbg!(uri);
                    self.post_ui_message(RedskyUiMsg::ShowBigImageView {
                        img_uri: uri.clone(),
                    });
                }
            }
        } else {
            ui.spinner();
        }
    }

                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     

    fn make_post_inner_view(&self, ui: &mut Ui, post: &Post) {
        ui.horizontal(|ui| {
            ui.set_min_height(57.6f32);
            if self.image_cache.contains_key(&post.avatar_img) {
                ui.vertical(|ui| {
                    ui.set_max_width(57.6f32);
                    self.make_buffer_image_view(
                        ui,
                        &post.avatar_img,
                        self.image_cache.get(&post.avatar_img).unwrap(),
                        Some(&post.avatar_img),
                    );
                });
            }

            ui.vertical(|ui| {
                if ui
                    .link(RichText::new(&post.display_name).strong())
                    .clicked()
                {
                    self.post_ui_message(RedskyUiMsg::PrepareUserView {
                        username: post.author.clone(),
                    });
                    self.post_message(BskyActorMsg::GetUserPosts {
                        username: post.author.clone(),
                        cursor: None,
                    });
                };
                ui.label(&post.author);
                ui.label(RichText::new(&post.date).small())
            });
        });
        ui.style_mut().spacing.item_spacing = vec2(16.0, 16.0);
        ui.label(&post.content);
    }

                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            

    fn make_post_view(&mut self, ui: &mut Ui, username: &str, posts: &mut Vec<FeedItem>) {
        let mut scroll_top_reset = false;
        let mut scroll_offset_y = 0.0;
        let mut content_size_y = 0.0;

        ui.vertical_centered_justified(|ui| {
            let scroll_area = egui::ScrollArea::vertical();
            let scroll_output = scroll_area.show(ui, |ui| {
                ui.vertical(|ui| {
                    for (idx, item) in posts.iter_mut().enumerate() {
                        match item {
                            FeedItem::Full(post) => {
                                let post_block = ui.vertical(|ui| {
                                    if idx == 0 && self.scroll_to_top {
                                        ui.scroll_to_rect(ui.max_rect(), Some(egui::Align::TOP));
                                        scroll_top_reset = true;
                                    }
                                    self.make_post_inner_view(ui, post);

                                    if let Some(quoted_post) = &post.quoted_post {
                                        egui::Frame::new()
                                            .inner_margin(8)
                                            .outer_margin(8)
                                            .corner_radius(8)
                                            .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
                                            .show(ui, |ui| {
                                                self.make_post_inner_view(ui, &quoted_post);
                                            });
                                    }

                                    if !&post.embeds.is_empty() {
                                        ui.horizontal_wrapped(|ui| {
                                            ui.set_min_height(200f32);
                                            for embed in &post.embeds {
                                                if self
                                                    .image_cache
                                                    .contains_key(&embed.thumbnail_url)
                                                {
                                                    self.make_buffer_image_view(
                                                        ui,
                                                        &embed.thumbnail_url,
                                                        self.image_cache
                                                            .get(&embed.thumbnail_url)
                                                            .unwrap(),
                                                        Some(&embed.url),
                                                    );
                                                }
                                            }
                                        });
                                    }
                                    ui.horizontal(|ui| {
                                        let like_text = if post.viewer_like.is_some() {
                                            RichText::new(format!("{} x ❤", &post.like_count))
                                                .color(egui::Color32::RED)
                                        } else {
                                            RichText::new(format!("{} x ❤", &post.like_count))
                                        };
                                        let like_btn = ui.button(like_text);
                                        if like_btn.clicked() {
                                            let post_uri = post.uri.clone();
                                            let post_cid = post.cid.clone();
                                            self.post_ui_message(RedskyUiMsg::PrepareLikeAction {
                                                post_uri,
                                                post_cid,
                                                unlike: post.viewer_like.is_some(),
                                            });
                                        }
                                        like_btn.context_menu(|ui| {
                                            self.make_post_context_menu_item(
                                                ui,
                                                "Show Likers",
                                                post,
                                                |post_ref| BskyActorMsg::GetPostLikers { post_ref },
                                            );
                                        });

                                        let repost_text = if post.viewer_repost.is_some() {
                                            RichText::new(format!("{} x 🔃", &post.repost_count))
                                                .color(egui::Color32::GREEN)
                                        } else {
                                            RichText::new(format!("{} x 🔃", &post.repost_count))
                                        };
                                        let repost_btn = ui.button(repost_text);
                                        if repost_btn.clicked() {
                                            let post_uri = post.uri.clone();
                                            let post_cid = post.cid.clone();
                                            self.post_ui_message(
                                                RedskyUiMsg::PrepareRepostAction {
                                                    post_uri,
                                                    post_cid,
                                                    unrepost: post.viewer_repost.is_some(),
                                                },
                                            );
                                        }
                                        repost_btn.context_menu(|ui| {
                                            self.make_post_context_menu_item(
                                                ui,
                                                "Show Reposters",
                                                post,
                                                |post_ref| BskyActorMsg::GetPostRepostedBy {
                                                    post_ref,
                                                },
                                            );
                                        });

                                        let _ = ui.button("…");
                                    });
                                    ui.separator();
                                });

                                if post_block.response.interact(Sense::click()).clicked() {
                                    self.post_ui_message(RedskyUiMsg::PrepareThreadView {
                                        thread_ref: StrongRef {
                                            uri: post.uri.clone(),
                                            cid: post.cid.clone(),
                                        },
                                    });
                                }
                            }
                            FeedItem::Dehydrated { uri: _ } => {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(50.0);
                                    ui.spinner();
                                    ui.add_space(50.0);
                                });
                            }
                        }

                        // Rehydration check
                        let mut rehydrate_uri = None;
                        if let FeedItem::Dehydrated { uri } = item {
                            if ui.is_rect_visible(ui.available_rect_before_wrap()) {
                                rehydrate_uri = Some(uri.clone());
                            }
                        }
                        if let Some(uri) = rehydrate_uri {
                            if let Some(post) = self.post_cache.remove(&uri) {
                                *item = FeedItem::Full(post);
                                self.post_cache_order.retain(|u| u != &uri);
                            }
                        }
                    }
                });
            });
            scroll_offset_y = scroll_output.state.offset.y;
            content_size_y = scroll_output.content_size.y;

            // Infinite Scroll Check
            if scroll_offset_y > content_size_y * 0.8 && content_size_y > 0.0 {
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
        });

        if scroll_top_reset {
            self.scroll_to_top = false;
        }

        // Dehydration logic
        let visible_idx = (scroll_offset_y / 200.0) as i32;
        for (idx, item) in posts.iter_mut().enumerate() {
            let mut should_dehydrate = false;
            if let FeedItem::Full(_) = item {
                if (idx as i32 - visible_idx).abs() > 50 {
                    should_dehydrate = true;
                }
            }

            if should_dehydrate {
                if let FeedItem::Full(post) =
                    std::mem::replace(item, FeedItem::Dehydrated { uri: String::new() })
                {
                    let uri = post.uri.clone();
                    if !uri.is_empty() {
                        self.post_cache.insert(uri.clone(), post);
                        self.post_cache_order.push_back(uri.clone());

                        // LRU Eviction
                        if self.post_cache.len() > 200 {
                            if let Some(oldest_uri) = self.post_cache_order.pop_front() {
                                self.post_cache.remove(&oldest_uri);
                            }
                        }
                        *item = FeedItem::Dehydrated { uri };
                    }
                }
            }
        }
    }
}
