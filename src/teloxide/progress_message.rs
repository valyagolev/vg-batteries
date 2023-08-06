use std::{sync::Arc, time::Duration};

use teloxide::{
    requests::{JsonRequest, Request},
    Bot,
};
use tokio::{sync::Mutex, time::Instant};

const THROTTLE: Duration = Duration::from_millis(200);

pub struct ProgressMessage {
    pub bot: Bot,
    pub edit_payload: teloxide::payloads::EditMessageText,
    pub schedule: Arc<Mutex<ScheduleStatus>>,
}

pub enum ScheduleStatus {
    LastUpdate(Instant),
    NeedsUpdate(Bot, teloxide::payloads::EditMessageText),
}

async fn updater_future(wait_for: Instant, schedule: Arc<Mutex<ScheduleStatus>>) {
    if wait_for > Instant::now() + Duration::from_millis(50) {
        tokio::time::sleep_until(wait_for).await;
    }

    let mut schedule = schedule.lock().await;

    match &mut *schedule {
        ScheduleStatus::LastUpdate(_) => {
        }
        ScheduleStatus::NeedsUpdate(bot, update) => {
            JsonRequest::new(bot.clone(), update.clone())
                .send()
                .await
                .unwrap();
            *schedule = ScheduleStatus::LastUpdate(Instant::now());
        }
    }
}

impl ProgressMessage {
    pub async fn new(bot: Bot, msg: teloxide::payloads::SendMessage) -> anyhow::Result<Self> {
        let result = JsonRequest::new(bot.clone(), msg.clone()).send().await?;

        let message_id = result.id;
        let edit_payload = teloxide::payloads::EditMessageText {
            chat_id: msg.chat_id,
            message_id,
            text: msg.text,
            parse_mode: msg.parse_mode,
            entities: msg.entities,
            disable_web_page_preview: msg.disable_web_page_preview,
            reply_markup: None, //msg.reply_markup, // todo?
        };

        Ok(Self {
            bot,
            edit_payload,
            schedule: Arc::new(Mutex::new(ScheduleStatus::LastUpdate(Instant::now()))),
            // needs_update: false,
            // update_scheduled: false,
        })
    }

    pub async fn update(&mut self, new_text: &str) {
        if self.edit_payload.text != new_text {
            self.edit_payload.text = new_text.to_owned();

            let mut schedule = self.schedule.lock().await;

            if let ScheduleStatus::LastUpdate(last) = &*schedule {
                let wait_for = *last + THROTTLE;
                let sched = self.schedule.clone();
                tokio::spawn(async move { updater_future(wait_for, sched).await });
            }

            *schedule = ScheduleStatus::NeedsUpdate(self.bot.clone(), self.edit_payload.clone());
        }
    }

    pub async fn append(&mut self, new_text: &str) {
        let mut text = self.edit_payload.text.clone();
        text.push_str(new_text);
        self.update(&text).await;
    }

    pub async fn append_md(&mut self, new_text: &str) {
        let mut text = self.edit_payload.text.clone();
        text.push_str(&teloxide::utils::markdown::escape(new_text));
        self.update(&text).await;
    }
}
