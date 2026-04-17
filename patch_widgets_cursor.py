import re
content = open("src/app/ui_widgets.rs").read()

content = content.replace(
    "for (post_ref, likers) in &self.post_likers_cache {",
    "for (post_ref, (likers, cursor)) in &self.post_likers_cache {"
)
content = content.replace(
    "for (post_ref, reposters) in &self.post_reposters_cache {",
    "for (post_ref, (reposters, cursor)) in &self.post_reposters_cache {"
)

content = content.replace(
    """                        crate::app::show_autoscroll_area(ui, "likers_scroll", false, |ui| {
                            for user in likers {
                                self.make_maybe_user_profile_view(ui, &user.handle, Some(user));
                                ui.separator();
                            }
                        });
                    });""",
    """                        crate::app::show_autoscroll_area(ui, "likers_scroll", false, |ui| {
                            for user in likers {
                                self.make_maybe_user_profile_view(ui, &user.handle, Some(user));
                                ui.separator();
                            }
                        });
                        if let Some(c) = cursor.clone() {
                            if ui.button("Load More").clicked() {
                                self.post_message(BskyActorMsg::GetPostLikers { post_ref: post_ref.clone(), cursor: Some(c) });
                            }
                        }
                    });"""
)

content = content.replace(
    """                            for user in reposters {
                                self.make_maybe_user_profile_view(ui, &user.handle, Some(user));
                                ui.separator();
                            }
                        });
                    });""",
    """                            for user in reposters {
                                self.make_maybe_user_profile_view(ui, &user.handle, Some(user));
                                ui.separator();
                            }
                        });
                        if let Some(c) = cursor.clone() {
                            if ui.button("Load More").clicked() {
                                self.post_message(BskyActorMsg::GetPostRepostedBy { post_ref: post_ref.clone(), cursor: Some(c) });
                            }
                        }
                    });"""
)

open("src/app/ui_widgets.rs", "w").write(content)
