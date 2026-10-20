# Downstream patch stack

This repository is [Herdr UI](https://github.com/penso/herdr-gpui) with downstream patches on top.
It is not affiliated with the upstream project.
Downstream changes are an explicit [forkctl](https://github.com/victor-software-house/forkctl) / StGit stack:
do not commit ad-hoc work onto `main`.

- Upstream: `penso/herdr-gpui`, fetched as the `upstream` remote, push disabled.
- Base: the upstream release tag recorded in [`patches/fork.yaml`](patches/fork.yaml).
- Patches, their purpose and when each can be dropped: [`patches/LEDGER.md`](patches/LEDGER.md), generated; do not edit it.
- Pull requests stay in this repository.
  Nothing is proposed upstream until the downstream problems are fixed.

## Work on the stack

The `fork` task is mounted by [`.config/mise.toml`](.config/mise.toml);
upstream owns `mise.toml`.

```sh
mise run fork status
mise run fork check
mise run fork patch create NAME \
  --kind source \
  --purpose 'Why this change exists.' \
  --upstream-status not-submitted \
  --drop-when 'Upstream provides it.' \
  --scope 'crates/herdr-gpui/src/path/**'
git add -- crates/herdr-gpui/src/path/file.rs
mise run fork patch refresh
mise run fork patch finish
mise run fork publish
```

`publish` defaults to `propose`: it pushes `forkctl/proposal/main` and opens or updates its draft pull request.
CI runs on that pull request.
After review, `mise run fork publish --promote` moves `main` to the proposed stack.

## Follow upstream

```sh
git fetch upstream --tags
mise run fork rebase --onto refs/tags/vYYYYMMDD.N --dry-run
mise run fork rebase --onto refs/tags/vYYYYMMDD.N
mise run fork publish
```

Rebase onto release tags, not upstream `main`, so the base is always a published version.
