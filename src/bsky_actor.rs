use atrium_api::app::bsky::feed::defs::PostViewEmbedRefs;
use atrium_api::app::bsky::feed::post;
use atrium_api::types::string::AtIdentifier;
use atrium_api::types::string::Datetime;
use atrium_api::types::TryFromUnknown;
use atrium_api::types::Union;
use bsky_sdk::BskyAgent;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use crate::app::BskyActorMsg;
use crate::app::Post;
use crate::app::PostImage;
use crate::app::RedskyUiMsg;


pub struct BskyActor {
    tx: Sender<RedskyUiMsg>,
    rx: Receiver<BskyActorMsg>,
    bsky_agent: BskyAgent,
    ctx: egui::Context, //for force repaint
}

impl BskyActor {
    pub fn new(agent: BskyAgent, ctx: egui::Context, rx: Receiver<BskyActorMsg>, tx: Sender<RedskyUiMsg>) -> Self {
        Self {
            tx,
            rx,
            bsky_agent: agent,
            ctx
        }
    }

    pub fn post_to_ui(&self, msg: RedskyUiMsg) {
        self.tx.try_send(msg).unwrap();
        self.ctx.request_repaint();
    }

    pub async fn listen(&mut self) -> bool {
        if let Some(msg) = self.rx.recv().await {
            let result = match msg {
                BskyActorMsg::Login { login, pass } => {
                    self.login(login, pass).await
                }
                BskyActorMsg::Post { msg_body } => {
                    self.post(msg_body).await
                }
                BskyActorMsg::GetTimeline() => {
                    self.get_timeline_posts().await
                }            
                BskyActorMsg::GetUserPosts { username } => {
                    self.get_user_posts(username).await
                }
            };
            if let Ok(reply) = result {
                self.post_to_ui(reply);
            } else if let Err(e) = result {
                self.post_to_ui(RedskyUiMsg::ShowErrorMsg { error: e.to_string() });
            }
            true
        } else {
        false
        }
    }

    async fn get_user_posts(&self, username: String)  -> Result<RedskyUiMsg, Box<dyn std::error::Error>> {
        dbg!("get user posts");
        let at_uri = format!("at://{}", username);
        dbg!(&at_uri);
        let response = self.bsky_agent
        .api
        .app
        .bsky
        .feed
        .get_author_feed(atrium_api::app::bsky::feed::get_author_feed::ParametersData {
            actor: AtIdentifier::Handle(username.parse().unwrap()),
            cursor: None,
            filter: None,
            include_pins: Some(true),
            limit: 20.try_into().ok()
        }.into()).await.unwrap();

        Ok(RedskyUiMsg::ShowUserPostsMsg{
            username,
            posts: response.data.feed.iter().map(|post_el: &atrium_api::types::Object<atrium_api::app::bsky::feed::defs::FeedViewPostData>| {
                let post_record_data = post::RecordData::try_from_unknown(post_el.clone().post.clone().data.record.clone()).unwrap();
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
                    post_el.post.author.display_name.clone()
                        .or(Some("none".to_string())).unwrap(),
                        post_record_data.text,
                        post_el.post.author.handle.to_string(),
                    images
                )
        }).collect()
        })
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
        }.into()).await.unwrap();

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
                        images
                    )
            }).collect()
        })
    }

    async fn login(&self, login: String, pass: String) -> Result<RedskyUiMsg, Box<dyn std::error::Error>> {
        dbg!("loggin in");
        let _ = self.bsky_agent.login(login, pass).await?;
        Ok(RedskyUiMsg::LogInSucceededMsg())
    } 

    async fn post(&self, msg: String) -> Result<RedskyUiMsg, Box<dyn std::error::Error >> {
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
            text: msg,
        })
        .await;
        Ok(RedskyUiMsg::PostSucceeed())
    }
}