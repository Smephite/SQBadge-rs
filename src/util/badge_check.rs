use crate::stellar::*;
use crate::util::error::Error;
use futures::stream::StreamExt;
use log::debug;
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Badge {
    pub token: stellar_data::TOMLCurrency,
    pub tx_hash: Option<String>,
    pub owned: bool,
    pub date_accuired: Option<String>,
}

impl Badge {
    pub fn is_mono(&self) -> bool {
        self.token.tag == String::from("mono")
    }
}

type Result<T> = std::result::Result<T, Error>;

pub async fn fetch_badges(
    id: &String,
    available_badges: &Vec<stellar_data::TOMLCurrency>,
) -> Result<Vec<Badge>> {
    let payments = stellar::fetch_account_payments(id).await?;
    let balances = stellar::fetch_account(id).await?.balances;

    let badges = available_badges
        .into_iter()
        .map(|badge| {
            let payment = payments
                .clone()
                .into_iter()
                .filter(|p| {
                    p.asset_type == "credit_alphanum12"
                        && p.asset_issuer == p.from
                        && p.asset_issuer == badge.issuer
                        && p.asset_code == badge.code
                })
                .next();
            let mut badge = Badge {
                token: badge.clone(),
                tx_hash: None,
                owned: false,
                date_accuired: None,
            };
            match payment {
                Some(b) => {
                    badge.tx_hash = Some(b.transaction_hash);
                    badge.owned = true;
                    badge.date_accuired = Some(b.created_at);
                }
                _ => {}
            };

            badge
        })
        .collect::<Vec<Badge>>();

    let badges = futures::stream::iter(badges)
        .then(|mut b| async {
            if !b.owned {
                let bal = balances
                    .iter()
                    .filter(|bal| {
                        &bal.asset_type == "credit_alphanum12"
                            && bal.asset_code == b.token.code
                            && bal.asset_issuer == b.token.issuer
                    })
                    .next();
                if let Some(bal) = bal {
                    debug!("{}: not owned but in balance!", b.token.code);
                    let asset = format!("{}:{}", bal.asset_code, bal.asset_issuer);
                    let claimable_balance =
                        stellar::search_created_claimed_balances(&bal.asset_issuer, &asset, id)
                            .await;
                    debug!(
                        "{}: found matching claimable balance? {}",
                        b.token.code,
                        claimable_balance.is_some()
                    );
                    if claimable_balance.is_some() {
                        let claimable_balance = claimable_balance.unwrap();
                        b.owned = true;
                        b.tx_hash = Some(claimable_balance.transaction_hash);
                        b.date_accuired = Some(claimable_balance.created_at)
                    }
                }
            }

            b
        })
        .collect::<Vec<_>>()
        .await;

    Ok(badges)
}
