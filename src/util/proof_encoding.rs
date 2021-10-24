use crate::{js::albedo, util::error::Error};
use itertools::Itertools;
use js_sys::JsString;
use log::{debug, info, warn};

use crate::stellar::stellar_data::TOMLCurrency;

use super::error::ProofErr;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Proof {
    pub owned_badges: Vec<TOMLCurrency>,
    pub timestamp: Option<u64>,
    pub unique_id: Option<String>,
}

impl Proof {
    pub fn encode_v1(&self) -> Result<String, Error> {
        let owned_badges = self
            .owned_badges
            .clone()
            .into_iter()
            .unique_by(|b| b.code.clone())
            .map(|b| {
                let name = b.code;
                let mut iter = name.chars().fuse();
                let mut series: u32;
                let mut task: u32;
                if name.starts_with("SSQ") {
                    iter.nth(2); // skip 3
                    task = 0; // special badges are 1 per series
                    series = iter.next().unwrap().to_digit(10).unwrap() * 10;
                    series += iter.next().unwrap().to_digit(10).unwrap();
                    //   SSQ        xx
                    //    v         v
                    // \ void /  $series
                } else {
                    iter.nth(1); // skip 2

                    series = iter.next().unwrap().to_digit(10).unwrap() * 10;
                    series += iter.next().unwrap().to_digit(10).unwrap();
                    task = iter.next().unwrap().to_digit(10).unwrap() * 10;
                    task += iter.next().unwrap().to_digit(10).unwrap();
                    //    SQ      xx     xx
                    //    v        v      v
                    // \ void / $series $task
                }

                (series as u8, task as u8)
            });
        let mut translated =
            vec![0; owned_badges.clone().into_iter().map(|b| b.0).max().unwrap() as usize]; // initialize with amout of series entries

        for b in owned_badges {
            if b.1 > 8 {
                continue; // only allow 8+1 badges per series
            }
            translated[b.0 as usize - 1] = translated[b.0 as usize - 1] + (1 << b.1);
            debug!(
                "S{}Q{} -> {}:{}",
                b.0,
                b.1,
                b.0,
                translated[b.0 as usize - 1]
            );
            // each series (index from 0) has one byte of storage where the LSB is Q00 (SSQ) and MSB is Q07
            // bit == 1 <=> user owns badge
            // encoding may be prefixed with zeros (-> any new quests are automatically set to not be owned)
        }

        debug!("translated values per quest: {:?}", translated);

        let value = translated
            .into_iter()
            .enumerate()
            .fold(0usize, |old, (index, val)| old + (val << 9 * index)); // 9 badges per series

        Ok(format!(
            "v1.{:x}.{}.{}",
            value,
            self.timestamp.unwrap_or(0),
            self.unique_id.clone().unwrap_or(String::default())
        ))
    }

    pub fn decode_v1(
        encoded: &String,
        available_badges: &Vec<TOMLCurrency>,
    ) -> Result<Proof, Error> {
        let mut parts = encoded.split(".");
        let parts_arr = parts.clone().collect::<Vec<&str>>();
        debug!("decoding proof({}): {:?}", encoded, parts_arr);
        if parts_arr.len() < 4 {
            return Err(Error::ProofErr(ProofErr::ProofInvalidEncoding));
        }
        if parts.next().unwrap() != "v1" {
            return Err(Error::ProofErr(ProofErr::ProofWrongVersion));
        }
        let badges = parts.next().unwrap();
        let datetime = parts.next().unwrap();

        let mut unique_id = String::new();
        parts.for_each(|p| unique_id.push_str(p));

        let mut final_proof = Proof::default();

        final_proof.unique_id = match unique_id == "" {
            true => None,
            false => Some(String::from(unique_id)),
        };
        final_proof.timestamp = match datetime == "" {
            true => None,
            false => datetime.parse().ok(),
        };

        let mut badges = u64::from_str_radix(badges, 16).unwrap_or(0);

        let mut q_series = 1;
        let mut q_challenge = 0;
        let mut owned_badges_name: Vec<String> = vec![];
        while badges != 0 {
            let badge_completed = badges % 2 == 1;
            let quest_name = match q_challenge {
                0 => format!("SSQ{:02}", q_series),
                _ => format!("SQ{:02}{:02}", q_series, q_challenge),
            };
            debug!("{} => {}", quest_name, badge_completed);

            if badge_completed {
                owned_badges_name.push(quest_name);
            }

            badges = badges >> 1;
            q_challenge += 1;
            if q_challenge > 8 {
                q_challenge = 0;
                q_series += 1;
            }
        }

        //TODO: This will add mono as well as normal badges even if account only ownes one of the two...
        final_proof.owned_badges = available_badges
            .clone()
            .into_iter()
            .filter(|b| owned_badges_name.contains(&b.code))
            .collect();

        Ok(final_proof)
    }
}

pub fn verify_albedo_signed_message(base64_proof: &String) -> Option<(bool, String, String)> {
    debug!("Trying to decrypt albedo signed message {}", base64_proof);

    let bytes = base64::decode(base64_proof);

    if bytes.is_err() {
        return None;
    }

    let proof = String::from_utf8(bytes.unwrap());

    debug!("Decoded {:?}", proof);

    if proof.is_err() {
        return None;
    }

    let split = proof
        .unwrap()
        .split(":")
        .map(|s| s.to_owned())
        .collect::<Vec<String>>();

    if split.len() < 3 {
        return None;
    }
    let message_sig = split[0].clone();
    let pub_key = split[1].clone();
    let mut plain_message = String::new();

    split.iter().skip(2).for_each(|s| plain_message.push_str(s));

    let valid = albedo::albedo_verify_message_signature(
        JsString::from(pub_key.clone()),
        JsString::from(plain_message.clone()),
        JsString::from(message_sig.clone()),
    )
    .value_of();

    debug!(
        "Message `{}` signed by `{}`: `{}` is {}!",
        plain_message,
        pub_key,
        message_sig,
        match valid {
            true => "valid",
            _ => "invalid",
        }
    );
    Some((valid, plain_message, pub_key))
}
