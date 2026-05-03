# Monorepo transition

This document records the transition from two standalone repositories to one shared `finances/` repository.

## Current state

- shared root exists at `finances/`
- shared documentation already lives at the root
- `ledger/` still has its own nested `.git`
- `tax/` still has its own nested `.git`

## Intended target

```text
finances/
  .git
  AGENTS.md
  README.md
  docs/
  ledger/
  tax/
```

## Principle

The monorepo should provide:

- shared documentation
- coordinated refactors
- a single architectural home

It should not erase the conceptual boundary between:

- bookkeeping in `ledger/`
- tax reporting in `tax/`

## Open implementation choice

How to preserve history when merging the two repositories:

- import both histories into the new root repository
- or keep only the current working trees and treat older history as archival

This decision should be made explicitly before removing the nested `.git` directories.

## Safe next steps

1. Create a root repository at `finances/`
2. Commit shared docs and root metadata
3. Decide on history preservation strategy
4. Remove nested `.git` directories only after that decision
