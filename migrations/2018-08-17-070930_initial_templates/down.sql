-- This file should undo anything in `up.sql`
DELETE FROM templates WHERE name = 'order_create_for_store';
DELETE FROM templates WHERE name = 'order_update_state_for_store';
DELETE FROM templates WHERE name = 'apply_email_verification_for_user';
DELETE FROM templates WHERE name = 'email_verification_for_user';
DELETE FROM templates WHERE name = 'order_create_for_user';
DELETE FROM templates WHERE name = 'order_update_state_for_user';
DELETE FROM templates WHERE name = 'apply_password_reset_for_user';
DELETE FROM templates WHERE name = 'password_reset_for_user';