use crate::pb::{
    chat_client::ChatClient, ChatMessage, GetMessageRequest, LoginRequest, NewChatMessage, Token,
};
use anyhow::Result;
use arc_swap::ArcSwap;
use dashmap::DashMap;
use std::{ops::Deref, sync::Arc};
use tonic::{
    codegen::InterceptedService, metadata::AsciiMetadataValue, service::Interceptor,
    transport::Channel, Request, Status,
};
use tracing::info;

lazy_static! {
    static ref TOKEN: ArcSwap<Token> = ArcSwap::from(Arc::new(Token {
        data: "".to_owned(),
    }));
}

#[derive(Clone, Default)]
struct Rooms(Arc<DashMap<String, Vec<ChatMessage>>>);

impl Rooms {
    fn insert_message(&self, msg: ChatMessage) {
        let room = msg.room.to_owned();
        let mut room_messages = self.entry(room).or_insert_with(Vec::new);
        room_messages.push(msg);
    }
}

pub struct Client {
    username: String,
    conn: ChatClient<InterceptedService<Channel, AuthInteceptor>>,
    rooms: Rooms,
}

impl Deref for Rooms {
    type Target = DashMap<String, Vec<ChatMessage>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Client {
    pub async fn new(username: impl Into<String>) -> Self {
        let channel = Channel::from_static("http://127.0.0.1:8080")
            .connect()
            .await
            .unwrap();
        let conn = ChatClient::with_interceptor(channel, AuthInteceptor);
        Self {
            username: username.into(),
            conn,
            rooms: Default::default(),
        }
    }
    pub async fn login(&mut self) -> Result<()> {
        let login = LoginRequest::new(&self.username, "password");
        let token = self.conn.login(login).await?.into_inner();
        TOKEN.store(Arc::new(token));
        Ok(())
    }
    pub async fn send_message(
        &mut self,
        room: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<(), Status> {
        let msg = NewChatMessage::new(room.into(), content.into());
        self.conn.send_message(msg).await?;
        Ok(())
    }
    pub async fn get_messages(&mut self) -> Result<()> {
        let req = GetMessageRequest::new();
        let mut stream = self.conn.get_messsages(req).await?.into_inner();
        let rooms = self.rooms.clone();
        tokio::spawn(async move {
            while let Ok(Some(msg)) = stream.message().await {
                info!("got message: {:?}", msg);
                rooms.insert_message(msg);
            }
        });
        Ok(())
    }
}

struct AuthInteceptor;

impl Interceptor for AuthInteceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        let token = TOKEN.load();
        if token.is_valid() {
            let value = AsciiMetadataValue::try_from(&format!("Bearer {}", token.data)).unwrap();
            req.metadata_mut().insert("authorization", value);
        }
        Ok(req)
    }
}
