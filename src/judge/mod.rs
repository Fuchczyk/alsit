use crate::ticket::{Language, TicketId};
use std::sync::Arc;
use tokio::sync::Mutex;

mod virtualization;

pub enum JudgeError {
    MismatchedLanguage {
        judge_lang: Language,
        ticket_lang: Language,
    },
    InternalError,
    PoisonedJudge,
}

struct Judge {}

impl Default for Judge {
    fn default() -> Self {
        Self {}
    }
}

impl Judge {
    fn judge(&self, ticket_id: TicketId) {}
}

#[derive(Clone)]
pub struct JudgeDispatcher {
    judges_list: Arc<Vec<Arc<Mutex<Judge>>>>,
}

unsafe impl Sync for JudgeDispatcher {}

impl JudgeDispatcher {
    fn new(number_of_judges: usize) -> JudgeDispatcher {
        let judges_vec = Arc::new(Vec::new());

        for _i in 0..number_of_judges {
            judges_vec.push(Arc::new(Mutex::new(Judge::default())));
        }

        JudgeDispatcher {
            judges_list: judges_vec,
        }
    }

    pub fn queue_judging(&self, ticket_id: TicketId) -> Result<(), JudgeError> {
        use rand::prelude::*;

        let mut rng = thread_rng();

        let judge = self
            .judges_list
            .get(rng.gen::<usize>() % self.judges_list.len())
            .unwrap()
            .clone();

        tokio::task::spawn(async move {
            judge.lock().await.judge(ticket_id);
        });

        Ok(())
    }
}
