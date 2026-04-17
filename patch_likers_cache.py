import re
content = open("src/app/mod.rs").read()

content = content.replace(
    "    post_likers_cache: HashMap<StrongRef, Vec<UserProfile>>,",
    "    post_likers_cache: HashMap<StrongRef, (Vec<UserProfile>, Option<String>)>,",
)
content = content.replace(
    "    post_reposters_cache: HashMap<StrongRef, Vec<UserProfile>>,",
    "    post_reposters_cache: HashMap<StrongRef, (Vec<UserProfile>, Option<String>)>,",
)

open("src/app/mod.rs", "w").write(content)
