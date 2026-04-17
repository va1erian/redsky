import re

content = open("src/app/types.rs").read()

content = content.replace(
    """pub enum FeedItem {
    Full(Post),
    Dehydrated { uri: String },
}""",
    """pub enum FeedItem {
    Full(Post, Option<f32>),
    Dehydrated { uri: String, height: Option<f32> },
}"""
)

content = content.replace(
    "posts.into_iter().map(FeedItem::Full).collect()",
    "posts.into_iter().map(|p| FeedItem::Full(p, None)).collect()"
)

content = content.replace(
    "pub zoom_factor: f32,",
    "pub zoom_factor: f32,\n    pub allow_dehydration: bool,"
)
content = content.replace(
    "zoom_factor: 1.0,",
    "zoom_factor: 1.0,\n            allow_dehydration: true,"
)

open("src/app/types.rs", "w").write(content)
