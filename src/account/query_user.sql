SELECT password_hash, user_salt, id
FROM user_data.users
WHERE username = $1
LIMIT 1;