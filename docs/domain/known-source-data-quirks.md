# Known source data quirks

## Kontist export limitations

Kontist export data is not always consistent with what the UI shows.

Examples learned during development:

- `Umsatzkategorie` shown in the UI may be missing in the CSV export
- `Privat` is not a reliable indicator for owner drawings
- tax-related categories may be incomplete or misleading in exports

## Architectural consequence

The system must not depend on bank-provided metadata as authoritative truth.

Bank metadata may be used as:

- a hint
- a convenience
- an import aid

But not as the final basis for tax logic or bookkeeping classification.

## Design implication

This strongly supports:

- a bank-agnostic bookkeeping layer
- explicit classification through our own account structure and rules
