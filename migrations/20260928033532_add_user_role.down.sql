ALTER TABLE public.users
    DROP CONSTRAINT users_role_check,
    DROP COLUMN role;
