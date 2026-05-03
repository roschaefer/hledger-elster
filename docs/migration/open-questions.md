# Open questions

## Interface

- Should `tax/` read `hledger` journal files directly, or only exported CSVs?
- What exact posting export schema should `ledger/` provide?

## Modeling

- Which tax-relevant adjustments should remain purely in `tax/`?
- Which adjustments, if any, should become journal entries in `ledger/`?

## Outputs

- Which reports should `tax/` generate directly from posting exports?
- Which spreadsheet layouts from V1 still matter as explicit compatibility targets?

## Verification

- At what point should `abgleich.yml` be migrated from V1 expectations to the new architecture?
