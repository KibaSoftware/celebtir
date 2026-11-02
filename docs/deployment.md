# Celebtir installation and deployment

The root `deploy.ps1` is an owner-local file intentionally excluded from Git. A fresh clone includes the host helpers under `scripts/deploy/`, but not that local entry point or `.env`. Deployment-entry tests likewise require the local script.

Run `deploy.ps1` from the checkout you intend to publish. It presents exactly two choices:

1. **Install and deploy** — install missing host dependencies, configure Celebtir credentials, HTTPS and timers, then build and publish fresh data.
2. **Complete deployment** — rebuild the current application, collect fresh data, score with the release's latest versioned methodology, and publish the complete release on an installed host.

“Latest” means the source in this local checkout, including uncommitted files. The script does not pull Git, commit, push or select a remote branch. Both modes upload the same inspected source package. Neither uploads local datasets, QA seeds, `.env`, registry drafts, dependencies or compiled output.

## Commands

```powershell
Set-Location D:\Work\celebtir
.\deploy.ps1                      # choose 1 or 2
.\deploy.ps1 -Mode Install        # installation + full deployment
.\deploy.ps1 -Mode Deploy         # full deployment on an installed host
```

`Install` asks for the certificate email if omitted; `-CertificateEmail 'you@example.com'` supplies it explicitly. The obsolete `-Update` and `-UpdateRpc` switches are removed. There is no code-only mode.

For local inspection without a connection:

```powershell
.\deploy.ps1 -Mode Install -DryRun
.\deploy.ps1 -Mode Deploy -DryRun
.\deploy.ps1 -Mode Deploy -PackageOnly
```

`DryRun` and `PackageOnly` are validation switches, not additional deployment modes. Packaging saves the inspected archive, entry list and hash under `output/predeployment/`. Paths with spaces are supported. Native-command failure stops execution and removes only that invocation's temporary files.

Default target: `jarvis@57.128.251.85`, SSH port22, `/opt/celebtir`, `https://celebtir.com`. `-RemoteUser`, `-RemoteHost`, `-Port` and `-IdentityFile` override SSH transport. Website/domain/managed paths remain fixed.

Each real deployment asks for the **SSH / sudo account password once**, through hidden input before packaging. That password is reused for the authentication preflight, SSH commands, SCP uploads, sudo and remote cleanup. `DryRun` and `PackageOnly` never ask for it. The SSH account and sudo must accept the same password; a different sudo password or MFA arrangement is not automatically guessed or retried interactively. Wrong SSH authentication stops before packaging/uploading.

The credential exists only for this invocation: a current-user Windows DPAPI-encrypted file in a user-only temporary directory and transient process memory. No plaintext password is put in command arguments, environment variables, the source archive or server files. The script restores the previous askpass environment and removes its encrypted credential and generated helper in `finally`, including ordinary failure paths. Abrupt process termination can leave encrypted temporary artifacts; they are not reused on the next run.

SSH host-key checking is strict. On a first connection, verify the server fingerprint through a trusted channel and establish its `known_hosts` entry with `ssh -p 22 jarvis@57.128.251.85` before running deployment. Changed or unknown keys stop deployment rather than being accepted automatically. `-IdentityFile` still works with an unlocked key; unlock an encrypted private key in `ssh-agent` beforehand. The account password is never supplied as a private-key passphrase or host-key confirmation.

