//! Shared helpers: a throwaway registry directory on disk.
#![allow(dead_code)]

use std::path::Path;

use tempfile::TempDir;

pub const VALID: &str = r#"
id: kamino-lend
name: Kamino Lend
chain: solana
category: lending
website: https://kamino.finance
defillama_slug: kamino-lend
programs:
  - id: KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD
    name: Lending
    role: core
    source: https://github.com/Kamino-Finance/klend
  - id: FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr
    name: Farms
    role: periphery
    source: https://github.com/Kamino-Finance/kfarms
multisigs:
  - address: 6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM
    program: squads-v4
    vault_index: 0
    source: https://docs.kamino.finance/
admin_accounts:
  - account: 7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF
    idl: idl/kamino-lend.json
    field: lending_market_owner
    power: parameters
    source: https://github.com/Kamino-Finance/klend
oracles:
  - account: 7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE
    market: SOL reserve
    role: primary
    source: https://docs.kamino.finance/
review:
  reviewed_by: andreikiba
  last_reviewed: 2026-11-02
"#;

pub const ORIGINS: &str = r#"
kamino-lend.json:
  source: https://github.com/Kamino-Finance/klend/blob/abc123/idl.json
  commit: abc123
"#;

pub struct Registry {
    pub dir: TempDir,
}

impl Registry {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("solana")).unwrap();
        std::fs::create_dir_all(dir.path().join("idl")).unwrap();
        Self { dir }
    }

    /// A registry holding `VALID` plus its IDL and origin record.
    pub fn valid() -> Self {
        let r = Self::new();
        r.protocol("kamino-lend", VALID);
        r.write("idl/kamino-lend.json", "{}");
        r.write("idl/origins.yaml", ORIGINS);
        r
    }

    pub fn protocol(&self, id: &str, yaml: &str) {
        self.write(&format!("solana/{id}.yaml"), yaml);
    }

    pub fn write(&self, rel: &str, text: &str) {
        std::fs::write(self.dir.path().join(rel), text).unwrap();
    }

    pub fn remove(&self, rel: &str) {
        std::fs::remove_file(self.dir.path().join(rel)).unwrap();
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }
}
