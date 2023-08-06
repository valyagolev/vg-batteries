use std::any;

use teloxide::{
    payloads::{self, EditMessageReplyMarkupSetters, SendMessageSetters},
    requests::{JsonRequest, Requester},
    types::{ChatId, InlineKeyboardMarkup, MessageId, Recipient, ReplyMarkup},
    Bot,
};

pub struct UpdateableMessage {
    pub chat_id: ChatId,
    pub message_id: MessageId,
    pub text: String,
    pub reply_markup: Option<InlineKeyboardMarkup>,
}

impl UpdateableMessage {
    pub async fn new(req: JsonRequest<payloads::SendMessage>) -> anyhow::Result<Self> {
        let Recipient::Id(chat_id) = req.chat_id else {
            return Err(anyhow::anyhow!("chat_id is not a ChatId"));
        };

        let reply_markup = match &req.reply_markup {
            None => None,
            Some(ReplyMarkup::InlineKeyboard(reply_markup)) => Some(reply_markup),
            _ => {
                return Err(anyhow::anyhow!(
                    "reply_markup is not an InlineKeyboardMarkup"
                ))
            }
        };

        Ok(Self {
            chat_id,
            text: req.text.to_owned(),
            reply_markup: reply_markup.cloned(),
            message_id: req.await?.id,
        })
    }

    pub async fn resend(&mut self, bot: &Bot) -> anyhow::Result<()> {
        let _ = bot.delete_message(self.chat_id, self.message_id);

        let mut crt = bot.send_message(self.chat_id, &self.text);

        if let Some(reply_markup) = self.reply_markup.to_owned() {
            crt = crt.reply_markup(reply_markup);
        }

        self.message_id = crt.await?.id;

        Ok(())
    }

    pub async fn update_text(&mut self, bot: &Bot, text: &str) -> anyhow::Result<()> {
        if self.text != text {
            self.text = text.to_owned();

            let res = bot
                .edit_message_text(self.chat_id, self.message_id, text)
                .await;

            if let Err(_) = res {
                self.resend(bot).await?;
            }
        }

        Ok(())
    }

    pub async fn update_markup(
        &mut self,
        bot: &Bot,
        reply_markup: Option<InlineKeyboardMarkup>,
    ) -> anyhow::Result<()> {
        if self.reply_markup != reply_markup {
            let mut req = bot.edit_message_reply_markup(self.chat_id, self.message_id);

            if let Some(reply_markup) = &reply_markup {
                req = req.reply_markup(reply_markup.clone());
            }

            let res = req.await;

            if let Err(_) = res {
                self.resend(bot).await?;
            }

            self.reply_markup = reply_markup;
        }

        Ok(())
    }

    pub async fn delete(self, bot: &Bot) -> anyhow::Result<()> {
        bot.delete_message(self.chat_id, self.message_id).await?;

        Ok(())
    }
}
