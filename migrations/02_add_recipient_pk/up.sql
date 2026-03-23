ALTER TABLE payment_addresses ADD COLUMN recipient_pk VARCHAR(66);
CREATE INDEX idx_recipient_pk ON payment_addresses (recipient_pk);
