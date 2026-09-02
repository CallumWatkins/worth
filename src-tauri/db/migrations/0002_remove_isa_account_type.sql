-- Reassign ISA accounts to Savings and remove the ISA account type.
UPDATE accounts
SET
  type_id = (SELECT id FROM account_types WHERE name = 'savings')
WHERE
  type_id = (SELECT id FROM account_types WHERE name = 'isa');

DELETE FROM account_types WHERE name = 'isa';
