use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use atrium_api::app::bsky::embed::record::ViewRecordRefs;
use atrium_api::app::bsky::feed::defs::PostViewData;
use atrium_api::app::bsky::feed::defs::PostViewEmbedRefs;
use atrium_api::app::bsky::feed::defs::ThreadViewPostRepliesItem;
use atrium_api::app::bsky::feed::get_post_thread::OutputThreadRefs;
use atrium_api::app::bsky::feed::post;
use atrium_api::types::string::AtIdentifier;
use atrium_api::types::string::Datetime;
use atrium_api::types::Object;
use atrium_api::types::TryFromUnknown;
use atrium_api::types::Union;
use bsky_sdk::BskyAgent;

use crate::app::BskyActorMsg;
use crate::app::DownloadStatus;
use crate::app::Post;
use crate::app::PostImage;
use crate::app::RedskyUiMsg;
use crate::app::StrongRef;
use crate::app::UserProfile;
use reqwest;
use std::collections::HashMap;
use tokio::sync::oneshot;


pub struct BskyActor {
    tx: Sender<RedskyUiMsg>,
    rx: Receiver<BskyActorMsg>,
    bsky_agent: BskyAgent,
    ctx: egui::Context,
    cancel_txs: HashMap<u64, oneshot::Sender<()>>,
}

struct BskyJob {
    job: BskyActorMsg,
    tx: Sender<RedskyUiMsg>,
    bsky_agent: BskyAgent,
    ctx: egui::Context, //for force repaint
}

impl BskyActor {
    pub fn new(bsky_agent: BskyAgent, ctx: egui::Context, rx: Receiver<BskyActorMsg>, tx: Sender<RedskyUiMsg>) -> Self {
        Self {
            tx,
            rx,
            bsky_agent,
            ctx,
            cancel_txs: HashMap::new(),
        }
    }

    pub fn pump(&mut self) -> bool {
        match self.rx.recv() {
            Ok(msg) => {
                match msg {
                    BskyActorMsg::Close() => {
                        println!("bsky actor: closing");
                        false
                    }
                    BskyActorMsg::CancelImageDownload { id } => {
                        if let Some(tx) = self.cancel_txs.remove(&id) {
                            let _ = tx.send(());
                        }
                        true
                    }
                    BskyActorMsg::StartImageDownload { id, username, path } => {
                        let (tx, rx) = oneshot::channel();
                        self.cancel_txs.insert(id, tx);
                        let job = BskyJob {
                            job: BskyActorMsg::StartImageDownload { id, username, path },
                            tx: self.tx.clone(),
                            bsky_agent: self.bsky_agent.clone(),
                            ctx: self.ctx.clone()
                        };
                        tokio::spawn(async move {
                            tokio::select! {
                                _ = job.perform() => {},
                                _ = rx => {
                                    println!("Job {} cancelled", id);
                                }
                            }
                        });
                        true
                    }
                    _ => {
                        let job = BskyJob {
                            job: msg,
                            tx: self.tx.clone(),
                            bsky_agent: self.bsky_agent.clone(),
                            ctx: self.ctx.clone()
                        };
                        tokio::spawn(job.perform());
                        true
                    }
                }
            }
            Err(err) => {
                println!("bsky actor: closed receiving chan - {}", err);
                false
            }
        }
    }
}

fn extract_quote_reply(post_view: &Object<PostViewData> ) -> Option<Post> {
    if let Some(Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordView(embedded_record))) = &post_view.embed {
        if let Union::Refs(ViewRecordRefs::ViewRecord(view_record )) = &embedded_record.record {
            let quote_post_data = post::RecordData::try_from_unknown(view_record.value.clone()).unwrap();
            Some(Post{
                uri: view_record.uri.clone(),
                cid: view_record.cid.clone(),
                content: quote_post_data.text,
                author: view_record.author.handle.to_string(),
                display_name: view_record.author.display_name.clone().unwrap_or("".to_string()),
                avatar_img: view_record.author.avatar.clone().unwrap_or("".to_string()),
                date: quote_post_data.created_at.as_str().to_string(),
                like_count: view_record.like_count.unwrap_or(0),
                repost_count: view_record.repost_count.unwrap_or(0),
                embeds: vec![],
                quoted_post: None,
                is_reply: quote_post_data.reply.is_some()
            })
        } else {
            None
        }
    } else {
        None
    }
}

fn extract_images(post_view: &Object<PostViewData>) -> Vec<PostImage> {
    post_view.embed.clone().map(|embed_el: Union<atrium_api::app::bsky::feed::defs::PostViewEmbedRefs>| {
        if let Union::Refs(PostViewEmbedRefs::AppBskyEmbedImagesView(data)) = embed_el {
            data.images.iter().map(|img| 
                PostImage::new( img.thumb.to_string(),img.fullsize.to_string(),img.alt.to_string())
                
            ).collect()
        } else {
            vec![]
        }
    }).into_iter().flatten().collect()
}

