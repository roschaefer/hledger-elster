# Current state

At the start of the migration:

- `ledger/` contains imported `hledger` journals for multiple years and accounts
- `tax/` contains the first-generation Python tax reporting tool

The existing `tax/` tool currently still includes logic for:

- raw bank CSV import
- normalization
- classification
- tax reporting
- audit artifacts
- verification against older tax calculations

This is the baseline to migrate away from.
