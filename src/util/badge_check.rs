use crate::stellar::*;

pub struct Badge {
    code: String,
    issuer: String,
}

pub async fn fetch_badges(id: String, available_badges: Vec<Badge>) -> Vec<Badge> {
    let account = match stellar::fetch_account(id).await {
        Ok(data) => data,
        Err(_) => {
            return vec![];
        }
    };

    let searched_for: Vec<String> = available_badges
        .into_iter()
        .map(|b| b.code + ":" + &b.issuer)
        .collect();

    let matching_badges = account
        .balances
        .into_iter()
        .filter(|balance| {
            balance.asset_type == "credit_alphanum12"
                && searched_for
                    .contains(&(balance.asset_code.clone() + ":" + &balance.asset_issuer))
        })
        .map(|bal| Badge {
            code: bal.asset_code,
            issuer: bal.asset_issuer,
        })
        .collect::<Vec<Badge>>();

    matching_badges
}
