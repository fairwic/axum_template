DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND indexname = 'users_phone_uniq'
    ) THEN
        CREATE UNIQUE INDEX users_phone_uniq ON users (phone) WHERE phone IS NOT NULL;
    END IF;
END $$;
