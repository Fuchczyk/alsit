SELECT *
FROM user_data.users
WHERE id = $1
LIMIT 1;