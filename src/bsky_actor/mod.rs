use crate::app::BskyActorMsg;
#[cfg(not(feature = "mock-api"))]
use crate::app::DownloadStatus;
use crate::app::Post;
use crate::app::PostImage;
use crate::app::RedskyUiMsg;
use crate::app::StrongRef;
use crate::app::UserProfile;
use atrium_api::app::bsky::bookmark::defs::BookmarkViewData;
use atrium_api::app::bsky::bookmark::defs::BookmarkViewItemRefs;
use atrium_api::app::bsky::embed::record::ViewRecordRefs;
use atrium_api::app::bsky::feed::defs::PostViewData;
use atrium_api::app::bsky::feed::defs::PostViewEmbedRefs;
#[cfg(not(feature = "mock-api"))]
use atrium_api::app::bsky::feed::defs::ThreadViewPostRepliesItem;
#[cfg(not(feature = "mock-api"))]
use atrium_api::app::bsky::feed::get_post_thread::OutputThreadRefs;
use atrium_api::app::bsky::feed::post;
#[cfg(not(feature = "mock-api"))]
use atrium_api::types::string::{AtIdentifier, Datetime, RecordKey};
use atrium_api::types::string::Cid;
use atrium_api::types::Object;
use atrium_api::types::TryFromUnknown;
use atrium_api::types::Union;
use bsky_sdk::BskyAgent;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
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
    #[cfg_attr(feature = "mock-api", allow(dead_code))]
    bsky_agent: BskyAgent,
    ctx: egui::Context, //for force repaint
}
impl BskyActor {
    pub fn new(
        bsky_agent: BskyAgent,
        ctx: egui::Context,
        rx: Receiver<BskyActorMsg>,
        tx: Sender<RedskyUiMsg>,
    ) -> Self {
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
            Ok(msg) => match msg {
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
                        ctx: self.ctx.clone(),
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
                        ctx: self.ctx.clone(),
                    };
                    tokio::spawn(job.perform());
                    true
                }
            },
            Err(err) => {
                println!("bsky actor: closed receiving chan - {}", err);
                false
            }
        }
    }
}
#[cfg_attr(feature = "mock-api", allow(dead_code))]
fn extract_quote_reply(post_view: &Object<PostViewData>) -> Option<Post> {
    if let Some(Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordView(embedded_record))) =
        &post_view.embed
    {
        if let Union::Refs(ViewRecordRefs::ViewRecord(view_record)) = &embedded_record.record {
            let quote_post_data =
                post::RecordData::try_from_unknown(view_record.value.clone()).ok()?;
            Some(Post {
                uri: view_record.uri.clone(),
                cid: view_record.cid.clone(),
                content: quote_post_data.text,
                author: view_record.author.handle.to_string(),
                display_name: view_record
                    .author
                    .display_name
                    .clone()
                    .unwrap_or("".to_string()),
                avatar_img: view_record.author.avatar.clone().unwrap_or("".to_string()),
                date: quote_post_data.created_at.as_str().to_string(),
                like_count: view_record.like_count.unwrap_or(0),
                repost_count: view_record.repost_count.unwrap_or(0),
                embeds: vec![],
                quoted_post: None,
                is_reply: quote_post_data.reply.is_some(),
                viewer_like: None,
                viewer_repost: None,
                thread_root: None,
            })
        } else {
            None
        }
    } else {
        None
    }
}
#[cfg_attr(feature = "mock-api", allow(dead_code))]
fn extract_images(post_view: &Object<PostViewData>) -> Vec<PostImage> {
    post_view
        .embed
        .clone()
        .map(
            |embed_el: Union<atrium_api::app::bsky::feed::defs::PostViewEmbedRefs>| {
                if let Union::Refs(PostViewEmbedRefs::AppBskyEmbedImagesView(data)) = embed_el {
                    data.images
                        .iter()
                        .map(|img| {
                            PostImage::new(
                                img.thumb.to_string(),
                                img.fullsize.to_string(),
                                img.alt.to_string(),
                            )
                        })
                        .collect()
                } else {
                    vec![]
                }
            },
        )
        .into_iter()
        .flatten()
        .collect()
}
#[cfg_attr(feature = "mock-api", allow(dead_code))]
fn extract_post(post_view: &Object<PostViewData>) -> Option<Post> {
    let post_record_data =
        post::RecordData::try_from_unknown(post_view.data.record.clone()).ok()?;
    let images: Vec<PostImage> = extract_images(post_view);
    let quoted_post: Option<Post> = extract_quote_reply(post_view);
    Some(Post {
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
        quoted_post: quoted_post.map(Box::new),
        is_reply: post_record_data.reply.is_some(),
        viewer_like: post_view.viewer.as_ref().and_then(|v| v.like.clone()),
        viewer_repost: post_view.viewer.as_ref().and_then(|v| v.repost.clone()),
        thread_root: post_record_data.reply.map(|reply| StrongRef {
                uri: reply.root.uri.clone(),
                cid: reply.root.cid.clone(),
            }),
    })
}
#[cfg_attr(feature = "mock-api", allow(dead_code))]
fn extract_post_from_bookmark(bookmark: &Object<BookmarkViewData>) -> Option<Post> {
    match &bookmark.item {
        Union::Refs(BookmarkViewItemRefs::AppBskyFeedDefsPostView(post)) => {
            extract_post(post.as_ref())
        }
        // Return None for BlockedPost, NotFoundPost, or other union variants
        _ => None,
    }
}
impl BskyJob {
    pub async fn perform(self) -> () {
        let result = match &self.job {
            BskyActorMsg::Login { login, pass } => self.login(login, pass).await,
            BskyActorMsg::Post { msg_body, image_paths, reply_to } => self.post(msg_body, image_paths, reply_to).await,
            BskyActorMsg::GetPostAndReplies { post_ref } => self.get_post_thread(post_ref).await,
            BskyActorMsg::GetPostLikers { post_ref, cursor } => self.get_post_likers(post_ref, cursor).await,
            BskyActorMsg::GetPostRepostedBy { post_ref, cursor } => {
                self.get_post_reposted_by(post_ref, cursor).await
            }
            BskyActorMsg::Like { post_ref } => self.like(post_ref.clone()).await,
            BskyActorMsg::Unlike {
                post_uri,
                like_record_uri,
            } => self.unlike(post_uri.clone(), like_record_uri.clone()).await,
            BskyActorMsg::DeletePost { post_uri, post_cid } => self.delete_post(post_uri.clone(), post_cid.clone()).await,
            BskyActorMsg::Repost { post_ref } => self.repost(post_ref.clone()).await,
            BskyActorMsg::Unrepost {
                post_uri,
                repost_record_uri,
            } => {
                self.unrepost(post_uri.clone(), repost_record_uri.clone())
                    .await
            }
            BskyActorMsg::GetTimeline { cursor } => self.get_timeline_posts(cursor).await,
            BskyActorMsg::GetBookmarks { cursor } => self.get_bookmarks(cursor).await,
            BskyActorMsg::GetUserProfile { username } => self.get_user_profile(username).await,
            BskyActorMsg::GetUserPosts { username, cursor } => {
                self.get_user_posts(username, cursor).await
            }
            BskyActorMsg::GetUserLikes { username, cursor } => {
                self.get_user_likes(username, cursor).await
            }
            BskyActorMsg::SearchActors { query } => self.search_actors(query).await,
            BskyActorMsg::SearchPosts { query, cursor } => self.search_posts(query, cursor).await,
            BskyActorMsg::LoadImage { url } => self.load_image(url).await,
            BskyActorMsg::StartImageDownload { id, username, path } => {
                self.download_all_images(*id, username, path).await
            }
            BskyActorMsg::CancelImageDownload { .. } => {
                Ok(RedskyUiMsg::LogInSucceededMsg()) // dummy
            }
            BskyActorMsg::GetUnreadCount() => self.get_unread_count().await,
            BskyActorMsg::GetNotifications { cursor } => self.get_notifications(cursor).await,
            BskyActorMsg::GetRawPost { post_uri } => self.get_raw_post(post_uri).await,
            BskyActorMsg::Close() => {
                panic!("unexpected message");
            }
        };
        if let Ok(reply) = result {
            self.post_to_ui(reply);
        } else if let Err(e) = result {
            self.post_to_ui(RedskyUiMsg::ShowErrorMsg {
                error: e.to_string(),
            });
        }
    }
    pub fn post_to_ui(&self, msg: RedskyUiMsg) {
        self.tx.send(msg).unwrap();
        self.ctx.request_repaint();
    }
}

#[cfg(not(feature = "mock-api"))]
include!("actor_methods.rs");
#[cfg(feature = "mock-api")]
include!("actor_methods_mock.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    #[tokio::test]
    async fn test_pump_channel_closed() {
        let (msg_tx, msg_rx) = channel();
        let (ui_tx, _ui_rx) = channel();
        let agent = BskyAgent::builder().build().await.unwrap();
        let mut actor = BskyActor::new(agent, egui::Context::default(), msg_rx, ui_tx);

        // Drop the sender to close the channel
        drop(msg_tx);

        // pump() should return false when the channel is closed
        assert!(!actor.pump());
    }
}
