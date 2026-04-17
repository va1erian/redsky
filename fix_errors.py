import re

content = open("src/app/ui_post.rs").read()

content = content.replace(
    "|post_ref| BskyActorMsg::GetPostLikers { post_ref },",
    "|post_ref| BskyActorMsg::GetPostLikers { post_ref, cursor: None },"
).replace(
    "|post_ref| BskyActorMsg::GetPostRepostedBy {",
    "|post_ref| BskyActorMsg::GetPostRepostedBy {\n                                                    cursor: None,"
)

# Dehydration Full pattern replacement
content = content.replace(
    """                            FeedItem::Full(post) => {""",
    """                            FeedItem::Full(post, ref mut height) => {"""
).replace(
    """                                if post_block.response.interact(Sense::click()).clicked() {
                                    self.post_ui_message(RedskyUiMsg::PrepareThreadView {
                                        thread_ref: StrongRef {
                                            uri: post.uri.clone(),
                                            cid: post.cid.clone(),
                                        },
                                    });
                                }
                            }
                            FeedItem::Dehydrated { uri: _ } => {""",
    """                                if post_block.response.interact(Sense::click()).clicked() {
                                    self.post_ui_message(RedskyUiMsg::PrepareThreadView {
                                        thread_ref: StrongRef {
                                            uri: post.uri.clone(),
                                            cid: post.cid.clone(),
                                        },
                                    });
                                }
                                *height = Some(post_block.response.rect.height());
                            }
                            FeedItem::Dehydrated { uri: _, height } => {"""
).replace(
    """                                ui.vertical_centered(|ui| {
                                    ui.add_space(50.0);
                                    ui.spinner();
                                    ui.add_space(50.0);
                                });
                            }""",
    """                                let h = height.unwrap_or(100.0);
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), h),
                                    egui::Sense::hover(),
                                );
                                ui.allocate_ui_at_rect(rect, |ui| {
                                    ui.centered_and_justified(|ui| {
                                        ui.spinner();
                                    });
                                });
                            }"""
)

# Rehydration check
content = content.replace(
    """                        // Rehydration check
                        let mut rehydrate_uri = None;
                        if let FeedItem::Dehydrated { uri } = item {
                            if ui.is_rect_visible(ui.available_rect_before_wrap()) {
                                rehydrate_uri = Some(uri.clone());
                            }
                        }""",
    """                        // Rehydration check
                        let mut rehydrate_uri = None;
                        if let FeedItem::Dehydrated { uri, height: _ } = item {
                            let visible_idx = (scroll_output.state.offset.y / 200.0) as i32;
                            if (idx as i32 - visible_idx).abs() <= 50 {
                                rehydrate_uri = Some(uri.clone());
                            }
                        }"""
)

content = content.replace(
    """                            if let Some(post) = self.post_cache.remove(&uri) {
                                *item = FeedItem::Full(post);
                                self.post_cache_order.retain(|u| u != &uri);
                            }""",
    """                            if let Some(post) = self.post_cache.remove(&uri) {
                                *item = FeedItem::Full(post, None);
                                self.post_cache_order.retain(|u| u != &uri);
                            }"""
)

# Replace Infinite Scroll (we need to be careful with the braces here)
match = re.search(r"// Infinite Scroll Check\n.*?\n        \}\);\n\n        if scroll_top_reset", content, flags=re.DOTALL)
if match:
    replacement = """// Infinite Scroll Check replaced by Load More buttons
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

        if scroll_top_reset"""
    content = content[:match.start()] + replacement + content[match.end():]

# Dehydration loops replace
content = content.replace(
    """        // Dehydration logic
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
}""",
    """        // Dehydration logic
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
}"""
)

open("src/app/ui_post.rs", "w").write(content)

content = open("src/app/ui_user.rs").read()

content = re.sub(
    r"// Infinite Scroll Check\n\s+let viewport_height = ui\.available_height\(\);\n\s+if \(scroll_offset_y \+ viewport_height\) > content_size_y \* 0\.8 && content_size_y > 0\.0 \{.*?\n\s+\}\n\s+\}\n\s+\}\n\s+\}\n\n",
    """// Infinite Scroll Check replaced by Load More
        if username == "Your timeline" {
            if let Some(cursor) = self.timeline_cursor.clone() {
                if ui.button("Load More").clicked() {
                    self.post_message(BskyActorMsg::GetTimeline {
                        cursor: Some(cursor),
                    });
                    self.timeline_cursor = None;
                }
            }
        } else if username != "Thread" {
            let current_view = self.user_view_states.get(username).cloned().unwrap_or(UserViewState::Posts);
            match current_view {
                UserViewState::Posts | UserViewState::Media => {
                    if let Some(cursor) = self.user_cursors.get(username).cloned().flatten() {
                        if ui.button("Load More").clicked() {
                            self.post_message(BskyActorMsg::GetUserPosts {
                                username: username.to_string(),
                                cursor: Some(cursor),
                            });
                            self.user_cursors.insert(username.to_string(), None);
                        }
                    }
                }
                UserViewState::Liked => {
                    if let Some(cursor) = self.user_likes_cursors.get(username).cloned().flatten() {
                        if ui.button("Load More").clicked() {
                            self.post_message(BskyActorMsg::GetUserLikes {
                                username: username.to_string(),
                                cursor: Some(cursor),
                            });
                            self.user_likes_cursors.insert(username.to_string(), None);
                        }
                    }
                }
            }
        }\n\n""",
    content,
    flags=re.DOTALL
)

content = content.replace(
    """                    let mut rehydrate_uri = None;
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
                        FeedItem::Full(post) => {""",
    """                    let mut rehydrate_uri = None;
                    if let FeedItem::Dehydrated { uri, height: _ } = item {
                        rehydrate_uri = Some(uri.clone());
                    }
                    if let Some(uri) = rehydrate_uri {
                        if let Some(post) = self.post_cache.remove(&uri) {
                            *item = FeedItem::Full(post, None);
                            self.post_cache_order.retain(|u| u != &uri);
                        }
                    }

                    match item {
                        FeedItem::Full(post, _) => {"""
).replace(
    """                        FeedItem::Dehydrated { uri: _ } => {}""",
    """                        FeedItem::Dehydrated { uri: _, height: _ } => {}"""
)

open("src/app/ui_user.rs", "w").write(content)
