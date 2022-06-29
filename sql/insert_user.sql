INSERT INTO user_data.users (username, password_hash, user_salt, email)
VALUES ($1, $2, $3, $4);