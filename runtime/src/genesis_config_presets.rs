use crate::{
    AccountId, BalancesConfig, CollatorSelectionConfig, MembershipConfig, ParachainInfoConfig,
    PolkadotXcmConfig, RuntimeGenesisConfig, SessionConfig, SessionKeys, SudoConfig,
    EXISTENTIAL_DEPOSIT,
};

use alloc::{format, vec, vec::Vec};

use polkadot_sdk::{staging_xcm as xcm, *};

use cumulus_primitives_core::ParaId;
use parachains_common::AuraId;
use polkadot_sdk::pallet_grandpa::AuthorityId as GrandpaId;
use polkadot_sdk::sp_runtime::BoundedVec;
use serde_json::Value;
use sp_genesis_builder::PresetId;
use sp_keyring::Sr25519Keyring;

#[cfg(feature = "runtime-benchmarks")]
use polkadot_sdk::frame_benchmarking::whitelisted_caller;
use polkadot_sdk::sp_application_crypto::Ss58Codec;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;
/// Parachain id used for gensis config presets of parachain template.
const PARACHAIN_ID: u32 = 4724;

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn template_session_keys(grandpa: GrandpaId, aura: AuraId) -> SessionKeys {
    SessionKeys { aura, grandpa }
}

fn testnet_genesis(
    invulnerables: Vec<(AccountId, GrandpaId, AuraId)>,
    endowed_accounts: Vec<AccountId>,
    root: AccountId,
    id: ParaId,
) -> Value {
    let config = RuntimeGenesisConfig {
        balances: BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1u128 << 60))
                .collect::<Vec<_>>(),
        },
        parachain_info: ParachainInfoConfig {
            parachain_id: id,
            ..Default::default()
        },
        collator_selection: CollatorSelectionConfig {
            invulnerables: invulnerables
                .iter()
                .cloned()
                .map(|(acc, _, _)| acc)
                .collect::<Vec<_>>(),
            candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
            ..Default::default()
        },
        session: SessionConfig {
            keys: invulnerables
                .into_iter()
                .map(|(acc, grandpa, aura)| {
                    (
                        acc.clone(),                          // account id
                        acc,                                  // validator id
                        template_session_keys(grandpa, aura), // session keys
                    )
                })
                .collect::<Vec<_>>(),
            ..Default::default()
        },
        polkadot_xcm: PolkadotXcmConfig {
            safe_xcm_version: Some(SAFE_XCM_VERSION),
            ..Default::default()
        },
        sudo: SudoConfig {
            key: Some(root.clone()),
        },
        membership: MembershipConfig {
            members: BoundedVec::try_from(vec![
                Sr25519Keyring::Alice.to_account_id(),
                root.clone(),
                //Sr25519Keyring::Bob.to_account_id(),
                #[cfg(feature = "runtime-benchmarks")]
                whitelisted_caller::<AccountId>().into(),
            ])
            .unwrap(),
            ..Default::default()
        },
        ..Default::default()
    };

    serde_json::to_value(config).expect("Could not build genesis config.")
}
use sp_core::{Pair, Public};
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .unwrap()
        .public()
}

fn development_config_genesis() -> Value {
    let sudo_account =
        AccountId::from_ss58check("5Ev8iRRe9R9Ag4V7hDDnDCwsx9pahjxeR39Bbq7vW69xyKtF").unwrap();
    testnet_genesis(
        // initial collators.
        vec![
            (
                Sr25519Keyring::Alice.to_account_id(),
                get_from_seed::<GrandpaId>("Alice"),
                Sr25519Keyring::Alice.public().into(),
            ),
            (
                Sr25519Keyring::Bob.to_account_id(),
                get_from_seed::<GrandpaId>("Bob"),
                Sr25519Keyring::Bob.public().into(),
            ),
        ],
        vec![Sr25519Keyring::Alice.to_account_id(), sudo_account.clone()],
        sudo_account,
        PARACHAIN_ID.into(),
    )
}

fn local_testnet_genesis() -> Value {
    testnet_genesis(
        // initial collators.
        vec![
            (
                Sr25519Keyring::Alice.to_account_id(),
                get_from_seed::<GrandpaId>("Alice"),
                Sr25519Keyring::Alice.public().into(),
            ),
            (
                Sr25519Keyring::Bob.to_account_id(),
                get_from_seed::<GrandpaId>("Bob"),
                Sr25519Keyring::Bob.public().into(),
            ),
        ],
        Sr25519Keyring::well_known()
            .map(|k| k.to_account_id())
            .collect(),
        Sr25519Keyring::Alice.to_account_id(),
        PARACHAIN_ID.into(),
    )
}

/// Provides the JSON representation of predefined genesis config for given `id`.
pub fn get_preset(id: &PresetId) -> Option<vec::Vec<u8>> {
    let patch = match id.as_ref() {
        sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => local_testnet_genesis(),
        sp_genesis_builder::DEV_RUNTIME_PRESET => development_config_genesis(),
        _ => return None,
    };
    Some(
        serde_json::to_string(&patch)
            .expect("serialization to json is expected to work. qed.")
            .into_bytes(),
    )
}

/// List of supported presets.
pub fn preset_names() -> Vec<PresetId> {
    vec![
        PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET),
        PresetId::from(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET),
    ]
}
