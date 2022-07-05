INSERT INTO user_data.users (id, username, password_hash, user_salt, email)
VALUES ($1, $2, $3, $4, $5)
RETURNING id;