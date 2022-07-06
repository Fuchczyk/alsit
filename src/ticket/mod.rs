use std::{fmt::Display, str::FromStr};

use crate::account::UserId;
use actix_web::{web, HttpResponse, Result};
use deadpool_postgres::{Object, Pool};
use serde::Deserialize;

pub type TicketId = i64;
pub type ExerciseId = i64;

#[derive(Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub enum Language {
    C = 0,
    Cpp,
    Rust,
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string_description = match &self {
            &Language::C => "C",
            &Language::Cpp => "Cpp",
            &Language::Rust => "Rust",
        };

        write!(f, "{string_description}")
    }
}

impl Language {
    pub fn extension(&self) -> &'static str {
        match &self {
            &Language::C => ".c",
            &Language::Cpp => ".cpp",
            &Language::Rust => ".rs",
        }
    }
}

pub enum TicketError {
    DatabaseError,
    WrongTicketId,
}

// TODO: Make macro to automatic conversion.
impl FromStr for Language {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "C" => Ok(Language::C),
            "Cpp" => Ok(Language::Cpp),
            "Rust" => Ok(Language::Rust),
            _ => Err(()),
        }
    }
}

pub struct Ticket {
    user_id: UserId,

    language: Language,
    content: String,
    exercise_id: ExerciseId,

    judged: bool,
    ticket_id: TicketId,

    results_id: Option<i64>,
}

#[derive(Deserialize)]
struct TicketForm {
    language: Language,
    exercise_id: ExerciseId,

    content: String,
}

async fn generate_id(client: &Object) -> Result<TicketId, ()> {
    use rand::prelude::*;
    let mut rng = thread_rng();
    let check_stmt = include_str!("query_ticket.sql");

    loop {
        let possible_id: TicketId = rng.gen();

        let query_result = client.query_opt(check_stmt, &[&possible_id]).await;

        match query_result {
            Err(_) => {
                return Err(());
            }
            Ok(result) => {
                if result.is_none() {
                    return Ok(possible_id);
                }
            }
        }
    }
}

pub async fn get_content(
    ticket_id: TicketId,
    db: &Pool,
) -> Result<(String, Language), TicketError> {
    todo!()
}

pub async fn set_judged(ticket_id: TicketId, db: &Pool) -> Result<(), TicketError> {
    todo!()
}

impl Ticket {
    async fn create(form: TicketForm, user_id: UserId, ticket_id: TicketId) -> Ticket {
        Ticket {
            user_id,
            language: form.language,
            content: form.content,
            exercise_id: form.exercise_id,
            judged: false,
            ticket_id,
            results_id: None,
        }
    }

    pub fn language(&self) -> Language {
        self.language.clone()
    }

    pub fn id(&self) -> TicketId {
        self.ticket_id
    }
}

// TODO: Make field in ticket table boolean not ticketstatus!
async fn insert_ticket(ticket: Ticket, client: &Object) -> HttpResponse {
    let insert_stmt = include_str!("insert_ticket.sql");

    let query_result = client
        .query(
            insert_stmt,
            &[
                &ticket.ticket_id,
                &ticket.user_id,
                &ticket.language.to_string(),
                &ticket.content,
                &ticket.exercise_id,
                &ticket.judged,
            ],
        )
        .await;

    if let Err(error) = query_result {
        error!("Error occured while inserting ticket. {:?}", error);
        HttpResponse::ServiceUnavailable().finish()
    } else {
        HttpResponse::Created().finish()
    }
}

// TODO: UserID
async fn create_ticket(form: web::Json<TicketForm>, db: web::Data<Pool>) -> HttpResponse {
    let client = match db.get().await {
        Ok(client) => client,
        Err(_) => {
            return HttpResponse::ServiceUnavailable().finish();
        }
    };

    let ticket_id = match generate_id(&client).await {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::ServiceUnavailable().finish();
        }
    };

    let user_id = 0;

    let ticket = Ticket::create(form.into_inner(), user_id, ticket_id).await;

    insert_ticket(ticket, &client).await
}

pub fn ticket_handler(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::post().to(create_ticket));
}
