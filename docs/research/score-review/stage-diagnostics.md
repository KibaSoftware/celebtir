# Stage diagnostics

Hosted snapshot 2026-10-26; methodology 0.2. All 12 Solana records are included once. Distribution: {"unrated":2,"stage_0":10}. Native HyperEVM and HIP-3 are excluded.

Current scorer replay agrees with 12/12 complete published protocol score objects.

| Protocol | Stage | Verified unmet | Unresolved | Determination |
|---|---|---:|---:|---|
| binance-staked-sol | unrated | 1 | 0 | VERIFIED_CONTROL_BOTTLENECK |
| drift-staked-sol | stage_0 | 1 | 2 | MIXED |
| forward-sol | unrated | 1 | 0 | VERIFIED_CONTROL_BOTTLENECK |
| gmx-solana | stage_0 | 1 | 1 | MIXED |
| jito | stage_0 | 1 | 2 | MIXED |
| jupiter-lend | stage_0 | 1 | 2 | MIXED |
| jupiter-perps | stage_0 | 1 | 1 | MIXED |
| jupiter-staked-sol | stage_0 | 1 | 2 | MIXED |
| kamino-lend | stage_0 | 2 | 0 | VERIFIED_CONTROL_BOTTLENECK |
| orca | stage_0 | 1 | 1 | MIXED |
| pumpswap | stage_0 | 1 | 1 | MIXED |
| raydium-amm | stage_0 | 2 | 1 | MIXED |

## Criterion frequencies

Multiple blockers can occur per protocol: percentages can sum above 100%. Denominators include unresolved eligible protocols and exclude explicit exemptions. Percentages use integer half-up rounding to two decimal places.

| Criterion | Verified / eligible | Unresolved / eligible |
|---|---|---|
| s0.core_programs | 0/2 (0.00%) | 0/2 (0.00%) |
| s0.public_source | 2/2 (100.00%) | 0/2 (0.00%) |
| s0.upgrade_control_known | 0/2 (0.00%) | 0/2 (0.00%) |
| s1.upgrade_control | 0/10 (0.00%) | 0/10 (0.00%) |
| s1.upgrade_delay | 10/10 (100.00%) | 0/10 (0.00%) |
| s1.admin_powers | 2/10 (20.00%) | 0/10 (0.00%) |
| s1.config_authority | 0/10 (0.00%) | 3/10 (30.00%) |
| s1.verified_builds | 0/10 (0.00%) | 8/10 (80.00%) |
| s1.no_unknown | 0/10 (0.00%) | 2/10 (20.00%) |

## Source traces

### Binance Staked SOL

admin_powers: **low** — the registry lists no admin accounts; the evidence is the core programs' upgrade reads

code_transparency: **high** — no public source

dependencies: **low** — all 1 callees rated Low on upgrade control

oracle_dependency: **not_applicable** — the registry lists no price feeds; the evidence is the core programs' upgrade reads

upgrade_control: **low** — Squads v3 multisig, 6 of 10 voters, time lock 0 s

upgrade_delay: **high** — time lock 0 s