fn extract_post(post_view: &Object<PostViewData>) -> Post {
    let post_record_data = post::RecordData::try_from_unknown(post_view.data.record.clone()).unwrap();
    let images : Vec<PostImage> = extract_images(post_view);
    let quoted_post: Option<Post> = extract_quote_reply(post_view);

    Post {
        uri: post_view.uri.clone(),
        cid: post_view.cid.clone(),
        content: post_record_data.text.clone(),
        author: post_view.author.handle.to_string(),
        display_name: post_view.author.display_name.clone().unwrap_or_default(),
        avatar_img: post_view.author.avatar.clone().unwrap_or("".to_string()),
        date: post_record_data.created_at.as_str().to_string(),
        like_count: post_view.like_count.unwrap_or(0),
        repost_count: post_view.repost_count.unwrap_or(0),
        embeds: images,
        quoted_post: quoted_post.map(|post| Box::new(post)),
        is_reply: post_record_data.reply.is_some()
    }
}

impl BskyJob {
    pub async fn perform(self) -> () {
        let result = match &self.job {
            BskyActorMsg::Login { login, pass } => {
                self.login(login, pass).await
            }
            BskyActorMsg::Post { msg_body } => {
                self.post(msg_body).await
            }
            BskyActorMsg::GetPostAndReplies { post_ref } => {
                self.get_post_thread(post_ref).await
            }
            //BskyActorMsg::GetPostLikers {post_ref } => {
            //    self.get_post_likers(post_ref).await
            //}
            BskyActorMsg::GetTimeline { cursor } => {
                self.get_timeline_posts(cursor).await
            }
            BskyActorMsg::GetBookmarks() => {
                self.get_bookmarks().await
            }
            BskyActorMsg::GetUserProfile { username } => {
                self.get_user_profile(username).await
            }
            BskyActorMsg::GetUserPosts { username, cursor } => {
                self.get_user_posts(username, cursor).await
            }
            BskyActorMsg::SearchActors { query } => {
                self.search_actors(query).await
            }
            BskyActorMsg::LoadImage { url } => {
                self.load_image(url).await
            }
            BskyActorMsg::StartImageDownload { id, username, path } => {
                self.download_all_images(*id, username, path).await
            }
            BskyActorMsg::CancelImageDownload { .. } => {
                Ok(RedskyUiMsg::LogInSucceededMsg()) // dummy
            }
            BskyActorMsg::Close() => {
                panic!("unexpected message");
            }
        };
        if let Ok(reply) = result {
            self.post_to_ui(reply);
        } else if let Err(e) = result {
            self.post_to_ui(RedskyUiMsg::ShowErrorMsg { error: e.to_string() });
        }
    }

    pub fn post_to_ui(&self, msg: RedskyUiMsg) {
        self.tx.send(msg).unwrap();
        self.ctx.request_repaint();
    }

    async fn get_post_likers(&self, strong_ref: &StrongRef)  -> Result<RedskyUiMsg,  Box<dyn std::error::Error + Send + Sync>> {
        dbg!("get likers");

        let response = self.bsky_agent
        .api
        .app
        .bsky
        .feed
        .get_likes(atrium_api::app::bsky::feed::get_likes::ParametersData {
            cid: Some(strong_ref.cid.clone()),
            uri: strong_ref.uri.clone(),
            cursor: None,
            limit: None
        }.into()).await?;

    let likers = response.data.likes.iter().map(|like_data| {
            let profile = &like_data.actor;
            UserProfile {
                handle: profile.handle.clone().into(),
                display_name: profile.display_name.clone().unwrap_or("(no display name)".to_string()),
                bio: profile.description.clone().unwrap_or("(No bio)".to_string()),
                avatar_uri: profile.avatar.clone().unwrap_or("".to_string()),
                follower_count: 0,
                follow_count: 0,
                post_count: 0
            }
        }).collect();

        Ok(RedskyUiMsg::NotifyLikesLoaded { post_uri: strong_ref.clone(), likers })
    }

    async fn get_post_thread(&self, strong_ref: &StrongRef) -> Result<RedskyUiMsg,  Box<dyn std::error::Error + Send + Sync>> {
        dbg!("get post thread");

        let response = self.bsky_agent
        .api
        .app
        .bsky
        .feed
        .get_post_thread(atrium_api::app::bsky::feed::get_post_thread::ParametersData {
            uri: strong_ref.uri.clone(),
            depth: 1.try_into().ok(),
            parent_height: 0.try_into().ok()
        }.into()).await?;

        if let atrium_api::types::Union::Refs(
            OutputThreadRefs::AppBskyFeedDefsThreadViewPost(post_data)) = &response.data.thread {
            let replies = match &post_data.replies {
                Some(reply_list) => {
                    reply_list.iter().map(|reply| {
                        match reply {
                            Union::Refs(maybe_reply) => {
                                if let ThreadViewPostRepliesItem::ThreadViewPost(view) = maybe_reply {
                                    vec![extract_post(&view.post)]
                                } else {
                                    vec![]
                                }
                            }
                            Union::Unknown(_) => {
                                vec![]
                            }
                        }
                    }).flatten().collect()
                }
                None => {
                    vec![]
                }
            };

            Ok(RedskyUiMsg::NotifyPostAndRepliesLoaded { 
                post: extract_post(&post_data.post), 
                replies
            })
        } else {
        Ok(RedskyUiMsg::PostSucceeed())
        }
    }

