-- Add the first application authorization role boundary.
--
-- Existing authenticated users predate authorization and therefore retain
-- their existing effective access as operators.
--
-- The default for any future insert that accidentally omits an explicit role
-- is deliberately the lower-privilege viewer role.

ALTER TABLE public.users
    ADD COLUMN role TEXT NOT NULL DEFAULT 'viewer',
    ADD CONSTRAINT users_role_check
        CHECK (role IN ('operator', 'viewer'));

UPDATE public.users
SET role = 'operator';
