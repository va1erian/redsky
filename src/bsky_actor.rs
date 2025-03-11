use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use atrium_api::app::bsky::feed::defs::PostViewEmbedRefs;
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

                Post::new(
                        post_record_data.text,
                        post_el.post.author.handle.to_string(),
                        post_el.post.author.display_name.clone().unwrap_or_default(),
                        post_el.post.indexed_at.as_str().to_string(),
                        post_el.post.like_count.unwrap_or(0),
                        images
                     )
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
        
                    Post::new(
                        post.text,
                        feed_element.post.author.handle.to_string(),
                        feed_element.post.author.display_name.clone()
                        .or(Some("none".to_string())).unwrap(),
                        feed_element.post.indexed_at.as_str().to_string(),
                        feed_element.post.like_count.unwrap_or(0),
                        images
                        )
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