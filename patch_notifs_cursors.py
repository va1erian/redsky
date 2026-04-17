import re

content = open("src/app/mod.rs").read()
if "pub notifications_cursor: Option<String>," not in content:
    content = content.replace("    pub settings: AppSettings,", "    pub notifications_cursor: Option<String>,\n    pub bookmarks_cursor: Option<String>,\n    pub settings: AppSettings,")
    content = content.replace("            settings: AppSettings::load(),", "            notifications_cursor: None,\n            bookmarks_cursor: None,\n            settings: AppSettings::load(),")
open("src/app/mod.rs", "w").write(content)
