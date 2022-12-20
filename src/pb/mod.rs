pub mod abi;
pub use abi::*;

impl LoginRequest {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
    pub fn into_token(&self) -> Token {
        Token::new(self.username.to_owned())
    }
}

impl Token {
    pub fn new(data: impl Into<String>) -> Self {
        Self { data: data.into() }
    }
    // TODO: to apply JWT
    pub fn into_username(&self) -> String {
        self.data.to_owned()
    }
    pub fn is_valid(&self) -> bool {
        self.data.len() > 0
    }
}

impl NewChatMessage {
    pub fn new(room: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            room: room.into(),
            content: content.into(),
        }
    }
    pub fn into_chat_message(self, sender: impl Into<String>) -> ChatMessage {
        ChatMessage::new(sender, self.room, self.content)
    }
}

impl ChatMessage {
    pub fn new(
        sender: impl Into<String>,
        room: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            sender: sender.into(),
            room: room.into(),
            content: content.into(),
            timestamp,
        }
    }
}

impl GetMessageRequest {
    pub fn new() -> Self {
        Self {}
    }
}