- **MET** core programs identified in the registry: 1 core programs. Causes: none assigned. [upgrade_authority, 461491369](https://solscan.io/account/EmiU8AQkB2sswTxVB6aCmsAJftoowZGGDXuytm6X65R3) [threshold, time_lock, members, config_authority, 461491369](https://solscan.io/account/3yqoHFE4nBGchuVH5rJuZMFvsmnaDTuLLdvGPDUEJcbW)

- **UNMET_VERIFIED** public source for every core program: no public source: SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy. Causes: CODE_TRANSPARENCY_GAP. [verified_build, 461491368](https://solscan.io/account/SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy)

- **MET** upgrade control readable for every core program: upgrade control read for all core programs. Causes: none assigned. [upgrade_authority, 461491369](https://solscan.io/account/EmiU8AQkB2sswTxVB6aCmsAJftoowZGGDXuytm6X65R3) [threshold, time_lock, members, config_authority, 461491369](https://solscan.io/account/3yqoHFE4nBGchuVH5rJuZMFvsmnaDTuLLdvGPDUEJcbW)

### Drift Staked SOL

admin_powers: **low** — the registry lists no admin accounts; the evidence is the core programs' upgrade reads

code_transparency: **medium** — source public at https://github.com/drift-labs, but the deployed build is not verified

dependencies: **low** — all 1 callees rated Low on upgrade control

oracle_dependency: **not_applicable** — the registry lists no price feeds; the evidence is the core programs' upgrade reads

upgrade_control: **medium** — Squads v3 multisig, 6 of 11 voters, time lock 0 s

upgrade_delay: **high** — time lock 0 s

- **MET** upgrade control rated Low or Medium: MEDIUM: Squads v3 multisig, 6 of 11 voters, time lock 0 s. Causes: none assigned. [upgrade_authority, 461491674](https://solscan.io/account/Cn5fegqLh8Fmvffisr4Wk3LmuaUgMMzTFfEuidpZFsvV) [threshold, time_lock, members, config_authority, 461491674](https://solscan.io/account/AApfiPZgV5MoPU691GwhdDhq5sKEMMH1Uh8S4Z9xvP6b)

- **UNMET_VERIFIED** upgrade delay of the Stage 1 minimum, unless upgrades need an emergency multisig: SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY: effective time lock 0 s; needs 5 days+ or an emergency multisig of 8+ voters at 75%+. Causes: DELAY_BELOW_REQUIRED. [upgrade_authority, 461491674](https://solscan.io/account/Cn5fegqLh8Fmvffisr4Wk3LmuaUgMMzTFfEuidpZFsvV) [threshold, time_lock, members, config_authority, 461491674](https://solscan.io/account/AApfiPZgV5MoPU691GwhdDhq5sKEMMH1Uh8S4Z9xvP6b)

- **MET** admin powers rated Low or Medium: LOW: the registry lists no admin accounts; the evidence is the core programs' upgrade reads. Causes: none assigned. [upgrade_authority, 461491674](https://solscan.io/account/Cn5fegqLh8Fmvffisr4Wk3LmuaUgMMzTFfEuidpZFsvV) [threshold, time_lock, members, config_authority, 461491674](https://solscan.io/account/AApfiPZgV5MoPU691GwhdDhq5sKEMMH1Uh8S4Z9xvP6b)

- **UNRESOLVED_EVIDENCE** no multisig has a single-key config authority: no single-key config authority. Causes: none assigned. [upgrade_authority, 461491674](https://solscan.io/account/Cn5fegqLh8Fmvffisr4Wk3LmuaUgMMzTFfEuidpZFsvV) [threshold, time_lock, members, config_authority, 461491674](https://solscan.io/account/AApfiPZgV5MoPU691GwhdDhq5sKEMMH1Uh8S4Z9xvP6b)

- **UNRESOLVED_EVIDENCE** verified builds for all core programs: not verified: SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY. Causes: CODE_TRANSPARENCY_GAP. [verified_build, 461491673](https://solscan.io/account/SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY)

- **MET** every factor could be read (Unknown caps the Stage at 0): no Unknown factors. Causes: none assigned. [upgrade_authority, 461491674](https://solscan.io/account/Cn5fegqLh8Fmvffisr4Wk3LmuaUgMMzTFfEuidpZFsvV) [threshold, time_lock, members, config_authority, 461491674](https://solscan.io/account/AApfiPZgV5MoPU691GwhdDhq5sKEMMH1Uh8S4Z9xvP6b) [verified_build, 461491673](https://solscan.io/account/SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY) [transaction sample (50 transactions), 461491808](https://solscan.io/account/SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY) [upgrade_authority, 461492010](https://solscan.io/account/6WU8Nxarf9fudRK5atWwjLY4vFaw5UrrWhL88qz7iCMJ)

### Forward Industries SOL

admin_powers: **low** — the registry lists no admin accounts; the evidence is the core programs' upgrade reads

code_transparency: **high** — no public source

dependencies: **low** — all 1 callees rated Low on upgrade control

oracle_dependency: **not_applicable** — the registry lists no price feeds; the evidence is the core programs' upgrade reads

upgrade_control: **medium** — Squads v3 multisig, 6 of 11 voters, time lock 0 s

upgrade_delay: **high** — time lock 0 s

- **MET** core programs identified in the registry: 1 core programs. Causes: none assigned. [upgrade_authority, 461492010](https://solscan.io/account/Cn5fegqLh8Fmvffisr4Wk3LmuaUgMMzTFfEuidpZFsvV) [threshold, time_lock, members, config_authority, 461492011](https://solscan.io/account/AApfiPZgV5MoPU691GwhdDhq5sKEMMH1Uh8S4Z9xvP6b)

- **UNMET_VERIFIED** public source for every core program: no public source: SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY. Causes: CODE_TRANSPARENCY_GAP. [verified_build, 461492010](https://solscan.io/account/SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY)

- **MET** upgrade control readable for every core program: upgrade control read for all core programs. Causes: none assigned. [upgrade_authority, 461492010](https://solscan.io/account/Cn5fegqLh8Fmvffisr4Wk3LmuaUgMMzTFfEuidpZFsvV) [threshold, time_lock, members, config_authority, 461492011](https://solscan.io/account/AApfiPZgV5MoPU691GwhdDhq5sKEMMH1Uh8S4Z9xvP6b)

### GMX Solana

admin_powers: **medium** — `authority` can change parameters, held by Squads v4 multisig, 4 of 7 voters, time lock 600 s

code_transparency: **low** — verified build of https://github.com/gmsol-labs/gmx-solana/tree/8af1e70c0c0b6ff9d6b41ba4997910588fbcec28 at 8af1e70c0c0b6ff9d6b41ba4997910588fbcec28

dependencies: **high** — callee Gt9S…ZA3c is unrecognized controller AAL4…edAD

oracle_dependency: **unknown** — price feeds not yet listed in the registry

upgrade_control: **medium** — Squads v4 multisig, 4 of 7 voters, time lock 600 s

upgrade_delay: **high** — time lock 600 s

- **MET** upgrade control rated Low or Medium: MEDIUM: Squads v4 multisig, 4 of 7 voters, time lock 600 s. Causes: none assigned. [upgrade_authority, 461492306](https://solscan.io/account/CSp2sB68THdrJP6N7Au4Y5u89VJ1E1ebaqZ2soHhQvav) [threshold, time_lock, members, config_authority, 461492306](https://solscan.io/account/CxnEVpQQcYa628TywzHGXeJ2jdVmbU51rnERat9xunP1)

- **UNMET_VERIFIED** upgrade delay of the Stage 1 minimum, unless upgrades need an emergency multisig: Gmso1uvJnLbawvw7yezdfCDcPydwW2s2iqG3w6MDucLo: effective time lock 600 s; needs 5 days+ or an emergency multisig of 8+ voters at 75%+. Causes: DELAY_BELOW_REQUIRED. [upgrade_authority, 461492306](https://solscan.io/account/CSp2sB68THdrJP6N7Au4Y5u89VJ1E1ebaqZ2soHhQvav) [threshold, time_lock, members, config_authority, 461492306](https://solscan.io/account/CxnEVpQQcYa628TywzHGXeJ2jdVmbU51rnERat9xunP1)

- **MET** admin powers rated Low or Medium: MEDIUM: `authority` can change parameters, held by Squads v4 multisig, 4 of 7 voters, time lock 600 s. Causes: none assigned. [authority, 461492307](https://solscan.io/account/CTDLvGGXnoxvqLyTpGzdGLg9pD6JexKxKXSV8tqqo8bN) [threshold, time_lock, members, config_authority, 461492306](https://solscan.io/account/CxnEVpQQcYa628TywzHGXeJ2jdVmbU51rnERat9xunP1)

- **MET** no multisig has a single-key config authority: no single-key config authority. Causes: none assigned. [upgrade_authority, 461492306](https://solscan.io/account/CSp2sB68THdrJP6N7Au4Y5u89VJ1E1ebaqZ2soHhQvav) [threshold, time_lock, members, config_authority, 461492306](https://solscan.io/account/CxnEVpQQcYa628TywzHGXeJ2jdVmbU51rnERat9xunP1) [authority, 461492307](https://solscan.io/account/CTDLvGGXnoxvqLyTpGzdGLg9pD6JexKxKXSV8tqqo8bN)

- **MET** verified builds for all core programs: all core programs verified. Causes: none assigned. [verified_build, 461492305](https://solscan.io/account/Gmso1uvJnLbawvw7yezdfCDcPydwW2s2iqG3w6MDucLo)

- **UNRESOLVED_EVIDENCE** every factor could be read (Unknown caps the Stage at 0): Unknown: Oracle dependency. Causes: ORACLE_CONTROL_GAP. [authority, 461492307](https://solscan.io/account/CTDLvGGXnoxvqLyTpGzdGLg9pD6JexKxKXSV8tqqo8bN) [threshold, time_lock, members, config_authority, 461492306](https://solscan.io/account/CxnEVpQQcYa628TywzHGXeJ2jdVmbU51rnERat9xunP1) [verified_build, 461492305](https://solscan.io/account/Gmso1uvJnLbawvw7yezdfCDcPydwW2s2iqG3w6MDucLo) [transaction sample (50 transactions), 461492182](https://solscan.io/account/Gmso1uvJnLbawvw7yezdfCDcPydwW2s2iqG3w6MDucLo) [owner, 461492353](https://solscan.io/account/ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL) [upgrade_authority, 461492354](https://solscan.io/account/ARpB7g1R3hsySq5rYg4ApH5r9UMCpgAUdeWGcv9XSNwd) [owner, 461492357](https://solscan.io/account/AAL4q1SZddd2SzdSa5ZBGDPHxRcoZE5RB7kNytK2edAD) [upgrade_authority, 461492306](https://solscan.io/account/CSp2sB68THdrJP6N7Au4Y5u89VJ1E1ebaqZ2soHhQvav)

### Jito

admin_powers: **low** — the registry lists no admin accounts; the evidence is the core programs' upgrade reads

code_transparency: **medium** — source public at https://github.com/jito-foundation, but the deployed build is not verified

dependencies: **low** — all 1 callees rated Low on upgrade control

oracle_dependency: **not_applicable** — the registry lists no price feeds; the evidence is the core programs' upgrade reads

upgrade_control: **low** — Squads v3 multisig, 6 of 10 voters, time lock 0 s

upgrade_delay: **high** — time lock 0 s

- **MET** upgrade control rated Low or Medium: LOW: Squads v3 multisig, 6 of 10 voters, time lock 0 s. Causes: none assigned. [upgrade_authority, 461492361](https://solscan.io/account/EmiU8AQkB2sswTxVB6aCmsAJftoowZGGDXuytm6X65R3) [threshold, time_lock, members, config_authority, 461492361](https://solscan.io/account/3yqoHFE4nBGchuVH5rJuZMFvsmnaDTuLLdvGPDUEJcbW)

- **UNMET_VERIFIED** upgrade delay of the Stage 1 minimum, unless upgrades need an emergency multisig: SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy: effective time lock 0 s; needs 5 days+ or an emergency multisig of 8+ voters at 75%+. Causes: DELAY_BELOW_REQUIRED. [upgrade_authority, 461492361](https://solscan.io/account/EmiU8AQkB2sswTxVB6aCmsAJftoowZGGDXuytm6X65R3) [threshold, time_lock, members, config_authority, 461492361](https://solscan.io/account/3yqoHFE4nBGchuVH5rJuZMFvsmnaDTuLLdvGPDUEJcbW)

- **MET** admin powers rated Low or Medium: LOW: the registry lists no admin accounts; the evidence is the core programs' upgrade reads. Causes: none assigned. [upgrade_authority, 461492361](https://solscan.io/account/EmiU8AQkB2sswTxVB6aCmsAJftoowZGGDXuytm6X65R3) [threshold, time_lock, members, config_authority, 461492361](https://solscan.io/account/3yqoHFE4nBGchuVH5rJuZMFvsmnaDTuLLdvGPDUEJcbW)

- **UNRESOLVED_EVIDENCE** no multisig has a single-key config authority: no single-key config authority. Causes: none assigned. [upgrade_authority, 461492361](https://solscan.io/account/EmiU8AQkB2sswTxVB6aCmsAJftoowZGGDXuytm6X65R3) [threshold, time_lock, members, config_authority, 461492361](https://solscan.io/account/3yqoHFE4nBGchuVH5rJuZMFvsmnaDTuLLdvGPDUEJcbW)

- **UNRESOLVED_EVIDENCE** verified builds for all core programs: not verified: SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy. Causes: CODE_TRANSPARENCY_GAP. [verified_build, 461492360](https://solscan.io/account/SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy)

- **MET** every factor could be read (Unknown caps the Stage at 0): no Unknown factors. Causes: none assigned. [upgrade_authority, 461492361](https://solscan.io/account/EmiU8AQkB2sswTxVB6aCmsAJftoowZGGDXuytm6X65R3) [threshold, time_lock, members, config_authority, 461492361](https://solscan.io/account/3yqoHFE4nBGchuVH5rJuZMFvsmnaDTuLLdvGPDUEJcbW) [verified_build, 461492360](https://solscan.io/account/SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy) [transaction sample (49 transactions), 461492585](https://solscan.io/account/SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy) [upgrade_authority, 461492655](https://solscan.io/account/6WU8Nxarf9fudRK5atWwjLY4vFaw5UrrWhL88qz7iCMJ)

### Jupiter Lend

admin_powers: **medium** — `authority` can change parameters, held by Squads v4 multisig, 4 of 7 voters, time lock 6 h; `authority` can change parameters, held by Squads v4 multisig, 4 of 7 voters, time lock 6 h; `authority` can change parameters, held by Squads v4 multisig, 4 of 7 voters, time lock 6 h

code_transparency: **medium** — jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9: source public at https://github.com/jup-ag, but the deployed build is not verified; jupZ4m2GqUCJ5iueMfzQf8khFfH31d4XAQt3RzCT9Vd: source public at https://github.com/jup-ag, but the deployed build is not verified; jupeiUmn818Jg1ekPURTpr4mFo29p46vygyykFJ3wZC: source public at https://github.com/jup-ag, but the deployed build is not verified; jupgfSgfuAXv4B6R2Uxu85Z1qdzgju79s6MfZekN6XS: source public at https://github.com/jup-ag, but the deployed build is not verified; jupr81YtYssSyPt8jbnGuiWon5f6x9TcDEFxYe3Bdzi: source public at https://github.com/jup-ag, but the deployed build is not verified

dependencies: **low** — all 2 callees rated Low on upgrade control

oracle_dependency: **unknown** — price feeds not yet listed in the registry

upgrade_control: **low** — jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9: Squads v4 multisig, 4 of 6 voters, time lock 12 h; jupZ4m2GqUCJ5iueMfzQf8khFfH31d4XAQt3RzCT9Vd: Squads v4 multisig, 4 of 6 voters, time lock 12 h; jupeiUmn818Jg1ekPURTpr4mFo29p46vygyykFJ3wZC: Squads v4 multisig, 4 of 6 voters, time lock 12 h; jupgfSgfuAXv4B6R2Uxu85Z1qdzgju79s6MfZekN6XS: Squads v4 multisig, 4 of 6 voters, time lock 12 h; jupr81YtYssSyPt8jbnGuiWon5f6x9TcDEFxYe3Bdzi: Squads v4 multisig, 4 of 6 voters, time lock 12 h

upgrade_delay: **high** — jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9: time lock 12 h; jupZ4m2GqUCJ5iueMfzQf8khFfH31d4XAQt3RzCT9Vd: time lock 12 h; jupeiUmn818Jg1ekPURTpr4mFo29p46vygyykFJ3wZC: time lock 12 h; jupgfSgfuAXv4B6R2Uxu85Z1qdzgju79s6MfZekN6XS: time lock 12 h; jupr81YtYssSyPt8jbnGuiWon5f6x9TcDEFxYe3Bdzi: time lock 12 h

- **MET** upgrade control rated Low or Medium: LOW: jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9: Squads v4 multisig, 4 of 6 voters, time lock 12 h; jupZ4m2GqUCJ5iueMfzQf8khFfH31d4XAQt3RzCT9Vd: Squads v4 multisig, 4 of 6 voters, time lock 12 h; jupeiUmn818Jg1ekPURTpr4mFo29p46vygyykFJ3wZC: Squads v4 multisig, 4 of 6 voters, time lock 12 h; jupgfSgfuAXv4B6R2Uxu85Z1qdzgju79s6MfZekN6XS: Squads v4 multisig, 4 of 6 voters, time lock 12 h; jupr81YtYssSyPt8jbnGuiWon5f6x9TcDEFxYe3Bdzi: Squads v4 multisig, 4 of 6 voters, time lock 12 h. Causes: none assigned. [upgrade_authority, 461492698](https://solscan.io/account/3zB5SLYJoqDPN21s54MCkLeDPuMKQH9vNCSaZKSWTZZd) [threshold, time_lock, members, config_authority, 461492698](https://solscan.io/account/J3mJ3wz6xkVUk3T8qHnuAYNxsRH3ixHsryYNZAU2vG8P) [upgrade_authority, 461492698](https://solscan.io/account/E4TqU7cqaKuAmpHZt2RZ6W4zPKqbC5K5muE3M2T5ZL1S) [upgrade_authority, 461492698](https://solscan.io/account/J6Ry5yuy4Vp4kNa1PTGASiHJxZ8x4pgRPaJ18SmnqtW5) [upgrade_authority, 461492698](https://solscan.io/account/ESoMWjdS6i8pMgc75SN4s1gNotRDtNLox4uCxk6udXqY) [upgrade_authority, 461492698](https://solscan.io/account/3EgkASqSXVcgD4s3BXR9dU8DGn5usNhNbkakCKYRvkAC)

- **UNMET_VERIFIED** upgrade delay of the Stage 1 minimum, unless upgrades need an emergency multisig: jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9: effective time lock 12 h; jupZ4m2GqUCJ5iueMfzQf8khFfH31d4XAQt3RzCT9Vd: effective time lock 12 h; jupeiUmn818Jg1ekPURTpr4mFo29p46vygyykFJ3wZC: effective time lock 12 h; jupgfSgfuAXv4B6R2Uxu85Z1qdzgju79s6MfZekN6XS: effective time lock 12 h; jupr81YtYssSyPt8jbnGuiWon5f6x9TcDEFxYe3Bdzi: effective time lock 12 h; needs 5 days+ or an emergency multisig of 8+ voters at 75%+. Causes: DELAY_BELOW_REQUIRED. [upgrade_authority, 461492698](https://solscan.io/account/3zB5SLYJoqDPN21s54MCkLeDPuMKQH9vNCSaZKSWTZZd) [threshold, time_lock, members, config_authority, 461492698](https://solscan.io/account/J3mJ3wz6xkVUk3T8qHnuAYNxsRH3ixHsryYNZAU2vG8P) [upgrade_authority, 461492698](https://solscan.io/account/E4TqU7cqaKuAmpHZt2RZ6W4zPKqbC5K5muE3M2T5ZL1S) [upgrade_authority, 461492698](https://solscan.io/account/J6Ry5yuy4Vp4kNa1PTGASiHJxZ8x4pgRPaJ18SmnqtW5) [upgrade_authority, 461492698](https://solscan.io/account/ESoMWjdS6i8pMgc75SN4s1gNotRDtNLox4uCxk6udXqY) [upgrade_authority, 461492698](https://solscan.io/account/3EgkASqSXVcgD4s3BXR9dU8DGn5usNhNbkakCKYRvkAC)

- **MET** admin powers rated Low or Medium: MEDIUM: `authority` can change parameters, held by Squads v4 multisig, 4 of 7 voters, time lock 6 h; `authority` can change parameters, held by Squads v4 multisig, 4 of 7 voters, time lock 6 h; `authority` can change parameters, held by Squads v4 multisig, 4 of 7 voters, time lock 6 h. Causes: none assigned. [authority, 461492702](https://solscan.io/account/4Pgx2M7ZsDRvXU4CitoT94EJUohxqSk3eBuLsoz834dm) [threshold, time_lock, members, config_authority, 461492703](https://solscan.io/account/5Y93cxqp8rGtjDhxGkehewfhFdxLkrCfPDWookCGeASF) [authority, 461492702](https://solscan.io/account/5nmGjA4s7ATzpBQXC5RNceRpaJ7pYw2wKsNBWyuSAZV6) [authority, 461492702](https://solscan.io/account/7s1da8DduuBFqGra5bJBjpnvL5E9mGzCuMk1Qkh4or2Z)

- **MET** no multisig has a single-key config authority: no single-key config authority. Causes: none assigned. [upgrade_authority, 461492698](https://solscan.io/account/3zB5SLYJoqDPN21s54MCkLeDPuMKQH9vNCSaZKSWTZZd) [threshold, time_lock, members, config_authority, 461492698](https://solscan.io/account/J3mJ3wz6xkVUk3T8qHnuAYNxsRH3ixHsryYNZAU2vG8P) [upgrade_authority, 461492698](https://solscan.io/account/E4TqU7cqaKuAmpHZt2RZ6W4zPKqbC5K5muE3M2T5ZL1S) [upgrade_authority, 461492698](https://solscan.io/account/J6Ry5yuy4Vp4kNa1PTGASiHJxZ8x4pgRPaJ18SmnqtW5) [upgrade_authority, 461492698](https://solscan.io/account/ESoMWjdS6i8pMgc75SN4s1gNotRDtNLox4uCxk6udXqY) [upgrade_authority, 461492698](https://solscan.io/account/3EgkASqSXVcgD4s3BXR9dU8DGn5usNhNbkakCKYRvkAC) [authority, 461492702](https://solscan.io/account/4Pgx2M7ZsDRvXU4CitoT94EJUohxqSk3eBuLsoz834dm) [threshold, time_lock, members, config_authority, 461492703](https://solscan.io/account/5Y93cxqp8rGtjDhxGkehewfhFdxLkrCfPDWookCGeASF) [authority, 461492702](https://solscan.io/account/5nmGjA4s7ATzpBQXC5RNceRpaJ7pYw2wKsNBWyuSAZV6) [authority, 461492702](https://solscan.io/account/7s1da8DduuBFqGra5bJBjpnvL5E9mGzCuMk1Qkh4or2Z)

- **UNRESOLVED_EVIDENCE** verified builds for all core programs: not verified: jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9, jupZ4m2GqUCJ5iueMfzQf8khFfH31d4XAQt3RzCT9Vd, jupeiUmn818Jg1ekPURTpr4mFo29p46vygyykFJ3wZC, jupgfSgfuAXv4B6R2Uxu85Z1qdzgju79s6MfZekN6XS, jupr81YtYssSyPt8jbnGuiWon5f6x9TcDEFxYe3Bdzi. Causes: CODE_TRANSPARENCY_GAP. [verified_build, 461492697](https://solscan.io/account/jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9) [verified_build, 461492697](https://solscan.io/account/jupZ4m2GqUCJ5iueMfzQf8khFfH31d4XAQt3RzCT9Vd) [verified_build, 461492697](https://solscan.io/account/jupeiUmn818Jg1ekPURTpr4mFo29p46vygyykFJ3wZC) [verified_build, 461492697](https://solscan.io/account/jupgfSgfuAXv4B6R2Uxu85Z1qdzgju79s6MfZekN6XS) [verified_build, 461492697](https://solscan.io/account/jupr81YtYssSyPt8jbnGuiWon5f6x9TcDEFxYe3Bdzi)

- **UNRESOLVED_EVIDENCE** every factor could be read (Unknown caps the Stage at 0): Unknown: Oracle dependency. Causes: ORACLE_CONTROL_GAP. [authority, 461492702](https://solscan.io/account/4Pgx2M7ZsDRvXU4CitoT94EJUohxqSk3eBuLsoz834dm) [threshold, time_lock, members, config_authority, 461492703](https://solscan.io/account/5Y93cxqp8rGtjDhxGkehewfhFdxLkrCfPDWookCGeASF) [authority, 461492702](https://solscan.io/account/5nmGjA4s7ATzpBQXC5RNceRpaJ7pYw2wKsNBWyuSAZV6) [authority, 461492702](https://solscan.io/account/7s1da8DduuBFqGra5bJBjpnvL5E9mGzCuMk1Qkh4or2Z) [verified_build, 461492697](https://solscan.io/account/jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9) [verified_build, 461492697](https://solscan.io/account/jupZ4m2GqUCJ5iueMfzQf8khFfH31d4XAQt3RzCT9Vd) [verified_build, 461492697](https://solscan.io/account/jupeiUmn818Jg1ekPURTpr4mFo29p46vygyykFJ3wZC) [verified_build, 461492697](https://solscan.io/account/jupgfSgfuAXv4B6R2Uxu85Z1qdzgju79s6MfZekN6XS) [verified_build, 461492697](https://solscan.io/account/jupr81YtYssSyPt8jbnGuiWon5f6x9TcDEFxYe3Bdzi) [transaction sample (163 transactions), 461492797](https://solscan.io/account/jupeiUmn818Jg1ekPURTpr4mFo29p46vygyykFJ3wZC) [owner, 461492839](https://solscan.io/account/ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL) [upgrade_authority, 461492840](https://solscan.io/account/PwDiXFxQsGra4sFFTT8r1QWRMd4vfumiWC1jfWNfdYT) [upgrade_authority, 461492698](https://solscan.io/account/3zB5SLYJoqDPN21s54MCkLeDPuMKQH9vNCSaZKSWTZZd) [threshold, time_lock, members, config_authority, 461492698](https://solscan.io/account/J3mJ3wz6xkVUk3T8qHnuAYNxsRH3ixHsryYNZAU2vG8P) [upgrade_authority, 461492698](https://solscan.io/account/E4TqU7cqaKuAmpHZt2RZ6W4zPKqbC5K5muE3M2T5ZL1S) [upgrade_authority, 461492698](https://solscan.io/account/J6Ry5yuy4Vp4kNa1PTGASiHJxZ8x4pgRPaJ18SmnqtW5) [upgrade_authority, 461492698](https://solscan.io/account/ESoMWjdS6i8pMgc75SN4s1gNotRDtNLox4uCxk6udXqY) [upgrade_authority, 461492698](https://solscan.io/account/3EgkASqSXVcgD4s3BXR9dU8DGn5usNhNbkakCKYRvkAC)

### Jupiter Perps

admin_powers: **medium** — `admin` can change parameters, held by Squads v3 multisig, 4 of 7 voters, time lock 0 s

code_transparency: **medium** — source public at https://github.com/jup-ag, but the deployed build is not verified

dependencies: **low** — all 1 callees rated Low on upgrade control

oracle_dependency: **medium** — BTC custody: single oracle network (pyth), no fallback; ETH custody: single oracle network (pyth), no fallback; SOL custody: single oracle network (pyth), no fallback; USDC custody: single oracle network (pyth), no fallback; USDT custody: single oracle network (pyth), no fallback

upgrade_control: **medium** — Squads v4 multisig, 4 of 8 voters, time lock 24 h

upgrade_delay: **medium** — time lock 24 h

- **MET** upgrade control rated Low or Medium: MEDIUM: Squads v4 multisig, 4 of 8 voters, time lock 24 h. Causes: none assigned. [upgrade_authority, 461492843](https://solscan.io/account/C258LjtN8Q928yz2nJe5kwQycckGfWBSrhZBP2mTpLQt) [threshold, time_lock, members, config_authority, 461492844](https://solscan.io/account/AxkJ8oH5aDu4ZRWfsujPtxdb6Vhq4gDehpoReBgrUUSm)

- **UNMET_VERIFIED** upgrade delay of the Stage 1 minimum, unless upgrades need an emergency multisig: PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu: effective time lock 24 h; needs 5 days+ or an emergency multisig of 8+ voters at 75%+. Causes: DELAY_BELOW_REQUIRED. [upgrade_authority, 461492843](https://solscan.io/account/C258LjtN8Q928yz2nJe5kwQycckGfWBSrhZBP2mTpLQt) [threshold, time_lock, members, config_authority, 461492844](https://solscan.io/account/AxkJ8oH5aDu4ZRWfsujPtxdb6Vhq4gDehpoReBgrUUSm)

- **MET** admin powers rated Low or Medium: MEDIUM: `admin` can change parameters, held by Squads v3 multisig, 4 of 7 voters, time lock 0 s. Causes: none assigned. [admin, 461492845](https://solscan.io/account/H4ND9aYttUVLFmNypZqLjZ52FYiGvdEB45GmwNoKEjTj) [threshold, time_lock, members, config_authority, 461492846](https://solscan.io/account/7ZyDFzet6sKgZLN4D89JLfo7chu2n7nYdkFt5RCFk8Sf)

- **MET** no multisig has a single-key config authority: no single-key config authority. Causes: none assigned. [upgrade_authority, 461492843](https://solscan.io/account/C258LjtN8Q928yz2nJe5kwQycckGfWBSrhZBP2mTpLQt) [threshold, time_lock, members, config_authority, 461492844](https://solscan.io/account/AxkJ8oH5aDu4ZRWfsujPtxdb6Vhq4gDehpoReBgrUUSm) [admin, 461492845](https://solscan.io/account/H4ND9aYttUVLFmNypZqLjZ52FYiGvdEB45GmwNoKEjTj) [threshold, time_lock, members, config_authority, 461492846](https://solscan.io/account/7ZyDFzet6sKgZLN4D89JLfo7chu2n7nYdkFt5RCFk8Sf)

- **UNRESOLVED_EVIDENCE** verified builds for all core programs: not verified: PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu. Causes: CODE_TRANSPARENCY_GAP. [verified_build, 461492842](https://solscan.io/account/PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu)

- **MET** every factor could be read (Unknown caps the Stage at 0): no Unknown factors. Causes: none assigned. [admin, 461492845](https://solscan.io/account/H4ND9aYttUVLFmNypZqLjZ52FYiGvdEB45GmwNoKEjTj) [threshold, time_lock, members, config_authority, 461492846](https://solscan.io/account/7ZyDFzet6sKgZLN4D89JLfo7chu2n7nYdkFt5RCFk8Sf) [verified_build, 461492842](https://solscan.io/account/PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu) [transaction sample (37 transactions), 461492827](https://solscan.io/account/PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu) [owner, 461492881](https://solscan.io/account/ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL) [owner, 461492846](https://solscan.io/account/42amVS4KgzR9rA28tkVYqVXjq9Qa8dcZQMbH5EYFX6XC) [owner, 461492846](https://solscan.io/account/4cSM2e6rvbGQUFiJbqytoVMi5GgghSMr8LwVrT9VPSPo) [owner, 461492846](https://solscan.io/account/7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE) [owner, 461492846](https://solscan.io/account/Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX) [owner, 461492846](https://solscan.io/account/HT2PLQBcG5EiCcNSaMHAjSgd9F98ecpATbk4Sk5oYuM) [upgrade_authority, 461492843](https://solscan.io/account/C258LjtN8Q928yz2nJe5kwQycckGfWBSrhZBP2mTpLQt) [threshold, time_lock, members, config_authority, 461492844](https://solscan.io/account/AxkJ8oH5aDu4ZRWfsujPtxdb6Vhq4gDehpoReBgrUUSm)

### Jupiter Staked SOL

admin_powers: **low** — the registry lists no admin accounts; the evidence is the core programs' upgrade reads

code_transparency: **medium** — source public at https://github.com/jup-ag, but the deployed build is not verified

dependencies: **low** — all 1 callees rated Low on upgrade control

oracle_dependency: **not_applicable** — the registry lists no price feeds; the evidence is the core programs' upgrade reads

upgrade_control: **medium** — Squads v3 multisig, 6 of 11 voters, time lock 0 s

upgrade_delay: **high** — time lock 0 s

- **MET** upgrade control rated Low or Medium: MEDIUM: Squads v3 multisig, 6 of 11 voters, time lock 0 s. Causes: none assigned. [upgrade_authority, 461492897](https://solscan.io/account/HxBTMuB7cFBPVWVJjTi9iBF8MPd7mfY1QnrrWfLAySFd) [threshold, time_lock, members, config_authority, 461492898](https://solscan.io/account/AApfiPZgV5MoPU691GwhdDhq5sKEMMH1Uh8S4Z9xvP6b)

- **UNMET_VERIFIED** upgrade delay of the Stage 1 minimum, unless upgrades need an emergency multisig: SPMBzsVUuoHA4Jm6KunbsotaahvVikZs1JyTW6iJvbn: effective time lock 0 s; needs 5 days+ or an emergency multisig of 8+ voters at 75%+. Causes: DELAY_BELOW_REQUIRED. [upgrade_authority, 461492897](https://solscan.io/account/HxBTMuB7cFBPVWVJjTi9iBF8MPd7mfY1QnrrWfLAySFd) [threshold, time_lock, members, config_authority, 461492898](https://solscan.io/account/AApfiPZgV5MoPU691GwhdDhq5sKEMMH1Uh8S4Z9xvP6b)

- **MET** admin powers rated Low or Medium: LOW: the registry lists no admin accounts; the evidence is the core programs' upgrade reads. Causes: none assigned. [upgrade_authority, 461492897](https://solscan.io/account/HxBTMuB7cFBPVWVJjTi9iBF8MPd7mfY1QnrrWfLAySFd) [threshold, time_lock, members, config_authority, 461492898](https://solscan.io/account/AApfiPZgV5MoPU691GwhdDhq5sKEMMH1Uh8S4Z9xvP6b)

- **UNRESOLVED_EVIDENCE** no multisig has a single-key config authority: no single-key config authority. Causes: none assigned. [upgrade_authority, 461492897](https://solscan.io/account/HxBTMuB7cFBPVWVJjTi9iBF8MPd7mfY1QnrrWfLAySFd) [threshold, time_lock, members, config_authority, 461492898](https://solscan.io/account/AApfiPZgV5MoPU691GwhdDhq5sKEMMH1Uh8S4Z9xvP6b)

- **UNRESOLVED_EVIDENCE** verified builds for all core programs: not verified: SPMBzsVUuoHA4Jm6KunbsotaahvVikZs1JyTW6iJvbn. Causes: CODE_TRANSPARENCY_GAP. [verified_build, 461492896](https://solscan.io/account/SPMBzsVUuoHA4Jm6KunbsotaahvVikZs1JyTW6iJvbn)

- **MET** every factor could be read (Unknown caps the Stage at 0): no Unknown factors. Causes: none assigned. [upgrade_authority, 461492897](https://solscan.io/account/HxBTMuB7cFBPVWVJjTi9iBF8MPd7mfY1QnrrWfLAySFd) [threshold, time_lock, members, config_authority, 461492898](https://solscan.io/account/AApfiPZgV5MoPU691GwhdDhq5sKEMMH1Uh8S4Z9xvP6b) [verified_build, 461492896](https://solscan.io/account/SPMBzsVUuoHA4Jm6KunbsotaahvVikZs1JyTW6iJvbn) [transaction sample (48 transactions), 461489904](https://solscan.io/account/SPMBzsVUuoHA4Jm6KunbsotaahvVikZs1JyTW6iJvbn) [upgrade_authority, 461493215](https://solscan.io/account/6WU8Nxarf9fudRK5atWwjLY4vFaw5UrrWhL88qz7iCMJ)

### Kamino Lend

admin_powers: **high** — `lending_market_owner` can change parameters, held by Squads v4 multisig, 4 of 10 voters, time lock 12 h

code_transparency: **low** — verified build of https://github.com/Kamino-Finance/klend/tree/e921c648765fc2343c87614a00f4b39b640f0510 at e921c648765fc2343c87614a00f4b39b640f0510

dependencies: **low** — calls no other programs with user funds

oracle_dependency: **high** — SOL reserve: prices come from a protocol-run feed

upgrade_control: **medium** — Squads v4 multisig, 5 of 10 voters, time lock 24 h

upgrade_delay: **medium** — time lock 24 h

- **MET** upgrade control rated Low or Medium: MEDIUM: Squads v4 multisig, 5 of 10 voters, time lock 24 h. Causes: none assigned. [upgrade_authority, 461493217](https://solscan.io/account/9uSbGW1y9H5Av6H5TKxQ1wnFApSq2t3oEpfF2YfjDQGA) [threshold, time_lock, members, config_authority, 461493219](https://solscan.io/account/6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM)

- **UNMET_VERIFIED** upgrade delay of the Stage 1 minimum, unless upgrades need an emergency multisig: KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD: effective time lock 24 h; needs 5 days+ or an emergency multisig of 8+ voters at 75%+. Causes: DELAY_BELOW_REQUIRED. [upgrade_authority, 461493217](https://solscan.io/account/9uSbGW1y9H5Av6H5TKxQ1wnFApSq2t3oEpfF2YfjDQGA) [threshold, time_lock, members, config_authority, 461493219](https://solscan.io/account/6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM)

- **UNMET_VERIFIED** admin powers rated Low or Medium: HIGH: `lending_market_owner` can change parameters, held by Squads v4 multisig, 4 of 10 voters, time lock 12 h. Causes: PRIVILEGED_ADMIN_POWER, VOTER_THRESHOLD_BELOW_REQUIRED. [lending_market_owner, 461493225](https://solscan.io/account/7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF) [threshold, time_lock, members, config_authority, 461493225](https://solscan.io/account/7idEEVRidWrahZJhxXMqniDbV6ESj7ZjyLrigcMcEt6H)

- **MET** no multisig has a single-key config authority: no single-key config authority. Causes: none assigned. [upgrade_authority, 461493217](https://solscan.io/account/9uSbGW1y9H5Av6H5TKxQ1wnFApSq2t3oEpfF2YfjDQGA) [threshold, time_lock, members, config_authority, 461493219](https://solscan.io/account/6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM) [lending_market_owner, 461493225](https://solscan.io/account/7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF) [threshold, time_lock, members, config_authority, 461493225](https://solscan.io/account/7idEEVRidWrahZJhxXMqniDbV6ESj7ZjyLrigcMcEt6H)

- **MET** verified builds for all core programs: all core programs verified. Causes: none assigned. [verified_build, 461493215](https://solscan.io/account/KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD)

- **MET** every factor could be read (Unknown caps the Stage at 0): no Unknown factors. Causes: none assigned. [lending_market_owner, 461493225](https://solscan.io/account/7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF) [threshold, time_lock, members, config_authority, 461493225](https://solscan.io/account/7idEEVRidWrahZJhxXMqniDbV6ESj7ZjyLrigcMcEt6H) [verified_build, 461493215](https://solscan.io/account/KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD) [transaction sample (34 transactions), 461493210](https://solscan.io/account/KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD) [owner, 461493226](https://solscan.io/account/3t4JZcueEzTbVP6kLxXrL3VpWx45jDer4eqysweBchNH) [upgrade_authority, 461493217](https://solscan.io/account/9uSbGW1y9H5Av6H5TKxQ1wnFApSq2t3oEpfF2YfjDQGA) [threshold, time_lock, members, config_authority, 461493219](https://solscan.io/account/6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM)

### Orca

admin_powers: **low** — `fee_authority` can change parameters behind a 24 h time lock

code_transparency: **medium** — source public at https://github.com/orca-so/whirlpools, but the deployed build is not verified

dependencies: **low** — all 1 callees rated Low on upgrade control

oracle_dependency: **not_applicable** — the registry lists no price feeds; the evidence is the core programs' upgrade reads

upgrade_control: **medium** — Squads v4 multisig, 5 of 9 voters, time lock 24 h

upgrade_delay: **medium** — time lock 24 h

- **MET** upgrade control rated Low or Medium: MEDIUM: Squads v4 multisig, 5 of 9 voters, time lock 24 h. Causes: none assigned. [upgrade_authority, 461493272](https://solscan.io/account/CtXfPzz36dH5Ws4UYKZvrQ1Xqzn42ecDW6y8NKuiN8nD) [threshold, time_lock, members, config_authority, 461493273](https://solscan.io/account/BQsDWkL417U4tVE2sDnPks469pKdm6YzFgKH77doiEjF)

- **UNMET_VERIFIED** upgrade delay of the Stage 1 minimum, unless upgrades need an emergency multisig: whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc: effective time lock 24 h; needs 5 days+ or an emergency multisig of 8+ voters at 75%+. Causes: DELAY_BELOW_REQUIRED. [upgrade_authority, 461493272](https://solscan.io/account/CtXfPzz36dH5Ws4UYKZvrQ1Xqzn42ecDW6y8NKuiN8nD) [threshold, time_lock, members, config_authority, 461493273](https://solscan.io/account/BQsDWkL417U4tVE2sDnPks469pKdm6YzFgKH77doiEjF)

- **MET** admin powers rated Low or Medium: LOW: `fee_authority` can change parameters behind a 24 h time lock. Causes: none assigned. [fee_authority, 461493552](https://solscan.io/account/2LecshUwdy9xi7meFgHtFJQNSKk4KdTrcpvaB56dP2NQ) [threshold, time_lock, members, config_authority, 461493273](https://solscan.io/account/BQsDWkL417U4tVE2sDnPks469pKdm6YzFgKH77doiEjF)

- **MET** no multisig has a single-key config authority: no single-key config authority. Causes: none assigned. [upgrade_authority, 461493272](https://solscan.io/account/CtXfPzz36dH5Ws4UYKZvrQ1Xqzn42ecDW6y8NKuiN8nD) [threshold, time_lock, members, config_authority, 461493273](https://solscan.io/account/BQsDWkL417U4tVE2sDnPks469pKdm6YzFgKH77doiEjF) [fee_authority, 461493552](https://solscan.io/account/2LecshUwdy9xi7meFgHtFJQNSKk4KdTrcpvaB56dP2NQ)

- **UNRESOLVED_EVIDENCE** verified builds for all core programs: not verified: whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc. Causes: CODE_TRANSPARENCY_GAP. [verified_build, 461493271](https://solscan.io/account/whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc)

- **MET** every factor could be read (Unknown caps the Stage at 0): no Unknown factors. Causes: none assigned. [fee_authority, 461493552](https://solscan.io/account/2LecshUwdy9xi7meFgHtFJQNSKk4KdTrcpvaB56dP2NQ) [threshold, time_lock, members, config_authority, 461493273](https://solscan.io/account/BQsDWkL417U4tVE2sDnPks469pKdm6YzFgKH77doiEjF) [verified_build, 461493271](https://solscan.io/account/whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc) [transaction sample (45 transactions), 461493553](https://solscan.io/account/whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc) [owner, 461493595](https://solscan.io/account/ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL) [upgrade_authority, 461493272](https://solscan.io/account/CtXfPzz36dH5Ws4UYKZvrQ1Xqzn42ecDW6y8NKuiN8nD)

### PumpSwap

admin_powers: **medium** — `admin` can pause, held by Squads v4 multisig, 3 of 4 voters, time lock 0 s

code_transparency: **medium** — source public at https://github.com/pump-fun, but the deployed build is not verified

dependencies: **medium** — callee pfee…ojVZ is Squads v4 multisig, 3 of 4 voters, time lock 0 s

oracle_dependency: **not_applicable** — the registry lists no price feeds; the evidence is the core programs' upgrade reads

upgrade_control: **medium** — Squads v4 multisig, 3 of 4 voters, time lock 0 s

upgrade_delay: **high** — time lock 0 s

- **MET** upgrade control rated Low or Medium: MEDIUM: Squads v4 multisig, 3 of 4 voters, time lock 0 s. Causes: none assigned. [upgrade_authority, 461493612](https://solscan.io/account/6naEzKeUuFh1Jeeu51NXQgr5qkXgXtc9WKNct4xynVJc) [threshold, time_lock, members, config_authority, 461493613](https://solscan.io/account/2yMoQqQrtbhq3nQ3wFoQQawWS65qcqUXcwHEYha4rshW)

- **UNMET_VERIFIED** upgrade delay of the Stage 1 minimum, unless upgrades need an emergency multisig: pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA: effective time lock 0 s; needs 5 days+ or an emergency multisig of 8+ voters at 75%+. Causes: DELAY_BELOW_REQUIRED. [upgrade_authority, 461493612](https://solscan.io/account/6naEzKeUuFh1Jeeu51NXQgr5qkXgXtc9WKNct4xynVJc) [threshold, time_lock, members, config_authority, 461493613](https://solscan.io/account/2yMoQqQrtbhq3nQ3wFoQQawWS65qcqUXcwHEYha4rshW)

- **MET** admin powers rated Low or Medium: MEDIUM: `admin` can pause, held by Squads v4 multisig, 3 of 4 voters, time lock 0 s. Causes: none assigned. [admin, 461493884](https://solscan.io/account/ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw) [threshold, time_lock, members, config_authority, 461493886](https://solscan.io/account/8EujzwPFNhZ5m4Kpa2XtrumNgwCbhgWwny9YWLCKA5at)

- **MET** no multisig has a single-key config authority: no single-key config authority. Causes: none assigned. [upgrade_authority, 461493612](https://solscan.io/account/6naEzKeUuFh1Jeeu51NXQgr5qkXgXtc9WKNct4xynVJc) [threshold, time_lock, members, config_authority, 461493613](https://solscan.io/account/2yMoQqQrtbhq3nQ3wFoQQawWS65qcqUXcwHEYha4rshW) [admin, 461493884](https://solscan.io/account/ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw) [threshold, time_lock, members, config_authority, 461493886](https://solscan.io/account/8EujzwPFNhZ5m4Kpa2XtrumNgwCbhgWwny9YWLCKA5at)

- **UNRESOLVED_EVIDENCE** verified builds for all core programs: not verified: pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA. Causes: CODE_TRANSPARENCY_GAP. [verified_build, 461493611](https://solscan.io/account/pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA)

- **MET** every factor could be read (Unknown caps the Stage at 0): no Unknown factors. Causes: none assigned. [admin, 461493884](https://solscan.io/account/ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw) [threshold, time_lock, members, config_authority, 461493886](https://solscan.io/account/8EujzwPFNhZ5m4Kpa2XtrumNgwCbhgWwny9YWLCKA5at) [verified_build, 461493611](https://solscan.io/account/pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA) [transaction sample (40 transactions), 461493887](https://solscan.io/account/pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA) [upgrade_authority, 461493926](https://solscan.io/account/75Uu23mqWBb8LM8vDppqC1mQAnCcBuLXhVaDezVMQLRw) [threshold, time_lock, members, config_authority, 461493613](https://solscan.io/account/2yMoQqQrtbhq3nQ3wFoQQawWS65qcqUXcwHEYha4rshW) [upgrade_authority, 461493612](https://solscan.io/account/6naEzKeUuFh1Jeeu51NXQgr5qkXgXtc9WKNct4xynVJc)

### Raydium

admin_powers: **high** — `owner` can change parameters, held by single key proj…k416

code_transparency: **medium** — 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8: source public at https://github.com/raydium-io/raydium-amm, but the deployed build is not verified; CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK: source public at https://github.com/raydium-io/raydium-clmm/tree/119ed36bf9f33, but the deployed build is not verified; CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C: source public at https://github.com/raydium-io/raydium-cp-swap/tree/cfdb70a8ca9ea62bb5c304d4492ac0fc371ae8ce, but the deployed build is not verified

dependencies: **low** — all 1 callees rated Low on upgrade control

oracle_dependency: **not_applicable** — the registry lists no price feeds; the evidence is the core programs' upgrade reads

upgrade_control: **medium** — 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8: Squads v4 multisig, 3 of 4 voters, time lock 0 s; CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK: Squads v4 multisig, 3 of 4 voters, time lock 0 s; CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C: Squads v4 multisig, 3 of 4 voters, time lock 0 s

upgrade_delay: **high** — 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8: time lock 0 s; CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK: time lock 0 s; CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C: time lock 0 s

- **MET** upgrade control rated Low or Medium: MEDIUM: 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8: Squads v4 multisig, 3 of 4 voters, time lock 0 s; CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK: Squads v4 multisig, 3 of 4 voters, time lock 0 s; CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C: Squads v4 multisig, 3 of 4 voters, time lock 0 s. Causes: none assigned. [upgrade_authority, 461493960](https://solscan.io/account/A7ZG7ByDi8DpzT9Ab7CiXhvgYTJQmaDPJkMDoPitaCQV) [threshold, time_lock, members, config_authority, 461493961](https://solscan.io/account/tr8rgazUrZzgdkfc6Q622nVJHMMzh29trdBE2uBHb4u) [upgrade_authority, 461493960](https://solscan.io/account/HzD2cCXXT3UQNjMMY6kDv9w6gZ9qquSdfoGXrLL3LXx) [upgrade_authority, 461493960](https://solscan.io/account/DMawCQzbgNTmbzaESc7o6pvL1KAeetY8zA7jNpzntHhU)

- **UNMET_VERIFIED** upgrade delay of the Stage 1 minimum, unless upgrades need an emergency multisig: 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8: effective time lock 0 s; CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK: effective time lock 0 s; CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C: effective time lock 0 s; needs 5 days+ or an emergency multisig of 8+ voters at 75%+. Causes: DELAY_BELOW_REQUIRED. [upgrade_authority, 461493960](https://solscan.io/account/A7ZG7ByDi8DpzT9Ab7CiXhvgYTJQmaDPJkMDoPitaCQV) [threshold, time_lock, members, config_authority, 461493961](https://solscan.io/account/tr8rgazUrZzgdkfc6Q622nVJHMMzh29trdBE2uBHb4u) [upgrade_authority, 461493960](https://solscan.io/account/HzD2cCXXT3UQNjMMY6kDv9w6gZ9qquSdfoGXrLL3LXx) [upgrade_authority, 461493960](https://solscan.io/account/DMawCQzbgNTmbzaESc7o6pvL1KAeetY8zA7jNpzntHhU)

- **UNMET_VERIFIED** admin powers rated Low or Medium: HIGH: `owner` can change parameters, held by single key proj…k416. Causes: PRIVILEGED_ADMIN_POWER, SINGLE_EFFECTIVE_KEY. [owner, 461494791](https://solscan.io/account/4BLNHtVe942GSs4teSZqGX24xwKNkqU7bGgNn3iUiUpw)

- **MET** no multisig has a single-key config authority: no single-key config authority. Causes: none assigned. [upgrade_authority, 461493960](https://solscan.io/account/A7ZG7ByDi8DpzT9Ab7CiXhvgYTJQmaDPJkMDoPitaCQV) [threshold, time_lock, members, config_authority, 461493961](https://solscan.io/account/tr8rgazUrZzgdkfc6Q622nVJHMMzh29trdBE2uBHb4u) [upgrade_authority, 461493960](https://solscan.io/account/HzD2cCXXT3UQNjMMY6kDv9w6gZ9qquSdfoGXrLL3LXx) [upgrade_authority, 461493960](https://solscan.io/account/DMawCQzbgNTmbzaESc7o6pvL1KAeetY8zA7jNpzntHhU) [owner, 461494791](https://solscan.io/account/4BLNHtVe942GSs4teSZqGX24xwKNkqU7bGgNn3iUiUpw)

- **UNRESOLVED_EVIDENCE** verified builds for all core programs: not verified: 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8, CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK, CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C. Causes: CODE_TRANSPARENCY_GAP. [verified_build, 461493959](https://solscan.io/account/675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8) [verified_build, 461493959](https://solscan.io/account/CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK) [verified_build, 461493959](https://solscan.io/account/CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C)

- **MET** every factor could be read (Unknown caps the Stage at 0): no Unknown factors. Causes: none assigned. [owner, 461494791](https://solscan.io/account/4BLNHtVe942GSs4teSZqGX24xwKNkqU7bGgNn3iUiUpw) [verified_build, 461493959](https://solscan.io/account/675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8) [verified_build, 461493959](https://solscan.io/account/CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK) [verified_build, 461493959](https://solscan.io/account/CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C) [transaction sample (113 transactions), 461494871](https://solscan.io/account/675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8) [owner, 461494905](https://solscan.io/account/ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL) [upgrade_authority, 461493960](https://solscan.io/account/A7ZG7ByDi8DpzT9Ab7CiXhvgYTJQmaDPJkMDoPitaCQV) [threshold, time_lock, members, config_authority, 461493961](https://solscan.io/account/tr8rgazUrZzgdkfc6Q622nVJHMMzh29trdBE2uBHb4u) [upgrade_authority, 461493960](https://solscan.io/account/HzD2cCXXT3UQNjMMY6kDv9w6gZ9qquSdfoGXrLL3LXx) [upgrade_authority, 461493960](https://solscan.io/account/DMawCQzbgNTmbzaESc7o6pvL1KAeetY8zA7jNpzntHhU)

## Separate blocker analyses

### Verified blockers

- s1.upgrade_delay: 10/10; protocols: drift-staked-sol, gmx-solana, jito, jupiter-lend, jupiter-perps, jupiter-staked-sol, kamino-lend, orca, pumpswap, raydium-amm
- s0.public_source: 2/2; protocols: binance-staked-sol, forward-sol
- s1.admin_powers: 2/10; protocols: kamino-lend, raydium-amm

### Unresolved evidence

- s1.verified_builds: 8/10; protocols: drift-staked-sol, jito, jupiter-lend, jupiter-perps, jupiter-staked-sol, orca, pumpswap, raydium-amm
- s1.config_authority: 3/10; protocols: drift-staked-sol, jito, jupiter-staked-sol
- s1.no_unknown: 2/10; protocols: gmx-solana, jupiter-lend

### Mixed cases

- drift-staked-sol: both verified unmet and unresolved next-level criteria (see source trace).
- gmx-solana: both verified unmet and unresolved next-level criteria (see source trace).
- jito: both verified unmet and unresolved next-level criteria (see source trace).
- jupiter-lend: both verified unmet and unresolved next-level criteria (see source trace).
- jupiter-perps: both verified unmet and unresolved next-level criteria (see source trace).
- jupiter-staked-sol: both verified unmet and unresolved next-level criteria (see source trace).
- orca: both verified unmet and unresolved next-level criteria (see source trace).
- pumpswap: both verified unmet and unresolved next-level criteria (see source trace).
- raydium-amm: both verified unmet and unresolved next-level criteria (see source trace).

## All status counts

| Protocol | Factor state counts | Criterion status counts |
|---|---|---|
| binance-staked-sol | {"low":3,"high":2,"not_applicable":1} | {"MET":2,"UNMET_VERIFIED":1} |
| drift-staked-sol | {"low":2,"medium":2,"not_applicable":1,"high":1} | {"MET":3,"UNMET_VERIFIED":1,"UNRESOLVED_EVIDENCE":2} |
| forward-sol | {"low":2,"high":2,"not_applicable":1,"medium":1} | {"MET":2,"UNMET_VERIFIED":1} |
| gmx-solana | {"medium":2,"low":1,"high":2,"unknown":1} | {"MET":4,"UNMET_VERIFIED":1,"UNRESOLVED_EVIDENCE":1} |
| jito | {"low":3,"medium":1,"not_applicable":1,"high":1} | {"MET":3,"UNMET_VERIFIED":1,"UNRESOLVED_EVIDENCE":2} |
| jupiter-lend | {"medium":2,"low":2,"unknown":1,"high":1} | {"MET":3,"UNMET_VERIFIED":1,"UNRESOLVED_EVIDENCE":2} |
| jupiter-perps | {"medium":5,"low":1} | {"MET":4,"UNMET_VERIFIED":1,"UNRESOLVED_EVIDENCE":1} |
| jupiter-staked-sol | {"low":2,"medium":2,"not_applicable":1,"high":1} | {"MET":3,"UNMET_VERIFIED":1,"UNRESOLVED_EVIDENCE":2} |
| kamino-lend | {"high":2,"low":2,"medium":2} | {"MET":4,"UNMET_VERIFIED":2} |
| orca | {"low":2,"medium":3,"not_applicable":1} | {"MET":4,"UNMET_VERIFIED":1,"UNRESOLVED_EVIDENCE":1} |
| pumpswap | {"medium":4,"not_applicable":1,"high":1} | {"MET":4,"UNMET_VERIFIED":1,"UNRESOLVED_EVIDENCE":1} |
| raydium-amm | {"high":2,"medium":2,"low":1,"not_applicable":1} | {"MET":3,"UNMET_VERIFIED":2,"UNRESOLVED_EVIDENCE":1} |

## Distinct-protocol cause counts and representative paths

- **CODE_TRANSPARENCY_GAP**: 10 distinct protocols (binance-staked-sol, drift-staked-sol, forward-sol, jito, jupiter-lend, jupiter-perps, jupiter-staked-sol, orca, pumpswap, raydium-amm). Example binance-staked-sol/s0.public_source, account SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy: {"kind":"no_public_source"}. Evidence IDs: ac46893bf1d9cb822d8a6081e92dbdc337ad18e537c9f618e7f34de50ff20634. Full controller tree/voting permissions/delay/configuration path is in this protocol's authority_roots in the diagnostic JSON.
- **DELAY_BELOW_REQUIRED**: 10 distinct protocols (drift-staked-sol, gmx-solana, jito, jupiter-lend, jupiter-perps, jupiter-staked-sol, kamino-lend, orca, pumpswap, raydium-amm). Example drift-staked-sol/s1.upgrade_delay, account SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY: Effective delay 0 seconds; SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY: effective time lock 0 s; needs 5 days+ or an emergency multisig of 8+ voters at 75%+. Evidence IDs: 08b284a141bf340f03b5c08f252d37d805378981d15b63a13b648694ace3b05a, c0074128529d138c8669f90ad6751e4622d8a7842710f138de24396a8f32b6ac. Full controller tree/voting permissions/delay/configuration path is in this protocol's authority_roots in the diagnostic JSON.
- **ORACLE_CONTROL_GAP**: 2 distinct protocols (gmx-solana, jupiter-lend). Example gmx-solana/s1.no_unknown, account CSp2sB68THdrJP6N7Au4Y5u89VJ1E1ebaqZ2soHhQvav: price feeds not yet listed in the registry. Evidence IDs: c4be90a2d37a6f962e8a8d64ed6b4382ff892f8068430f340e32c4c10f1ef9bd. Full controller tree/voting permissions/delay/configuration path is in this protocol's authority_roots in the diagnostic JSON.
- **PRIVILEGED_ADMIN_POWER**: 2 distinct protocols (kamino-lend, raydium-amm). Example kamino-lend/s1.admin_powers, account 7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF.lending_market_owner: parameters: lending_market_owner. Evidence IDs: 60c9bcc35d8aa126c18c632abb424fbd7ea974b59b8009761578c2583c7db882, ed1456caeb743483f23084c742d596aaacf06de2808e5963eeef54ff3c4819b2. Full controller tree/voting permissions/delay/configuration path is in this protocol's authority_roots in the diagnostic JSON.
- **SINGLE_EFFECTIVE_KEY**: 1 distinct protocols (raydium-amm). Example raydium-amm/s1.admin_powers, account 4BLNHtVe942GSs4teSZqGX24xwKNkqU7bGgNn3iUiUpw.owner: {"key":"projjosVCPQH49d5em7VYS7fJZzaqKixqKtus7yk416","kind":"single_key"}. Evidence IDs: 42ce8e27e83f7dcc23f9bd644407c0d2450a57bd813c1026a2ea5a70e0241be5. Full controller tree/voting permissions/delay/configuration path is in this protocol's authority_roots in the diagnostic JSON.
- **VOTER_THRESHOLD_BELOW_REQUIRED**: 1 distinct protocols (kamino-lend). Example kamino-lend/s1.admin_powers, account 7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF.lending_market_owner: {"address":"7idEEVRidWrahZJhxXMqniDbV6ESj7ZjyLrigcMcEt6H","config_authority":null,"kind":"multisig","multisig_kind":"squads_v4","signers":["2RJJ1ZZpeQLm6HEQ1HtcrGJUzQa8hKQgjgDf7ANz9Rd5","5ggs2vd1csz74YMCzxesmcUEd3ycYFKAW6kfyttVYwBv","6vkzAQLAERW5RKHX68ZCDxWxW77T8dMqYUYMXCe6ooMC","7tH1k4PsMu3sNUYJxD5ezxhAmYYXyVVZz5c3dbaxcvUV","BEUTwDjLNoXvT26J9CZD5A1Fsr9WF2UXdcUVZuy4zQ3d","BMGYmN1hG8evD3pvy1Rk41bFUdGGVP5fYJbh2JHMonqK","CmJywvWynJAiADsgTkVWu5BJus4ru2SP1PnfBXvThrxZ","DFheUjWj6BL9CuF8gAYfF6TgrbrruTTCc3vE6Ajn1gyK","G8fVffcHt3n6DkjSdYj47xWe55QjFsL4P5zhQ9RXFuPa","HrKa158AuSt2QtDWnvr81qdgBZjJ7MJGkaQ8D7pZeYMj"],"threshold":4,"time_lock_secs":43200,"voters":10}. Evidence IDs: 60c9bcc35d8aa126c18c632abb424fbd7ea974b59b8009761578c2583c7db882, ed1456caeb743483f23084c742d596aaacf06de2808e5963eeef54ff3c4819b2. Full controller tree/voting permissions/delay/configuration path is in this protocol's authority_roots in the diagnostic JSON.

The current canonical Stage rubric has no per-criterion exemption field; factor N/A is not a criterion exemption. NOT_APPLICABLE criterion cases are therefore NOT_OBSERVED, not fabricated. Stage 1 and Stage 2 counts are zero. Counterfactual sensitivity: BLOCKED_UNSUPPORTED; the structural criterion frequencies above are the supported alternative. Native HyperEVM and HIP-3 do not enter these denominators.

## Limitations and anomalies

- Hosted v0.2 differs from the owner-expected v0.3 candidate.
- Replay reconstructs only publicly exported facts; null metrics are absent, not zero. HIP-3 raw JSON is unavailable.
- Criterion classifications are review annotations, never alternate Stage assignments.
- Missing binary verification is unresolved evidence, not proof of a binary mismatch.
- Unrated review next_stage is null per contract; published_next_stage retains Stage 0. Stage 0 entry criteria remain visible.
- Counterfactual overrides: BLOCKED_UNSUPPORTED; no safe criterion-override replay API. Structural frequencies follow instead.

- drift-staked-sol s1.config_authority: CAUSE_CLASSIFICATION_UNRESOLVED; No source-supported cause can be assigned; no guessed catch-all.
- jito s1.config_authority: CAUSE_CLASSIFICATION_UNRESOLVED; No source-supported cause can be assigned; no guessed catch-all.
- jupiter-staked-sol s1.config_authority: CAUSE_CLASSIFICATION_UNRESOLVED; No source-supported cause can be assigned; no guessed catch-all.
