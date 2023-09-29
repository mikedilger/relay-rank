
use std::io;
use nostr_types::{PublicKey, RelayInformationDocument, RelayUrl, Unixtime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relay {
    pub url: RelayUrl,
    pub success_count: u64,
    pub failure_count: u64,
    pub last_connected_at: Option<u64>,
    pub last_general_eose_at: Option<u64>,
    pub rank: u64,
    pub hidden: bool,
    pub usage_bits: u64,
    pub nip11: Option<RelayInformationDocument>,
    pub last_attempt_nip11: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Scoring {
    pub score: f32,
    pub ago: i64,
    pub attempts: u64,
    pub success: u64,
    pub rate: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut ranked: Vec<(Relay, Scoring, PublicKey)> = Vec::new();

    let lines = io::stdin().lines();
    for line in lines {
        let line = line?;
        let relay: Relay = serde_json::from_str(&line)?;

        // Skip if we never successfully connected to it
        if relay.success_count==0 {
            continue;
        }

        // Skip if the URL has a path beyond the hostname
        if relay.url.as_url_crate_url().path() != "/" {
            continue;
        }

        // Skip if the relay does not have a NIP-11
        let nip11 = match &relay.nip11 {
            Some(n11) => n11.clone(),
            None => continue,
        };

        // Skip if the pubkey is invalid (or a prefix)
        let pubkey = match nip11.pubkey {
            Some(pkp) => match PublicKey::try_from_hex_string(pkp.as_str(), true) {
                Ok(pk) => pk,
                Err(_) => continue,
            },
            None => continue,
        };

        // Skip if they have a payments url or fees
        if nip11.payments_url.is_some() || nip11.fees.is_some() {
            continue;
        }

        // Skip my perosnal relay (it has high stats because I use it for archival)
        if relay.url.as_str().contains("mikedilger") {
            continue;
        }

        // Score
        let scoring = rank(&relay);

        ranked.push((relay, scoring, pubkey));
    }

    ranked.sort_by(|a,b| b.1.score.partial_cmp(&a.1.score).unwrap());

    for (relay, scoring, _pubkey) in ranked.iter().take(20) {
        println!("{} {:?}", relay.url, scoring);
    }

    Ok(())
}

pub fn rank(relay: &Relay) -> Scoring {
    let last_connected_at = match relay.last_connected_at {
        None => 0,
        Some(time) => time,
    };

    let ago = Unixtime::now().unwrap().0 - last_connected_at as i64;

    let attempts = relay.success_count + relay.failure_count;
    let success = relay.success_count;
    let rate = relay.success_count as f32 / attempts as f32;
    let log_attempts = (attempts as f32).log2();

    let age_penalty_divisor = 1.0 + ago as f32 / 86400.0;

    let score = rate.powf(1.414) * log_attempts * log_attempts / age_penalty_divisor;

    Scoring {
        score,
        ago,
        attempts,
        success,
        rate,
    }
}
