import re
content = open("src/app/mod.rs").read()

content = content.replace(
    """                                    if ui.button("Refresh notifications").clicked() {
                                        self.post_message(BskyActorMsg::GetNotifications());
                                    }""",
    """                                    if ui.button("Refresh notifications").clicked() {
                                        self.post_message(BskyActorMsg::GetNotifications { cursor: None });
                                    }
                                    if let Some(cursor) = self.notifications_cursor.clone() {
                                        if ui.button("Load More").clicked() {
                                            self.post_message(BskyActorMsg::GetNotifications { cursor: Some(cursor) });
                                            self.notifications_cursor = None;
                                        }
                                    }"""
)

open("src/app/mod.rs", "w").write(content)
