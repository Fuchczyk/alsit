//! ## Environment variables
//! Some variables must be set in order to run this server:
//! 
//! * TESTING_IMAGE_NAME defines name of docker images used to conduct
//!   testing. It is read by default from 'config.json'
//! * TESTS_PATH defines path (absolute) at the machine, where tests data
//!   is stored. It is read by default from 'config.json'. More info about
//!   tests format is in module [alsit::judge].
//! * ALSIT_ADDRESS defines IP address at which server should start. Example:
//!   "127.0.0.1:8080".
//! 
//! [alsit::judge]: crate::judge
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod account;
mod crypto;
mod judge;
mod ticket;

use actix_web::{web, App, HttpServer};
use bollard::Docker;
use lazy_static::lazy_static;

const HASH_LENGTH_BYTES: usize = 32;
const MAX_USERNAME_LENGTH: usize = 40;
const HASH_SALT_LEN: usize = 16;
const ENCRYPT_NONCE_LEN: usize = 12;

lazy_static! {
    static ref DOCKER: Docker = Docker::connect_with_socket_defaults().unwrap();
}

/// Retriving key used for encryption.
fn encryption_seed() -> [u8; 32] {
    // Perfectly to restore it from something.
    [
        35, 23, 12, 63, 21, 23, 92, 2, 1, 62, 173, 12, 162, 36, 232, 15, 35, 23, 12, 63, 21, 23,
        35, 23, 12, 63, 35, 23, 12, 63, 2, 3,
    ]
}

/// Retriving key which serves as base for hashing.
fn hashing_seed() -> [u8; 64] {
    [
        35, 23, 12, 63, 21, 23, 92, 2, 1, 62, 173, 12, 162, 36, 232, 15, 35, 23, 12, 63, 21, 23,
        92, 2, 1, 62, 173, 12, 162, 36, 232, 15, 35, 23, 12, 63, 21, 23, 92, 2, 1, 62, 173, 12,
        162, 36, 232, 15, 35, 23, 12, 63, 21, 23, 92, 2, 1, 62, 173, 12, 162, 36, 232, 15,
    ]
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init_timed();

    if cfg!(debug_assertions) {
        warn!("You are running the application in debug mode! It should only be used for developement.");

        info!("Reading env configuration from db.env file in sql directory.");
        dotenv::from_filename("sql/db.env").expect("Unable to find file ../sql/db.env");
    }

    let server_address =
        std::env::var("ALSIT_ADDRESS").expect("Unable to find ALSIT__ADDRESS env variable.");

    let password_hash = Box::leak(Box::new(hashing_seed()));

    let encryptor = crypto::Encryptor::new(&encryption_seed());

    let hasher = crypto::Hasher::new(password_hash);

    let pool = crypto::init_database_pool().await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(encryptor.clone()))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(hasher.clone()))
            .service(web::scope("/account").configure(account::account_handler))
            .service(web::scope("/ticket").configure(ticket::ticket_handler))
    })
    .bind(server_address)?
    .run()
    .await
}
