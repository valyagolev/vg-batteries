use std::future::IntoFuture;

use teloxide::{requests::Requester, types::ChatId, Bot};
use tokio::task::JoinHandle;

pub struct Typer {
    handle: JoinHandle<()>,
}

impl Typer {
    pub fn new(bot: &Bot, chat_id: ChatId) -> Self {
        let bot = bot.clone();

        Self {
            handle: tokio::spawn(async move {
                loop {
                    if let Err(e) = bot
                        .send_chat_action(chat_id, teloxide::types::ChatAction::Typing)
                        .await
                    {
                        println!("error sending chat action: {}", e);
                        return;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(4500)).await;
                }
            }),
        }
    }
    pub async fn resolve<O, F: IntoFuture<Output = O>>(self, fut: F) -> O {
        drop(self);

        fut.into_future().await
    }
}

impl Drop for Typer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}
