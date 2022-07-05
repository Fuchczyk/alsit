use crate::ticket::{Language, TicketId};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tokio::task;

mod virtualization;

pub use virtualization::testing;

pub enum JudgeError {
    MismatchedLanguage {
        judge_lang: Language,
        ticket_lang: Language,
    },
    InternalError,
    PoisonedJudge,
}

struct Judge {
    lang: Language, // Some docker field
}

impl Judge {
    fn judge(&self, ticket_id: TicketId) {}
}

#[derive(Clone)]
pub struct JudgeDispatcher {
    judges_list: HashMap<Language, Vec<Arc<Mutex<Judge>>>>,
}

unsafe impl Sync for JudgeDispatcher {}

impl JudgeDispatcher {
    async fn new(judges: Vec<Arc<Mutex<Judge>>>) -> Result<JudgeDispatcher, JudgeError> {
        let mut map = HashMap::new();

        for judge in judges {
            let judge_inside = judge.lock().await;

            let language = judge_inside.lang.clone();

            match map.get_mut(&language) {
                None => {
                    let mut vector = Vec::new();
                    vector.push(judge.clone());

                    map.insert(language, vector);
                }
                Some(vector) => {
                    vector.push(judge.clone());
                }
            }
        }

        Ok(JudgeDispatcher { judges_list: map })
    }

    fn queue_judging(&self, ticket: &super::ticket::Ticket) -> Result<(), JudgeError> {
        use rand::prelude::*;

        let judges = match self.judges_list.get(&ticket.language()) {
            Some(list) => list,
            None => {
                error!(
                    "No judges were found for language: {:?}.",
                    ticket.language()
                );
                return Err(JudgeError::InternalError);
            }
        };

        let mut rng = thread_rng();

        let judge = judges
            .get(rng.gen::<usize>() % judges.len())
            .unwrap()
            .clone();

        let ticket_id = ticket.id();
        tokio::task::spawn(async move {
            judge.lock().await.judge(ticket_id);
        });

        Ok(())
    }
}
