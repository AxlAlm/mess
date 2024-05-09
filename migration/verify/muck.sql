-- Verify mess:muck on sqlite

BEGIN;

SELECT 
    id, name, description, registered_at, deregistered_at, alive
FROM 
    muck
WHERE 
    0;

ROLLBACK;
