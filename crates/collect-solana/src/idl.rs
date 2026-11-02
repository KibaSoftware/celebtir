//! Just enough of Anchor IDLs to read one field out of a state account
//! (COL-11). Handles the legacy format (`publicKey`, `defined: "Name"`, no
//! stored discriminators) and the 0.30 format (`pubkey`, `defined: {name}`,
//! explicit discriminators). Layout is Borsh: fields in order, no padding;
//! variable-length fields are sized from the data itself.

use std::collections::BTreeMap;

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::program::DecodeError;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Ty {
    Fixed(usize),
    Pubkey,
    String,
    Bytes,
    Array(Box<Ty>, usize),
    Vec(Box<Ty>),
    Option(Box<Ty>),
    Defined(String),
}

#[derive(Debug, Clone)]
enum TypeDef {
    Struct(Vec<(String, Ty)>),
    /// Variants: name and field types (empty for unit variants).
    Enum(Vec<(String, Vec<Ty>)>),
}

#[derive(Debug, Clone)]
pub struct Idl {
    /// Account type name → discriminator.
    accounts: Vec<(String, [u8; 8])>,
    types: BTreeMap<String, TypeDef>,
}

fn bad(msg: impl Into<String>) -> DecodeError {
    DecodeError(msg.into())
}

/// `lendingMarketOwner`, `lending_market_owner` and `LendingMarketOwner` match.
fn same_name(a: &str, b: &str) -> bool {
    let norm = |s: &str| {
        s.chars()
            .filter(|c| *c != '_')
            .flat_map(char::to_lowercase)
            .collect::<String>()
    };
    norm(a) == norm(b)
}

fn parse_ty(v: &Value) -> Result<Ty, DecodeError> {
    if let Some(s) = v.as_str() {
        return Ok(match s {
            "bool" | "u8" | "i8" => Ty::Fixed(1),
            "u16" | "i16" => Ty::Fixed(2),
            "u32" | "i32" | "f32" => Ty::Fixed(4),
            "u64" | "i64" | "f64" => Ty::Fixed(8),
            "u128" | "i128" => Ty::Fixed(16),
            "u256" | "i256" => Ty::Fixed(32),
            "publicKey" | "pubkey" => Ty::Pubkey,
            "string" => Ty::String,
            "bytes" => Ty::Bytes,
            other => return Err(bad(format!("unsupported IDL type `{other}`"))),
        });
    }
    let obj = v
        .as_object()
        .ok_or_else(|| bad("IDL type is neither a string nor an object"))?;
    if let Some(arr) = obj.get("array").and_then(Value::as_array) {
        let n = arr
            .get(1)
            .and_then(Value::as_u64)
            .ok_or_else(|| bad("IDL array length must be a number"))?;
        return Ok(Ty::Array(Box::new(parse_ty(&arr[0])?), n as usize));
    }
    if let Some(inner) = obj.get("vec") {
        return Ok(Ty::Vec(Box::new(parse_ty(inner)?)));
    }
    if let Some(inner) = obj.get("option").or_else(|| obj.get("coption")) {
        return Ok(Ty::Option(Box::new(parse_ty(inner)?)));
    }
    if let Some(d) = obj.get("defined") {
        let name = d
            .as_str()
            .or_else(|| d.get("name").and_then(Value::as_str))
            .ok_or_else(|| bad("IDL `defined` without a name"))?;
        return Ok(Ty::Defined(name.to_string()));
    }
    Err(bad(format!("unsupported IDL type {v}")))
}

fn parse_fields(fields: &Value) -> Result<Vec<(String, Ty)>, DecodeError> {
    fields
        .as_array()
        .ok_or_else(|| bad("struct fields are not a list"))?
        .iter()
        .map(|f| {
            let name = f["name"]
                .as_str()
                .ok_or_else(|| bad("field without a name"))?;
            Ok((name.to_string(), parse_ty(&f["type"])?))
        })
        .collect()
}

fn parse_typedef(t: &Value) -> Result<TypeDef, DecodeError> {
    match t["kind"].as_str() {
        Some("struct") => Ok(TypeDef::Struct(parse_fields(&t["fields"])?)),
        Some("enum") => {
            let variants = t["variants"]
                .as_array()
                .ok_or_else(|| bad("enum without variants"))?;
            variants
                .iter()
                .map(|v| {
                    let name = v["name"].as_str().unwrap_or_default().to_string();
                    let fields = match v.get("fields").and_then(Value::as_array) {
                        None => Vec::new(),
                        Some(fields) => fields
                            .iter()
                            .map(|f| parse_ty(f.get("type").unwrap_or(f)))
                            .collect::<Result<_, _>>()?,
                    };
                    Ok((name, fields))
                })
                .collect::<Result<_, _>>()
                .map(TypeDef::Enum)
        }
        other => Err(bad(format!("unsupported type kind {other:?}"))),
    }
}

