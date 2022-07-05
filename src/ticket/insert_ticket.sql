INSERT INTO ticket_data.tickets (id, owner_id, lang, content, exercise_id, ticket_status)
VALUES ($1, $2, $3, $4, $5, $6)
RETURNING id;