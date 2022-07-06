use actix_web::{cookie::CookieBuilder, web, HttpResponse};

use deadpool_postgres::{Object, Pool};
use serde::{Deserialize, Serialize};

use crate::crypto::{Encryptor, Hasher};

pub type UserId = i64;

// FIXME: Security error - username needs to be validated before sql query.

#[derive(Deserialize, Serialize)]
pub struct SinupForm {
    username: String,
    password: String,
    email: String,
}

struct SignupProcessed {
    id: i64,
    username: String,
    password_hash: Vec<u8>,
    user_salt: Vec<u8>,
    email: Vec<u8>,
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn generate_user_id(client: &Object) -> Result<i64, ()> {
    use rand::prelude::*;
    let mut rng = thread_rng();
    let check_stmt = include_str!("id_query.sql");

    loop {
        let possible_id: i64 = rng.gen();

        let query_result = client.query_opt(check_stmt, &[&possible_id]).await;

        match query_result {
            Err(error) => {
                error!("Error occured while generated user id. {:?}", error);
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

async fn insert_into_database(processed_data: SignupProcessed, client: Object) -> HttpResponse {
    let insert_stmt = include_str!("insert_user.sql");

    let query_result = client
        .query(
            insert_stmt,
            &[
                &processed_data.id,
                &processed_data.username,
                &processed_data.password_hash,
                &processed_data.user_salt,
                &processed_data.email,
            ],
        )
        .await;

    error!("JESTEM TU3");

    match query_result {
        Ok(_) => HttpResponse::Created().finish(),
        Err(error) => {
            let decoded_error = error.code().unwrap();
            error!(
                "Error occured while querying the database. ERROR = {:?}",
                error
            );

            if *decoded_error == tokio_postgres::error::SqlState::UNIQUE_VIOLATION {
                HttpResponse::Conflict().body("Username is not avilable.")
            } else {
                HttpResponse::ServiceUnavailable().finish()
            }
        }
    }
}

pub async fn create_account(
    db: web::Data<Pool>,
    user_data: web::Json<SinupForm>,
    encryptor: web::Data<Encryptor>,
    hasher: web::Data<Hasher<'_>>,
) -> HttpResponse {
    if user_data.username.len() > crate::MAX_USERNAME_LENGTH {
        return HttpResponse::UnprocessableEntity().body("Username is too long");
    }

    let mut password_hash = [0u8; crate::HASH_LENGTH_BYTES];
    let user_salt = crate::crypto::generate_salt();

    let nonce: Vec<u8> = user_salt
        .iter()
        .take(crate::ENCRYPT_NONCE_LEN)
        .map(|x| x.to_owned())
        .collect();

    let email_enc = match encryptor.encrypt(user_data.email.as_bytes(), &nonce) {
        Ok(v) => v,
        Err(e) => {
            error!("Error occured while encrypting email. ERROR = {e:?}");

            return HttpResponse::ServiceUnavailable().finish();
        }
    };

    if let Err(e) = hasher.hash_password(&user_data.password, &user_salt, &mut password_hash) {
        error!("Error occured while hashing password. ERROR = {e:?}");

        return HttpResponse::ServiceUnavailable().finish();
    }

    error!("JESTEM TUTAJ1");

    let db = match db.get().await {
        Ok(database) => database,
        Err(_) => {
            return HttpResponse::ServiceUnavailable().finish();
        }
    };

    error!("JESTEM TUTAJ2");

    let user_id = match generate_user_id(&db).await {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::ServiceUnavailable().finish();
        }
    };

    let user_preprocessed = SignupProcessed {
        id: user_id,
        username: user_data.username.to_string(),
        password_hash: password_hash.into(),
        user_salt: user_salt.into(),
        email: email_enc,
    };

    insert_into_database(user_preprocessed, db).await
}

async fn find_user_in_database(
    db: web::Data<Pool>,
    username: &str,
) -> Result<(Vec<u8>, Vec<u8>, UserId), HttpResponse> {
    // (password_hash, salt)
    let select_stmt = include_str!("../../sql/query_user.sql");

    let client = match db.get().await {
        Ok(cli) => cli,
        Err(_) => return Err(HttpResponse::ServiceUnavailable().finish()),
    };

    let query_result = client
        .query_opt(select_stmt, &[&username.to_string()])
        .await;

    let row = match query_result {
        Ok(row_result) => {
            if let Some(row) = row_result {
                row
            } else {
                return Err(HttpResponse::Unauthorized().finish());
            }
        }
        Err(_) => {
            return Err(HttpResponse::ServiceUnavailable().finish());
        }
    };

    let password_hash: Vec<u8> = row.get(0);
    let user_salt: Vec<u8> = row.get(1);
    let user_id: i64 = row.get(2);

    Ok((password_hash, user_salt, user_id))
}

/// Function tries to authenticate user with data in 'form'. Possible respones:
///     HTTP 202 => User authenticated successfully.
///     HTTP 401 => Function with given data does not exist in database.
///     HTTP 503 => Server problem, try again later.
///     HTTP 422 => Either username or password does not meet formal requirements.
async fn login_into_account(
    db: web::Data<Pool>,
    form: web::Json<LoginForm>,
    hasher: web::Data<Hasher<'_>>,
    encryptor: web::Data<Encryptor>,
) -> HttpResponse {
    if form.username.len() > crate::MAX_USERNAME_LENGTH {
        return HttpResponse::UnprocessableEntity().body("Username is too long.");
    }

    let (password_hash, user_salt, user_id) = match find_user_in_database(db, &form.username).await
    {
        Ok(res) => res,
        Err(e) => return e,
    };

    match hasher.password_matches(&form.password, &password_hash, &user_salt) {
        Err(_) => HttpResponse::ServiceUnavailable().finish(),
        Ok(false) => HttpResponse::Unauthorized().finish(),
        Ok(true) => {
            use actix_web::cookie::Expiration;

            let token = encryptor
                .encrypt(form.username.as_bytes(), &user_salt)
                .unwrap();

            // TODO: Expiration time and its renewal
            let cookie_at = CookieBuilder::new("auth_token", crate::crypto::encode_hex(&token))
                .secure(true)
                .expires(Expiration::Session)
                .finish();

            let token_id = encryptor
                .encrypt(&user_id.to_ne_bytes(), &crate::crypto::id_salt())
                .unwrap();

            let cookie_id =
                CookieBuilder::new("auth_token_id", crate::crypto::encode_hex(&token_id))
                    .secure(true)
                    .expires(Expiration::Session)
                    .finish();

            HttpResponse::Accepted()
                .cookie(cookie_at)
                .cookie(cookie_id)
                .finish()
        }
    }
}

/// Function is used to handle "/account" route.
pub fn account_handler(cfg: &mut web::ServiceConfig) {
    cfg.route("/create", web::post().to(create_account));
    cfg.route("/login", web::post().to(login_into_account));
}