The local helper is compiled using Windows PowerShell 5.1/.NET Framework already included with Windows; no additional package is installed. Native SSH/SCP use [OpenSSH's askpass mechanism](https://man.openbsd.org/ssh.1#SSH_ASKPASS_REQUIRE). For sudo, the relay waits until the remote PTY has disabled echo, then sends one password frame over SSH. The shell consumes that frame even when sudo credentials are cached or passwordless. The root command reconnects stdin to `/dev/tty`, so leftover password input cannot become an answer to installation prompts. RPC/API credential prompts remain interactive and separate.

## Installation and credentials

Supported host family: Ubuntu with systemd. Install adds missing Nginx, Certbot and compiler dependencies through apt without upgrading the OS or changing package sources. Unrelated virtual hosts, services and Node installations are preserved. Unsupported or broken/EOL package repositories need separate maintenance.

Celebtir uses a dedicated account and isolated Node22 under `/opt/celebtir/tools/node`. Application builds read the pinned Rust version from `rust-toolchain.toml` and pnpm version from `site/package.json`, install those project tool versions and use locked dependencies. Complete deployment can update these application tool versions; missing base host tools require `Install`.

First installation asks for the Solana RPC URL, optional HyperEVM URL and optional Etherscan key through hidden input. Values stay on the server in `/etc/celebtir/collector.env` (root owner, Celebtir group,0640). Blank values use public collector fallbacks, subject to their limits. No local `.env` is read or uploaded. Re-running `Install` offers to replace the saved Solana URL while retaining other provider settings. `Deploy` reuses saved values and performs a redacted RPC preflight before compiling. Invalid credentials stop deployment.

New installations start with an empty data generation, not a QA seed. Reinstallation preserves existing managed data. Unmanaged directories/configuration are rejected rather than silently adopted.

## Fresh data and same-day revisions

Every explicit `Install` or `Deploy` collects today's UTC snapshot. It selects the numerically latest versioned TOML in `methodology/`, verifies the scorer's bundled copy matches, and passes that exact file to scoring. Currently this is methodology0.3. `/data/v2/` is the independent API schema path.

When today already exists, deployment first copies the active data generation into an isolated candidate. Inside that candidate only, the previous complete snapshot moves to:

```text
snapshot-revisions/YYYY-MM-DD-<release-id>/snapshot/
snapshot-revisions/YYYY-MM-DD-<release-id>/revision.json
```

The revision manifest records SHA-256 for every preserved file. Fresh collection creates a new `snapshots/YYYY-MM-DD/`. Older days and archived same-day facts/scores/events remain unchanged. Archives survive generation pruning inside the active data matrix. They are research archives, not extra daily chart observations or public API routes. No old facts are rescored. The prior `latest/facts.json` remains available to collection for history continuity.

Explicit deployment requires successful collection. Partial collection, missing tokens/protocols, stale methodology/date, failed build/invariants or failed public verification prevent success. Failure before publication leaves the live release untouched; the existing journal restores data/web/source pointers after a failed switch. This is stricter than the normal daily partial-publication policy.

The daily job remains at **02:17 UTC**. It collects a missing day; a repeated same-day timer invocation reuses that day's snapshot and validates its methodology. It does not create a revision automatically. An old-methodology same-day snapshot fails the release check and requires an explicit fresh deployment. Missed days remain gaps, never fabricated observations.

## Publication and host configuration

Archive/helper hashes are verified before execution. Both modes run offline publication fixtures before a real release. The shared lock serializes collection/deployment. Checks cover the static release, nonempty routes, JS budget, local HTTP API, full current-methodology invariants and exact generated date/methodology. No old-methodology compatibility shortcut applies to complete deployment.

Managed source/data/web pointers switch through the recovery journal. Public HTTPS API and exact release-marker checks must pass before old generations/exports/source releases are pruned. Initial installation finishes these checks after HTTPS configuration. Snapshots/revisions remain in the active generation. Cargo output is removed after retaining the executable; Next intermediates are disposable.

`Deploy` preserves credentials/certificates and migrates only the managed API route if needed. It enables and verifies existing Celebtir collection/certificate timers after success. Missing service/timer files require `Install`. Installation configures the managed virtual host, renewal and daily services. The existing noindex policy remains until separately approved for launch.

## DNS and HTTPS

Point the `celebtir.com` A record to `57.128.251.85`, and `www` CNAME to `celebtir.com`. Preserve unrelated DNS/MX/TXT records. Remove conflicting parking/redirect records only for these names. Do not retain an unconfigured AAAA record. Public ports80/443 must reach this host; deployment does not reset the shared firewall. Both names need working DNS before certificate issuance.

```powershell
Resolve-DnsName celebtir.com -Type A
Resolve-DnsName www.celebtir.com -Type A
```

Certbot webroot issuance affects only the Celebtir certificate. Nginx is validated before reload. Unmanaged/conflicting Celebtir blocks cause a stop. `www` redirects to canonical HTTPS. Other sites' certificates and renewal configuration remain unchanged.

## Verify and operate

```bash
sudo systemctl list-timers 'celebtir-*'
sudo journalctl -u celebtir-update.service -n 100 --no-pager
sudo cat /opt/celebtir/shared/last-run.txt
sudo cat /opt/celebtir/source/DEPLOYMENT.txt
curl -fsS https://celebtir.com/data/v2/summary.json
curl -fsS https://celebtir.com/deployment-id.txt
sudo certbot renew --cert-name celebtir.com --dry-run
```

Check actual snapshot date and `methodology_version`, not just the API URL. Verify Summary, Risk, Tokens, Compare, protocol details and HyperEVM, plus unrelated host services. `sudo systemctl start celebtir-update.service` follows normal daily policy; use `-Mode Deploy` for an explicit same-day revision. Stop future scheduled runs with `sudo systemctl stop celebtir-update.timer`; this does not interrupt a running service.

Minimal retention keeps one active source/export/data generation after verification. Revision archives preserve evidence, not old web/source builds. Reverting code requires deploying a reviewed checkout with fresh data; archives are not an automatic rollback command. Never overwrite historical scores manually to make a retry succeed.

## Validation status

The GitHub daily collection/data-commit/Pages workflow and its schedule monitor require the repository variable `CELEBTIR_ENABLE_GITHUB_DAILY=true`. They remain inactive by default while the host deployment path is in use; a first source push does not opt in to that publication path.

This rework is local only. PowerShell entry/packaging and portable archive/version/storage/RPC/Nginx tests are under `scripts/tests/`; logs are in `output/deployment-rework/`. The Bash publication harness requires native Unix symlinks; Windows reports exit77. Linux CI and actual host installation, HTTPS, timers and publication remain mandatory verification. No remote deployment was performed during implementation.
