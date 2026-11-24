# Commit evidence cannot be written into the repository

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

The `scrut` blocks below use helpers from [`readme-setup.sh`](../readme-setup.sh),
such as `new_scenario`, which starts each scenario in a new empty directory.

Starts from the chapter's [shared setup](README.md#shared-setup), which
[`setup.bash`](setup.bash) builds:

```scrut
$ build_scenario scenario_commit_evidence
```

```scrut
$ hledger-elster export-commit-evidence --output commit-evidence.pdf
Error: commit evidence output must be outside the Git repository: */work/commit-evidence.pdf (glob)
[1]
```

```scrut
$ test ! -e commit-evidence.pdf
```

```scrut
$ git status --porcelain
```

## Dangling symlink cycles return an error

A missing path component can hide a symlink cycle from the filesystem's
canonicalization. Resolving such an output must fail without crashing or
creating directories, including when overwriting is requested.

```scrut
$ ln -s missing/../cycle.pdf ../cycle.pdf
```

```scrut
$ hledger-elster export-commit-evidence --force --output ../cycle.pdf
Error: failed to write commit evidence PDF at */work/../cycle.pdf: too many symlinks while resolving commit evidence output (glob)
[1]
```

```scrut
$ test ! -e ../missing
```

## A symlink cannot hide an output inside the repository

The output may name a symlink outside the repository, including a directory that
has not been created yet. It must still be rejected before changing the tree.

```scrut
$ ln -s "$PWD" ../repository-alias
```

```scrut
$ hledger-elster export-commit-evidence --output ../repository-alias/new/nested/commit-evidence.pdf
Error: commit evidence output must be outside the Git repository: */work/new/nested/commit-evidence.pdf (glob)
[1]
```

```scrut
$ test ! -e new
```

```scrut
$ git status --porcelain
```

An existing or dangling file symlink also cannot redirect a forced write into
the repository.

```scrut
$ ln -s "$PWD/commit-evidence.pdf" ../evidence-alias.pdf
```

```scrut
$ hledger-elster export-commit-evidence --force --output ../evidence-alias.pdf
Error: commit evidence output must be outside the Git repository: */work/commit-evidence.pdf (glob)
[1]
```

```scrut
$ test ! -e commit-evidence.pdf
```

```scrut
$ git status --porcelain
```
