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
            if let Some(img_data) = self.image_cache.get(&post.avatar_img) {
                ui.vertical(|ui| {
                    ui.set_max_width(57.6f32);
                    self.make_buffer_image_view(
                        ui,
                        &post.avatar_img,
                        img_data,
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
            let scroll_output = crate::app::show_autoscroll_area(ui, "post_scroll", false, |ui| {
                ui.vertical(|ui| {
                    for (idx, item) in posts.iter_mut().enumerate() {
                        match item {
                            FeedItem::Full(post, ref mut height) => {
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
                                                self.make_post_inner_view(ui, quoted_post);
                                            });
                                    }

                                    if !&post.embeds.is_empty() {
                                        ui.horizontal_wrapped(|ui| {
                                            ui.set_min_height(200f32);
                                            for embed in &post.embeds {
                                                if let Some(img_data) = self.image_cache.get(&embed.thumbnail_url) {
                                                    self.make_buffer_image_view(
                                                        ui,
                                                        &embed.thumbnail_url,
                                                        img_data,
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
                                        let like_btn = ui.button(like_text).on_hover_text("Right-click to see likers");
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
                                                |post_ref| BskyActorMsg::GetPostLikers { post_ref, cursor: None },
                                            );
                                        });

                                        let repost_text = if post.viewer_repost.is_some() {
                                            RichText::new(format!("{} x 🔃", &post.repost_count))
                                                .color(egui::Color32::GREEN)
                                        } else {
                                            RichText::new(format!("{} x 🔃", &post.repost_count))
                                        };
                                        let repost_btn = ui.button(repost_text).on_hover_text("Right-click to see reposters");
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
                                                    cursor: None,
                                                    post_ref,
                                                },
                                            );
                                        });

                                        if ui.button("Reply").clicked() {
                                            self.is_post_window_open = true;
                                            let parent_ref = StrongRef {
                                                uri: post.uri.clone(),
                                                cid: post.cid.clone(),
                                            };
                                            let root_ref = post.thread_root.clone().unwrap_or(parent_ref.clone());
                                            self.reply_to = Some((root_ref, parent_ref));
                                        }

                                        ui.menu_button("…", |ui| {
                                            if ui.add_enabled(
                                                post.author == self.login,
                                                egui::Button::new("Delete Post")
                                            ).clicked() {
                                                self.post_ui_message(RedskyUiMsg::DeletePost {
                                                    post_uri: post.uri.clone(),
                                                    post_cid: post.cid.clone(),
                                                });
                                                ui.close();
                                            }
                                            if ui.button("Raw View").clicked() {
                                                self.post_message(BskyActorMsg::GetRawPost {
                                                    post_uri: post.uri.clone(),
                                                });
                                                ui.close();
                                            }
                                        });
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
                                *height = Some(post_block.response.rect.height());
                            }
                            FeedItem::Dehydrated { uri: _, height } => {
                                let h = height.unwrap_or(100.0);
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), h),
                                    egui::Sense::hover(),
                                );
                                ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                                    ui.centered_and_justified(|ui| {
                                        ui.spinner();
                                    });
                                });
                            }
                        }

                        // Rehydration check
                        let mut rehydrate_uri = None;
                        if let FeedItem::Dehydrated { uri, height: _ } = item {
                            let visible = ui.is_rect_visible(ui.cursor());
                            if visible {
                                rehydrate_uri = Some(uri.clone());
                            }
                        }
                        if let Some(uri) = rehydrate_uri {
                            if let Some(post) = self.post_cache.remove(&uri) {
                                *item = FeedItem::Full(post, None);
                                self.post_cache_order.retain(|u| u != &uri);
                            }
                        }
                    }
                });
            });
            scroll_offset_y = scroll_output.state.offset.y;
            content_size_y = scroll_output.content_size.y;

            // Infinite Scroll Check replaced by Load More buttons
            if username == "Your timeline" {
                if let Some(cursor) = self.timeline_cursor.clone() {
                    if ui.button("Load More").clicked() {
                        self.post_message(BskyActorMsg::GetTimeline {
                            cursor: Some(cursor),
                        });
                        self.timeline_cursor = None; // Avoid duplicate requests
                    }
                }
            } else if username == "Search Results" {
                if let Some(cursor) = self.search_posts_cursor.clone() {
                    if ui.button("Load More").clicked() {
                        self.post_message(BskyActorMsg::SearchPosts {
                            query: self.search_posts_query.clone(),
                            cursor: Some(cursor),
                        });
                        self.search_posts_cursor = None; // Avoid duplicate requests
                    }
                }
            } else if username == "Your bookmarks" {
                if let Some(cursor) = self.bookmarks_cursor.clone() {
                    if ui.button("Load More").clicked() {
                        self.post_message(BskyActorMsg::GetBookmarks {
                            cursor: Some(cursor),
                        });
                        self.bookmarks_cursor = None; // Avoid duplicate requests
                    }
                }
            } else if username != "Thread" {
                // It's a user view
                if let Some(cursor) = self.user_cursors.get(username).cloned().flatten() {
                    if ui.button("Load More (Posts)").clicked() {
                        self.post_message(BskyActorMsg::GetUserPosts {
                            username: username.to_string(),
                            cursor: Some(cursor),
                        });
                        self.user_cursors.insert(username.to_string(), None); // Avoid duplicate requests
                    }
                }
                if let Some(cursor) = self.user_likes_cursors.get(username).cloned().flatten() {
                    if ui.button("Load More (Likes)").clicked() {
                        self.post_message(BskyActorMsg::GetUserLikes {
                            username: username.to_string(),
                            cursor: Some(cursor),
                        });
                        self.user_likes_cursors.insert(username.to_string(), None); // Avoid duplicate requests
                    }
                }
            }
        });

        if scroll_top_reset {
            self.scroll_to_top = false;
        }

        // Dehydration logic
        if self.settings.allow_dehydration {
            let visible_idx = (scroll_offset_y / 200.0) as i32;
            for (idx, item) in posts.iter_mut().enumerate() {
                let mut should_dehydrate = false;
                if let FeedItem::Full(_, _) = item {
                    if (idx as i32 - visible_idx).abs() > 50 {
                        should_dehydrate = true;
                    }
                }

                if should_dehydrate {
                    if let FeedItem::Full(post, height) =
                        std::mem::replace(item, FeedItem::Dehydrated { uri: String::new(), height: None })
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
                            *item = FeedItem::Dehydrated { uri, height };
                        }
                    }
                }
            }
        }
    }
}
