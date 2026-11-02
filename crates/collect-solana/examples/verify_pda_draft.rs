//! Read-only offline verification of a review draft against captured authorities.
fn main() {
    let path = std::env::args().nth(1).expect("draft JSON path");
    let rows: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    for row in rows.as_array().unwrap() {
        for value in row["authority_pdas"].as_array().unwrap() {
            let d: celebtir_registry::AuthorityPda = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(
                collect_solana::pda::declared_address(&d).as_ref(),
                Some(&d.key),
                "{}",
                row["symbol"]
            );
            println!("{}: derivation matches {}", row["symbol"], d.key);
        }
    }
}
