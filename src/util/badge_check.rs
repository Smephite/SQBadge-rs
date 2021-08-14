use crate::stellar::*;
use crate::util::error::Error;
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Badge {
    pub token: stellar_data::TOMLCurrency,
    pub tx_hash: String,
    pub owned: bool,
}

type Result<T> = std::result::Result<T, Error>;

pub async fn fetch_badges(
    id: &String,
    available_badges: &Vec<stellar_data::TOMLCurrency>,
) -> Result<Vec<Badge>> {
    let payments = stellar::fetch_account_payments(id).await?;

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
                tx_hash: String::default(),
                owned: false,
            };
            match payment {
                Some(b) => {
                    badge.tx_hash = b.transaction_hash;
                    badge.owned = true;
                }
                _ => {}
            };

            badge
        })
        .collect::<Vec<Badge>>();

    Ok(badges)
}
