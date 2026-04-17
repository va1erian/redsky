import re

content = open("src/app/msg_handler.rs").read()
content = content.replace("vec![FeedItem::Full(post)]", "vec![FeedItem::Full(post, None)]")
content = content.replace("replies.into_iter().map(FeedItem::Full)", "replies.into_iter().map(|p| FeedItem::Full(p, None))")
open("src/app/msg_handler.rs", "w").write(content)

content = open("src/app/mod.rs").read()
content = content.replace("FeedItem::Full(Post {", "FeedItem::Full(Post {")
content = content.replace(
    """                raw_json: "{}".to_string(),
            }));""",
    """                raw_json: "{}".to_string(),
            }, None));"""
)
content = content.replace("FeedItem::Full(post) = item", "FeedItem::Full(post, _) = item")
open("src/app/mod.rs", "w").write(content)
