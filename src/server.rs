use crate::pb::abi::*;
use crate::pb::{abi::chat_server::Chat, chat_server::ChatServer};
use futures::Stream;
use std::pin::Pin;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::transport::Server;
use tonic::{Extensions, Request, Response, Status};
use tracing::{info, warn};

const MAX_MESSAGES: usize = 1024;

pub struct ChatService {
    tx: broadcast::Sender<ChatMessage>,
}

pub type ChatResult<T> = Result<Response<T>, Status>;

#[tonic::async_trait]
impl Chat for ChatService {
    async fn login(&self, request: Request<LoginRequest>) -> ChatResult<Token> {
        let info = request.into_inner();
        info!("login: {info:?}");
        let token = info.into_token();
        Ok(Response::new(token))
    }
    /// to send message to a chatting room
    async fn send_message(
        &self,
        request: Request<NewChatMessage>,
    ) -> ChatResult<SendMessageResponse> {
        let sender = get_username(request.extensions())?;
        let info = request.into_inner();
        info!("send_message: {info:?}");
        let msg = info.into_chat_message(sender);
        // to publish message to everyone who is interested in it
        self.tx.send(msg).unwrap();
        Ok(Response::new(SendMessageResponse {}))
    }
    type GetMesssagesStream = Pin<Box<dyn Stream<Item = Result<ChatMessage, Status>> + Send>>;
    /// to subscribe and get all messages
    async fn get_messsages(
        &self,
        request: Request<GetMessageRequest>,
    ) -> Result<Response<Self::GetMesssagesStream>, Status> {
        let info = request.into_inner();
        info!("subscribe to messages: {info:?}");
        let mut rx = self.tx.subscribe();
        let (sender, receiver) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            // TODO: to filter out messages that I am not interested in
            while let Ok(msg) = rx.recv().await {
                if let Err(_) = sender.send(Ok(msg)) {
                    warn!("Failed to send. The sender might be closed.")
                }
            }
        });
        let stream = UnboundedReceiverStream::new(receiver);
        Ok(Response::new(Box::pin(stream)))
    }
}

impl Default for ChatService {
    fn default() -> Self {
        let (tx, _rx) = broadcast::channel(MAX_MESSAGES);
        Self { tx }
    }
}

pub async fn start() {
    let svc = ChatServer::with_interceptor(ChatService::default(), check_auth);
    let addr = "0.0.0.0:8080".parse().unwrap();
    info!("listening on http://{}", addr);
    Server::builder()
        .add_service(svc)
        .serve(addr)
        .await
        .unwrap();
}

fn check_auth(mut req: Request<()>) -> Result<Request<()>, Status> {
    let token = match req.metadata().get("authorization") {
        Some(v) => {
            let data = v
                .to_str()
                .map_err(|_| Status::new(tonic::Code::Unauthenticated, "Invalid token format"))?;
            Token::new(data.strip_prefix("Bearer ").unwrap())
        }
        None => Token::default(),
    };
    req.extensions_mut().insert(token);
    Ok(req)
}

fn get_username(ext: &Extensions) -> Result<String, Status> {
    let token = ext
        .get::<Token>()
        .ok_or(Status::unauthenticated("No token"))?;
    if token.is_valid() {
        Ok(token.into_username())
    } else {
        Err(Status::unauthenticated("Invalid token"))
    }
}