impl Idl {
    pub fn from_json(text: &str) -> Result<Self, DecodeError> {
        let v: Value =
            serde_json::from_str(text).map_err(|e| bad(format!("IDL is not JSON: {e}")))?;
        let mut types = BTreeMap::new();
        for t in v["types"].as_array().into_iter().flatten() {
            if let (Some(name), Ok(def)) = (t["name"].as_str(), parse_typedef(&t["type"])) {
                types.insert(name.to_string(), def);
            }
        }
        let mut accounts = Vec::new();
        for a in v["accounts"].as_array().into_iter().flatten() {
            let name = a["name"]
                .as_str()
                .ok_or_else(|| bad("account without a name"))?;
            // Legacy IDLs define the account layout inline.
            if let Some(t) = a.get("type") {
                types.insert(name.to_string(), parse_typedef(t)?);
            }
            let disc = match a.get("discriminator").and_then(Value::as_array) {
                Some(bytes) => {
                    let b: Vec<u8> = bytes
                        .iter()
                        .filter_map(|x| x.as_u64().map(|n| n as u8))
                        .collect();
                    b.try_into()
                        .map_err(|_| bad("discriminator is not 8 bytes"))?
                }
                None => {
                    let h = Sha256::digest(format!("account:{name}"));
                    h[..8].try_into().expect("8 bytes")
                }
            };
            accounts.push((name.to_string(), disc));
        }
        Ok(Self { accounts, types })
    }

    /// The account type whose discriminator starts `data`.
    pub fn account_type(&self, data: &[u8]) -> Option<&str> {
        let head = data.get(..8)?;
        self.accounts
            .iter()
            .find(|(_, d)| d.as_slice() == head)
            .map(|(n, _)| n.as_str())
    }

    /// The name of variant `index` of enum `name`.
    pub fn enum_variant(&self, name: &str, index: usize) -> Option<&str> {
        match self.types.get(name)? {
            TypeDef::Enum(variants) => variants.get(index).map(|(n, _)| n.as_str()),
            TypeDef::Struct(_) => None,
        }
    }

    /// Offset and type of the field at `path`: dot-separated field names, each
    /// optionally indexed into an array (`price_types[3]`).
    fn locate(&self, data: &[u8], path: &str) -> Result<(usize, Ty), DecodeError> {
        let account = self
            .account_type(data)
            .ok_or_else(|| bad("account discriminator matches no account in the IDL"))?;
        let mut ty = Ty::Defined(account.to_string());
        let mut at = 8usize;
        for part in path.split('.') {
            let (field, index) = match part.split_once('[') {
                Some((f, rest)) => {
                    let i = rest
                        .strip_suffix(']')
                        .and_then(|n| n.parse::<usize>().ok())
                        .ok_or_else(|| bad(format!("bad index in `{part}`")))?;
                    (f, Some(i))
                }
                None => (part, None),
            };
            let Ty::Defined(name) = &ty else {
                return Err(bad(format!(
                    "`{field}` is inside a field that is not a struct"
                )));
            };
            let Some(TypeDef::Struct(fields)) = self.types.get(name) else {
                return Err(bad(format!("type `{name}` is not a struct in the IDL")));
            };
            let mut found = None;
            for (fname, fty) in fields {
                if same_name(fname, field) {
                    found = Some(fty.clone());
                    break;
                }
                at += self.size(fty, data, at)?;
            }
            ty = found.ok_or_else(|| bad(format!("no field `{field}` in `{name}`")))?;
            if let Some(i) = index {
                let Ty::Array(inner, n) = ty else {
                    return Err(bad(format!("`{field}` is not an array")));
                };
                if i >= n {
                    return Err(bad(format!("index {i} is past the end of `{field}` ({n})")));
                }
                for _ in 0..i {
                    at += self.size(&inner, data, at)?;
                }
                ty = *inner;
            }
        }
        Ok((at, ty))
    }

    fn bytes_at(data: &[u8], at: usize, n: usize) -> Result<&[u8], DecodeError> {
        data.get(at..at + n)
            .ok_or_else(|| bad("account data ends before the field"))
    }

