use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use atrium_api::app::bsky::actor::profile;
use atrium_api::app::bsky::feed::defs::PostViewEmbedRefs;
use atrium_api::app::bsky::feed::defs::ThreadViewPost;
use atrium_api::app::bsky::feed::defs::ThreadViewPostRepliesItem;
use atrium_api::app::bsky::feed::get_post_thread::OutputThreadRefs;
use atrium_api::app::bsky::feed::post;
use atrium_api::types::string::AtIdentifier;
use atrium_api::types::string::Datetime;
use atrium_api::types::TryFromUnknown;
use atrium_api::types::Union;
use bsky_sdk::BskyAgent;

use crate::app::BskyActorMsg;
use crate::app::Post;
use crate::app::PostImage;
use crate::app::RedskyUiMsg;
use crate::app::StrongRef;
use crate::app::UserProfile;


pub struct BskyActor {
    tx: Sender<RedskyUiMsg>,
    rx: Receiver<BskyActorMsg>,
    bsky_agent: BskyAgent,
    ctx: egui::Context
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
            ctx
        }
    }

    pub fn pump(&mut self) -> bool {
        match self.rx.recv() {
            Ok(msg) => {
                if msg == BskyActorMsg::Close() {
                    println!("bsky actor: closing");
                    false
                } else {
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
            Err(err) => {
                println!("bsky actor: closed receiving chan - {}", err);
                false
            }
        }
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
            BskyActorMsg::GetPostLikers {post_ref } => {
                self.get_post_likers(post_ref).await
            }
            BskyActorMsg::GetTimeline() => {
                self.get_timeline_posts().await
            }            
            BskyActorMsg::GetUserProfile { username } => {
                self.get_user_profile(username).await
            }
            BskyActorMsg::GetUserPosts { username } => {
                self.get_user_posts(username).await
            }
            BskyActorMsg::LoadImage { url } => {
                self.load_image(url).await
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

    async fn get_post_likers(&self, strong_ref: &StrongRef)  -> Result<RedskyUiMsg,  Box<dyn std::error::Error>> {
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

    async fn get_post_thread(&self, strong_ref: &StrongRef) -> Result<RedskyUiMsg,  Box<dyn std::error::Error>> {
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
            OutputThreadRefs::AppBskyFeedDefsThreadViewPost(post_data)) = response.data.thread {
            let post_record_data = post::RecordData::try_from_unknown(post_data.post.record.clone()).unwrap();
            let images : Vec<PostImage> = post_data.post.embed.clone().map(|embed_el: Union<atrium_api::app::bsky::feed::defs::PostViewEmbedRefs>| {
                if let Union::Refs(PostViewEmbedRefs::AppBskyEmbedImagesView(data)) = embed_el {
                    data.images.iter().map(|img| 
                        PostImage::new( img.thumb.to_string(),img.fullsize.to_string(),img.alt.to_string())
                        
                    ).collect()
                } else {
                    vec![]
                }
            }).into_iter().flatten().collect();

            let replies = match &post_data.replies {
                Some(reply_list) => {
                    reply_list.iter().map(|reply| {
                        match reply {
                            Union::Refs(maybe_reply) => {
                                if let ThreadViewPostRepliesItem::ThreadViewPost(view) = maybe_reply {
                                    let images : Vec<PostImage> = view.post.embed.clone().map(|embed_el: Union<atrium_api::app::bsky::feed::defs::PostViewEmbedRefs>| {

                                        if let Union::Refs(PostViewEmbedRefs::AppBskyEmbedImagesView(data)) = embed_el {
                                            data.images.iter().map(|img| 
                                                PostImage::new( img.thumb.to_string(),img.fullsize.to_string(),img.alt.to_string())
                                                
                                            ).collect()
                                        } else {
                                            vec![]
                                        }
                                    }).into_iter().flatten().collect();
                                    let reply_record: post::RecordData = post::RecordData::try_from_unknown(view.post.record.clone()).unwrap();

                                    vec![
                                        Post {
                                            uri: view.post.uri.clone(),
                                            cid: view.post.cid.clone(),
                                            content: reply_record.text.clone(),
                                            author: view.post.author.handle.to_string(),
                                            display_name: view.post.author.display_name.clone().unwrap_or_default(),
                                            avatar_img: view.post.author.avatar.clone().unwrap_or("".to_string()),
                                            date: reply_record.created_at.as_str().to_string(),
                                            like_count: view.post.like_count.unwrap_or(0),
                                            repost_count: view.post.repost_count.unwrap_or(0),
                                            embeds: images
                                        }
                                    ]
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
                post: Post {
                    uri: post_data.post.uri.clone(),
                    cid: post_data.post.cid.clone(),
                    content: post_record_data.text.clone(),
                    author: post_data.post.author.handle.to_string(),
                    display_name: post_data.post.author.display_name.clone().unwrap_or_default(),
                    avatar_img: post_data.post.author.avatar.clone().unwrap_or("".to_string()),
                    date: post_record_data.created_at.as_str().to_string(),
                    like_count: post_data.post.like_count.unwrap_or(0),
                    repost_count: post_data.post.repost_count.unwrap_or(0),
                    embeds: images
                }, 
                replies
            })
        } else {
        Ok(RedskyUiMsg::PostSucceeed())
        }
    }

    async fn load_image(&self, url: &String) -> Result<RedskyUiMsg,  Box<dyn std::error::Error>> {
        let resp = reqwest::get(url)
        .await?;
        let bytes = resp.bytes().await?;
        Ok(RedskyUiMsg::NotifyImageLoaded { url: url.to_string(), data: bytes.to_vec().into() })
    }

    async fn get_user_posts(&self, username: &String)  -> Result<RedskyUiMsg, Box<dyn std::error::Error>> {
        dbg!("get user posts");
        let at_uri = format!("at://{}", username);
        dbg!(&at_uri);
        let response = self.bsky_agent
        .api
        .app
        .bsky
        .feed
        .get_author_feed(atrium_api::app::bsky::feed::get_author_feed::ParametersData {
            actor: AtIdentifier::Handle(username.parse()?),
            cursor: None,
            filter: None,
            include_pins: Some(true),
            limit: 20.try_into().ok()
        }.into()).await?;

        Ok(RedskyUiMsg::ShowUserPostsMsg{
            username: username.to_string(),
            posts: response.data.feed.iter().map(|post_el: &atrium_api::types::Object<atrium_api::app::bsky::feed::defs::FeedViewPostData>| {
                let post_record_data = post::RecordData::try_from_unknown(post_el.post.data.record.clone()).unwrap();
                let images : Vec<PostImage> = post_el.post.embed.clone().map(|embed_el: Union<atrium_api::app::bsky::feed::defs::PostViewEmbedRefs>| {
                    if let Union::Refs(PostViewEmbedRefs::AppBskyEmbedImagesView(data)) = embed_el {
                        data.images.iter().map(|img| 
                            PostImage::new( img.thumb.to_string(),img.fullsize.to_string(),img.alt.to_string())
                            
                        ).collect()
                    } else {
                        vec![]
                    }
                }).into_iter().flatten().collect();

                Post {
                    uri: post_el.post.uri.clone(),
                    cid: post_el.post.cid.clone(),
                    content: post_record_data.text.clone(),
                    author: post_el.post.author.handle.to_string(),
                    display_name: post_el.post.author.display_name.clone().unwrap_or_default(),
                    avatar_img: post_el.post.author.avatar.clone().unwrap_or("".to_string()),
                    date: post_record_data.created_at.as_str().to_string(),
                    like_count: post_el.post.like_count.unwrap_or(0),
                    repost_count: post_el.post.repost_count.unwrap_or(0),
                    embeds: images
                }
        }).collect()
        })
    }

    async fn get_user_profile(&self, username: &String) -> Result<RedskyUiMsg, Box<dyn std::error::Error>> {
        dbg!("get user profile", &username);

        let profile = self.bsky_agent
        .api
        .app
        .bsky
        .actor
        .get_profile(atrium_api::app::bsky::actor::get_profile::ParametersData{
            actor: AtIdentifier::Handle(username.parse()?)
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

    async fn get_timeline_posts(&self) -> Result<RedskyUiMsg, Box<dyn std::error::Error>> {
        dbg!("get tl");

        let posts = self.bsky_agent
        .api
        .app
        .bsky
        .feed
        .get_timeline( atrium_api::app::bsky::feed::get_timeline::ParametersData{
            algorithm: None,
            cursor: None,
            limit: 30.try_into().ok()
        }.into()).await?;

        Ok(RedskyUiMsg::RefreshTimelineMsg 
            { 
                posts: posts.data.feed.iter().map(|feed_element| {
                    let post = post::RecordData::try_from_unknown(feed_element.data.post.data.record.clone()).unwrap();
                    let images : Vec<PostImage> = feed_element.post.embed.clone().map(|embed_el: Union<atrium_api::app::bsky::feed::defs::PostViewEmbedRefs>| {
                        if let Union::Refs(PostViewEmbedRefs::AppBskyEmbedImagesView(data)) = embed_el {
                            data.images.iter().map(|img| 
                                PostImage::new( img.thumb.to_string(),img.fullsize.to_string(),img.alt.to_string())
                                
                            ).collect()
                        } else {
                            vec![]
                        }
                    }).into_iter().flatten().collect();
        
                    Post {
                        uri: feed_element.post.uri.clone(),
                        cid: feed_element.post.cid.clone(),
                        content: post.text.clone(),
                        author: feed_element.post.author.handle.to_string(),
                        display_name: feed_element.post.author.display_name.clone().unwrap_or_default(),
                        avatar_img: feed_element.post.author.avatar.clone().unwrap_or("".to_string()),
                        date: post.created_at.as_str().to_string(),
                        like_count: feed_element.post.like_count.unwrap_or(0),
                        repost_count: feed_element.post.repost_count.unwrap_or(0),
                        embeds: images
                    }
            }).collect()
        })
    }

    async fn login(&self, login: &String, pass: &String) -> Result<RedskyUiMsg, Box<dyn std::error::Error>> {
        dbg!("loggin in");
        let _ = self.bsky_agent.login(login, pass).await?;
        Ok(RedskyUiMsg::LogInSucceededMsg())
    } 

    async fn post(&self, msg: &String) -> Result<RedskyUiMsg, Box<dyn std::error::Error >> {
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