# Emergent Garden local integration provenance

Local-only integration candidate. No branch was pushed and no hosted pull request was merged.

| Role                         | Full SHA                                   |
| ---------------------------- | ------------------------------------------ |
| Frozen main base             | `ddc2aa85930facbeb058242559bdbee5a5248161` |
| Hosted census child          | `cfb0e7e1a91adba38a44b0c5d3487a31e9737c79` |
| Local repair tip             | `122f603b69da59309a44be98a9651e902549240b` |
| Local repair parent          | `e5361d9dfdb6d0a63737c3e243734d24a1f8afa2` |
| Hosted/local common ancestor | `2cc1046727b968cb4d38367f445d288a8866ceae` |
| Main/feature common ancestor | `67a7cccc8ed5292a2cfbe341d626d0c3bb2cc3dc` |

## Reconciliation

The hosted census child was retained. Its two persisted raw pagination-token
fields were replaced with SHA-256 metadata at `api_requests[2]` and
`api_requests[6]`. Each replacement is
`43a1dbaaf49c2dc724c75907ea9bf63b30e9ba4a8aaef60b2bb0139388b48110`,
which matches the prior token digest already recorded at the common ancestor.
The original hosted inventory SHA-256,
`ec650ea172cedfe26c3e95c319de59e0cfd77c4e3a369d4a73007fa37ceaec26`, is
retained here as provenance. All other hosted inventory bytes were retained.
The census manifest inventory entry now records the sanitized inventory
SHA-256, `3d965198e36b496e14da13e0ae4f7dd9dca89fa243da1c96833654153430c0eb`.

The sole Git conflict was `.github/workflows/review-fanout.yml`. The candidate
retains main's `actions/github-script` v7.1.0 pin because it accompanies the
main dispatch payload/schema change.
