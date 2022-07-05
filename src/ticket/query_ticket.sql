SELECT *
FROM ticket_data.tickets
WHERE id = $1
LIMIT 1;