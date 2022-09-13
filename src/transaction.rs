use super::*;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Transaction {
    pub from: Hash,
    pub signature: Hash,
    pub to: Hash,
    pub amount: u64,
}