    async fn load_image(&self, url: &String) -> Result<RedskyUiMsg,  Box<dyn std::error::Error + Send + Sync>> {
        let resp = reqwest::get(url)
        .await?;
        let bytes = resp.bytes().await?;
        Ok(RedskyUiMsg::NotifyImageLoaded { url: url.to_string(), data: bytes.to_vec().into() })
    }

    async fn search_actors(&self, query: &String) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("search actors", &query);
        let response = self.bsky_agent
            .api
            .app
            .bsky
            .actor
            .search_actors_typeahead(atrium_api::app::bsky::actor::search_actors_typeahead::ParametersData {
                limit: 10.try_into().ok(),
                q: Some(query.clone()),
                term: None // DEPRECATED: use 'q' instead.
            }.into()).await?;

        let results = response.data.actors.iter().map(|actor| {
            UserProfile {
                handle: actor.handle.to_string(),
                display_name: actor.display_name.clone().unwrap_or("(no display name)".to_string()),
                bio: "".to_string(),
                avatar_uri: actor.avatar.clone().unwrap_or("".to_string()),
                follower_count: 0,
                follow_count: 0,
                post_count: 0
            }
        }).collect();

        Ok(RedskyUiMsg::ShowSearchResults { results })
    }

    async fn get_user_posts(&self, username: &String, cursor: &Option<String>)  -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("get user posts");
        let at_uri = format!("at://{}", username);
        dbg!(&at_uri);
        let response = self.bsky_agent
        .api
        .app
        .bsky
        .feed
        .get_author_feed(atrium_api::app::bsky::feed::get_author_feed::ParametersData {
            actor: AtIdentifier::Handle(username.parse().map_err(|e| format!("Invalid handle: {}", e))?),
            cursor: cursor.clone(),
            filter: None,
            include_pins: Some(true),
            limit: 30.try_into().ok()
        }.into()).await?;

        Ok(RedskyUiMsg::ShowUserPostsMsg{
            username: username.to_string(),
            posts: response.data.feed.iter().map(|post_el: &atrium_api::types::Object<atrium_api::app::bsky::feed::defs::FeedViewPostData>| {
                extract_post(&post_el.post)
        }).collect(),
            cursor: response.data.cursor,
            append: cursor.is_some()
        })
    }

    async fn get_user_profile(&self, username: &String) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("get user profile", &username);

        let profile = self.bsky_agent
        .api
        .app
        .bsky
        .actor
        .get_profile(atrium_api::app::bsky::actor::get_profile::ParametersData{
            actor: AtIdentifier::Handle(username.parse().map_err(|e| format!("Invalid handle: {}", e))?)
        }.into()).await?;

        Ok(RedskyUiMsg::ShowUserProfile { 
            profile: UserProfile {
                handle: username.clone(),
                display_name: profile.display_name.clone().unwrap_or("(no display name)".to_string()),
                bio: profile.description.clone().unwrap_or("(No bio)".to_string()),
                avatar_uri: profile.avatar.clone().unwrap_or("".to_string()),
                follower_count: profile.followers_count.unwrap_or_default(),
                follow_count: profile.follows_count.unwrap_or_default(),
                post_count: profile.posts_count.unwrap_or_default()
        }})
    }

    async fn get_timeline_posts(&self, cursor: &Option<String>) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("get tl");

        let response = self.bsky_agent
        .api
        .app
        .bsky
        .feed
        .get_timeline( atrium_api::app::bsky::feed::get_timeline::ParametersData{
            algorithm: None,
            cursor: cursor.clone(),
            limit: 30.try_into().ok()
        }.into()).await?;

        Ok(RedskyUiMsg::RefreshTimelineMsg 
            { 
                posts: response.data.feed.iter().map(|feed_element| {
                   extract_post(&feed_element.post)
            }).collect(),
                cursor: response.data.cursor,
                append: cursor.is_some()
        })
    }

    async fn get_bookmarks(&self) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("get bookmarks");

        let response = self.bsky_agent
            .api
            .app
            .bsky
            .feed
            .get_bookmarks(atrium_api::app::bsky::feed::get_bookmarks::ParametersData {
                cursor: None,
                limit: 30.try_into().ok(),
            }.into()).await?;

        Ok(RedskyUiMsg::RefreshBookmarksMsg {
            posts: response.data.posts.iter().map(|post_view| {
                extract_post(post_view)
            }).collect()
        })
    }

    async fn login(&self, login: &String, pass: &String) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("loggin in");
        let _ = self.bsky_agent.login(login, pass).await?;
        Ok(RedskyUiMsg::LogInSucceededMsg())
    } 

    async fn download_all_images(&self, id: u64, username: &String, path: &String) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        let mut all_posts = Vec::new();
        let mut cursor = None;

        // 1. Scan all posts
        loop {
            let response = self.bsky_agent
                .api
                .app
                .bsky
                .feed
                .get_author_feed(atrium_api::app::bsky::feed::get_author_feed::ParametersData {
                    actor: AtIdentifier::Handle(username.parse().map_err(|e| format!("Invalid handle: {}", e))?),
                    cursor: cursor.clone(),
                    filter: None,
                    include_pins: Some(true),
                    limit: 100.try_into().ok()
                }.into()).await?;

            let posts_in_batch: Vec<Post> = response.data.feed.iter().map(|post_el| {
                extract_post(&post_el.post)
            }).collect();

            all_posts.extend(posts_in_batch);
            cursor = response.data.cursor;

            self.post_to_ui(RedskyUiMsg::DownloadProgress {
                id,
                processed_posts: all_posts.len(),
                total_posts: None,
                downloaded_images: 0,
                total_images: None,
                status: DownloadStatus::Scanning,
            });

            if cursor.is_none() {
                break;
            }
            // Rate limiting awareness
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        // 2. Filter images (only those posted by this account as requested, skip replies and reposts)
        let mut images_to_download = Vec::new();
        for post in &all_posts {
            // Note: atrium_api::app::bsky::feed::defs::FeedViewPostData also has 'reason' for reposts,
            // but here we filter by author and check if it's a reply.
            if post.author == *username && !post.is_reply {
                for img in &post.embeds {
                    images_to_download.push((img.url.clone(), post.date.clone()));
                }
            }
        }

        let total_images = images_to_download.len();
        self.post_to_ui(RedskyUiMsg::DownloadProgress {
            id,
            processed_posts: all_posts.len(),
            total_posts: Some(all_posts.len()),
            downloaded_images: 0,
            total_images: Some(total_images),
            status: DownloadStatus::Downloading,
        });

        // 3. Download images
        let mut downloaded_count = 0;
        let mut errors = Vec::new();
        let target_dir = std::path::Path::new(path);

        for (url, date) in images_to_download {
            {
                let result = async {
                    let resp = reqwest::get(&url).await?;
                    let bytes = resp.bytes().await?;

                let raw_filename = url.split('/').last().unwrap_or("image");
                let extension = if raw_filename.contains("@png") {
                    "png"
                } else if raw_filename.contains("@jpeg") || raw_filename.contains("@jpg") {
                    "jpg"
                } else if raw_filename.contains("@webp") {
                    "webp"
                } else if raw_filename.contains("@gif") {
                    "gif"
                } else {
                    "png"
                };

                let base_name = raw_filename.split('@').next().unwrap_or(raw_filename);
                let truncated_base = if base_name.len() > 20 {
                    &base_name[..20]
                } else {
                    base_name
                };

                    // date is like 2024-05-18T10:00:00.000Z, sanitized for filename
                    let sanitized_date = date.replace(':', "-");
                let full_filename = format!("img{}_{}.{}", sanitized_date, truncated_base, extension);
                    let file_path = target_dir.join(full_filename);

                    tokio::fs::write(file_path, bytes).await?;
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                }.await;

                if let Err(e) = result {
                    errors.push(format!("Failed to download {}: {}", url, e));
                }
            }

            downloaded_count += 1;
            self.post_to_ui(RedskyUiMsg::DownloadProgress {
                id,
                processed_posts: all_posts.len(),
                total_posts: Some(all_posts.len()),
                downloaded_images: downloaded_count,
                total_images: Some(total_images),
                status: DownloadStatus::Downloading,
            });

            // Rate limiting awareness
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        self.post_to_ui(RedskyUiMsg::DownloadFinished { id, errors });

        Ok(RedskyUiMsg::PostSucceeed()) // dummy successful msg
    }

    async fn post(&self, msg: &String) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("post");
        let _ = self.bsky_agent.create_record(atrium_api::app::bsky::feed::post::RecordData {
            created_at: Datetime::now(),
            embed: None,
            entities: None,
            facets: None,
            labels: None,
            langs: None,
            reply: None,
            tags: None,
            text: msg.to_string(),
        })
        .await?;
        Ok(RedskyUiMsg::PostSucceeed())
    }
}
