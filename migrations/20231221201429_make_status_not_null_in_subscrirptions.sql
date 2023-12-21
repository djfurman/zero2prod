-- Wrapping full migration in a transaction to make sure it succeeds or fails automatically
-- `sqlx` does not automatically do this for us, which is a good thing
Begin;
    -- Backfill `status` for historical entries
    Update subscriptions
        Set status = 'confirmed'
        Where status IS NULL;
    -- Make `status` mandatory
    Alter Table subscriptions Alter Column status Set Not Null;
Commit;
