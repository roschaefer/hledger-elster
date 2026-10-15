# Commit evidence cannot be written into the repository

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

Starts from the chapter's [shared setup](README.md#shared-setup), which
[`setup.bash`](setup.bash) builds:

```scrut
$ build_scenario scenario_commit_evidence
```

```scrut
$ hledger-elster export-commit-evidence --output commit-evidence.pdf
Error: commit evidence output must be outside the Git repository: */scenario/work/commit-evidence.pdf (glob)
[1]
```

```scrut
$ test ! -e commit-evidence.pdf
```

```scrut
$ git status --porcelain
```
