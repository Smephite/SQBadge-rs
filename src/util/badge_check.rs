use crate::stellar::*;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Badge {
    code: String,
    issuer: String,
    ledger_mod: u64,
}

pub async fn fetch_badges(id: &String, available_badges: &Vec<(String, String)>) -> Vec<Badge> {
    let account = match stellar::fetch_account(id).await {
        Ok(data) => data,
        Err(_) => {
            return vec![];
        }
    };

    log::info!("{:?}\n{:?}", available_badges, account.balances);

    let matching_badges = account
        .balances
        .into_iter()
        .filter(|balance| {
            balance.asset_type == "credit_alphanum12"
                && available_badges
                    .contains(&(balance.asset_issuer.clone(), balance.asset_code.clone()))
        })
        .map(|bal| Badge {
            code: bal.asset_code,
            issuer: bal.asset_issuer,
            ledger_mod: bal.last_modified_ledger,
        })
        .collect::<Vec<Badge>>();

    matching_badges
}
