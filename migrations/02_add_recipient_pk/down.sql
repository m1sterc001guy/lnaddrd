DROP INDEX IF EXISTS idx_recipient_pk;
ALTER TABLE payment_addresses DROP COLUMN IF EXISTS recipient_pk;
