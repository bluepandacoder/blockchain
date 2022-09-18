
use super::*;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TransactionData {
    pub from: PublicKey,
    pub to: PublicKey,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub data: TransactionData,
    pub signature: Signature
}

impl core::fmt::Debug for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Transaction with {} coins", self.data.amount)
    }
}

impl Transaction {
    pub fn new(to: PublicKey, amount: u64, key_pair: &Keypair) -> Self {
        let data = TransactionData { from: key_pair.public, to, amount };
        let signature = key_pair.sign(&bincode::serialize(&data).unwrap());

        Self {
            data,
            signature
        }
    }

    pub fn valid(&self) -> bool {
        self.data.from.verify(
            &bincode::serialize(&self.data).unwrap(),
            &self.signature
        ).is_ok()
    }
}


