use aes_gcm::NewAead;

/// Strucute to wrap entryptor to provide as actix_web state.
pub struct Encryptor {
    /// Aes encryption structure.
    machine: aes_gcm::Aes256Gcm,
}

#[derive(Debug)]
pub enum CryptoError {
    InternalError,
    OutPlaceTooSmall,
    OutPlaceTooBig,
    UserSaltTooSmall,
}

use aes_gcm::aead::Aead;
use aes_gcm::Nonce;
impl Encryptor {
    pub fn new(key: &[u8]) -> Encryptor {
        let machine = aes_gcm::Aes256Gcm::new(aes_gcm::Key::from_slice(key));

        Encryptor { machine }
    }

    /// 'user_salt' should be at least 12 bytes long.
    pub fn encrypt(&self, to_encrypt: &[u8], user_salt: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if user_salt.len() < 12 {
            return Err(CryptoError::UserSaltTooSmall);
        }

        let mut nonce = [0u8; 12];
        let mut salt_iterator = user_salt.iter();

        for val in nonce.iter_mut() {
            *val = *salt_iterator.next().unwrap();
        }

        match self.machine.encrypt(Nonce::from_slice(&nonce), to_encrypt) {
            Ok(result) => Ok(result),
            Err(error) => {
                error!(
                    "Error occured while condcting encryption process. ERROR = {}",
                    error
                );
                Err(CryptoError::InternalError)
            }
        }
    }

    /// 'user_salt' should be at least 12 bytes long.
    pub fn decrypt(&self, to_decrypt: &[u8], user_salt: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if user_salt.len() < 12 {
            return Err(CryptoError::UserSaltTooSmall);
        }

        let mut nonce = [0u8; 12];
        let mut salt_iterator = user_salt.iter();

        for val in nonce.iter_mut() {
            *val = salt_iterator.next().unwrap().to_owned();
        }

        match self.machine.decrypt(Nonce::from_slice(&nonce), to_decrypt) {
            Ok(result) => Ok(result),
            Err(error) => {
                error!(
                    "Error occured while conducting decryption process. ERROR = {}",
                    error
                );
                Err(CryptoError::InternalError)
            }
        }
    }
}

impl Clone for Encryptor {
    fn clone(&self) -> Self {
        Self {
            machine: self.machine.clone(),
        }
    }
}

use argon2::Argon2;
pub struct Hasher<'a> {
    machine: Argon2<'a>,
}

impl<'a> Hasher<'a> {
    /// Generates new hasher with supplied secret (Pepper).
    pub fn new(secret: &'a [u8]) -> Hasher {
        use argon2::Params;

        let params = Params::new(
            Params::DEFAULT_M_COST,
            Params::DEFAULT_T_COST,
            Params::DEFAULT_P_COST,
            Some(crate::HASH_LENGTH_BYTES),
        )
        .expect("Wrong hasher configuration.");

        let machine = Argon2::new_with_secret(
            secret,
            argon2::Algorithm::Argon2id,
            argon2::Version::default(),
            params,
        )
        .expect("Unable to construct Argon2 structure. Probably bad secret was supplied.");

        Hasher { machine }
    }

    /// Hashes password into 'out_place'.
    /// Salt should be
    /// Parameter 'out_place' should have length as 'crate::HASH_LENGTH_BYTES', otherwise it will fail.
    pub fn hash_password(
        &self,
        password: &str,
        salt: &[u8],
        out_place: &mut [u8],
    ) -> Result<(), CryptoError> {
        match self
            .machine
            .hash_password_into(password.as_bytes(), salt, out_place)
        {
            Ok(_) => Ok(()),
            Err(argon2::Error::OutputTooLong) => Err(CryptoError::OutPlaceTooBig),
            Err(argon2::Error::OutputTooShort) => Err(CryptoError::OutPlaceTooSmall),
            Err(error) => {
                error!(
                    "Error occured while conducting hashing process. ERROR = {}",
                    error
                );
                Err(CryptoError::InternalError)
            }
        }
    }

    pub fn password_matches(
        &self,
        password: &str,
        hash: &[u8],
        salt: &[u8],
    ) -> Result<bool, CryptoError> {
        let mut out_hash: [u8; crate::HASH_LENGTH_BYTES] = [0u8; crate::HASH_LENGTH_BYTES];

        self.hash_password(password, salt, &mut out_hash)?;

        Ok(out_hash == hash)
    }
}

impl Clone for Hasher<'_> {
    fn clone(&self) -> Self {
        Self {
            machine: self.machine.clone(),
        }
    }
}

pub fn generate_salt() -> [u8; crate::HASH_SALT_LEN] {
    use rand::prelude::*;
    let mut result = [0u8; crate::HASH_SALT_LEN];

    let mut rng = rand::thread_rng();

    for val in result.iter_mut() {
        *val = rng.gen();
    }

    result
}

use deadpool_postgres::Pool;
use tokio_postgres::NoTls;
/// Function inits deadpool with read values from env variables.
/// PG__DBNAME -> Database name.
/// PG__USER -> Username for application account.
/// PG__HOST -> Address of the database.
/// PG__PORT -> Port of the database.
pub async fn init_database_pool() -> Pool {
    let mut config = deadpool_postgres::Config::new();

    config.dbname =
        Some(std::env::var("PG__DBNAME").expect("Unable to find PG__DBNAME env variable.")); // Database name

    config.user = Some(std::env::var("PG__USER").expect("Unable to find PG__USER env variable.")); // Username for application account

    config.host = Some(std::env::var("PG__HOST").expect("Unable to find PG__HOST env variable.")); // Address of the database

    config.port = Some(
        std::env::var("PG__PORT")
            .expect("Unable to find PG__PORT env variable.")
            .parse()
            .expect("Value of PG__PORT is not 16-bit unsigned int number."),
    ); // Port at which database is running.

    info!("Successfully parsed environment variables for database configuration.");

    let pool = config
        .create_pool(Some(deadpool_postgres::Runtime::Tokio1), NoTls)
        .expect("Unable to create dabatabse pool.");

    let _connection_testing = pool
        .get()
        .await
        .expect("Connection with database cannot be established.");

    info!("Creation of database pool and connection checking were successful.");
    pool
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut result_string = String::with_capacity(2 * bytes.len());

    for byte in bytes {
        result_string.push_str(&format!("{:02x}", byte));
    }

    result_string
}
