use crypto_lib::{future::FusedFuture, *};
use std::hash::Hash;

use dialoguer::console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;

const OPTIONS: [&str; 5] = [
    "Current block",
    "Balance",
    "Blockchain",
    "Make transaction",
    "Exit",
];

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let blockchain_topic = gossipsub::IdentTopic::new("blockchain");
    let transactions_topic = gossipsub::IdentTopic::new("transactions");

    let mut client = Client::start(transactions_topic.clone()).await?;

    let node = Node::start(blockchain_topic, transactions_topic, client.key_pair.public).await?;

    println!("PUBLIC KEY: {}", hex::encode(client.key_pair.public));

    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select action:")
            .default(0)
            .items(&OPTIONS)
            .interact_on(&Term::stdout())
            .unwrap();
        match selection {
            0 => println!("{:?}", node.active_block.lock().unwrap()),
            1 => println!(
                "{:?}",
                node.active_blockchain
                    .lock()
                    .unwrap()
                    .balances
                    .get(client.key_pair.public.as_bytes())
                    .unwrap_or(&0)
            ),
            2 => println!("{:?}", node.active_blockchain.lock().unwrap()),
            3 => {
                let payee_address =
                    dialoguer::Input::<String>::with_theme(&ColorfulTheme::default())
                        .with_prompt("Payee address")
                        .interact_on(&Term::stdout())
                        .unwrap();

                let amount = dialoguer::Input::<u64>::with_theme(&ColorfulTheme::default())
                    .with_prompt("Amount")
                    .interact_on(&Term::stdout())
                    .unwrap();

                let balance = *node
                    .active_blockchain
                    .lock()
                    .unwrap()
                    .balances
                    .get(client.key_pair.public.as_bytes())
                    .unwrap_or(&0);

                if balance < amount {
                    println!("You don't have that many coins!");
                    continue;
                }

                if let Ok(payee_address) = hex::decode(payee_address) {
                    client.send_transaction(PublicKey::from_bytes(&payee_address).unwrap(), amount);
                } else {
                    println!("Wrong payee address format!");
                }
            }

            4 => break,
            _ => println!("You need to select an action!"),
        }
    }

    Ok(())
}