    /// Reads the public key at `path` (dot-separated, e.g. `config.admin`)
    /// from an account's data. `Ok(None)` means an empty `Option<Pubkey>`.
    pub fn read_pubkey(&self, data: &[u8], path: &str) -> Result<Option<[u8; 32]>, DecodeError> {
        let (at, ty) = self.locate(data, path)?;
        let take = |at: usize| -> Result<[u8; 32], DecodeError> {
            Ok(Self::bytes_at(data, at, 32)?.try_into().expect("32 bytes"))
        };
        match ty {
            Ty::Pubkey => take(at).map(Some),
            Ty::Option(inner) if *inner == Ty::Pubkey => match data.get(at) {
                Some(0) => Ok(None),
                Some(1) => take(at + 1).map(Some),
                Some(t) => Err(bad(format!("invalid Option tag {t}"))),
                None => Err(bad("account data ends before the field")),
            },
            other => Err(bad(format!(
                "field `{path}` is {other:?}, not a public key"
            ))),
        }
    }

    /// Reads the one-byte field at `path` (`u8`, `i8` or `bool`).
    pub fn read_u8(&self, data: &[u8], path: &str) -> Result<u8, DecodeError> {
        match self.locate(data, path)? {
            (at, Ty::Fixed(1)) => Ok(Self::bytes_at(data, at, 1)?[0]),
            (_, other) => Err(bad(format!("field `{path}` is {other:?}, not one byte"))),
        }
    }

    /// Reads the `[u16; N]` field at `path`.
    pub fn read_u16s(&self, data: &[u8], path: &str) -> Result<Vec<u16>, DecodeError> {
        match self.locate(data, path)? {
            (at, Ty::Array(inner, n)) if *inner == Ty::Fixed(2) => {
                Ok(Self::bytes_at(data, at, 2 * n)?
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect())
            }
            (_, other) => Err(bad(format!("field `{path}` is {other:?}, not a u16 array"))),
        }
    }

    /// Reads the `[u8; N]` field at `path`.
    pub fn read_bytes<'d>(&self, data: &'d [u8], path: &str) -> Result<&'d [u8], DecodeError> {
        match self.locate(data, path)? {
            (at, Ty::Array(inner, n)) if *inner == Ty::Fixed(1) => Self::bytes_at(data, at, n),
            (_, other) => Err(bad(format!(
                "field `{path}` is {other:?}, not a byte array"
            ))),
        }
    }

    /// Bytes `ty` occupies at offset `at` (Borsh).
    fn size(&self, ty: &Ty, data: &[u8], at: usize) -> Result<usize, DecodeError> {
        let u32_at = |at: usize| -> Result<usize, DecodeError> {
            data.get(at..at + 4)
                .map(|b| u32::from_le_bytes(b.try_into().expect("4 bytes")) as usize)
                .ok_or_else(|| bad("account data ends inside a length prefix"))
        };
        Ok(match ty {
            Ty::Fixed(n) => *n,
            Ty::Pubkey => 32,
            Ty::String | Ty::Bytes => 4 + u32_at(at)?,
            Ty::Array(inner, n) => {
                let mut off = 0;
                for _ in 0..*n {
                    off += self.size(inner, data, at + off)?;
                }
                off
            }
            Ty::Vec(inner) => {
                let n = u32_at(at)?;
                let mut off = 4;
                for _ in 0..n {
                    off += self.size(inner, data, at + off)?;
                }
                off
            }
            Ty::Option(inner) => match data.get(at) {
                Some(0) => 1,
                Some(_) => 1 + self.size(inner, data, at + 1)?,
                None => return Err(bad("account data ends inside an Option")),
            },
            Ty::Defined(name) => match self.types.get(name) {
                Some(TypeDef::Struct(fields)) => {
                    let mut off = 0;
                    for (_, f) in fields {
                        off += self.size(f, data, at + off)?;
                    }
                    off
                }
                Some(TypeDef::Enum(variants)) => {
                    let idx = *data
                        .get(at)
                        .ok_or_else(|| bad("data ends inside an enum"))?
                        as usize;
                    let (_, fields) = variants
                        .get(idx)
                        .ok_or_else(|| bad(format!("enum `{name}` has no variant {idx}")))?;
                    let mut off = 1;
                    for f in fields {
                        off += self.size(f, data, at + off)?;
                    }
                    off
                }
                None => return Err(bad(format!("type `{name}` is not in the IDL"))),
            },
        })
    }
}
