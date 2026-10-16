#![cfg(test)]

use crate::{Error, ProposalKind, SubscriptionVault, SubscriptionVaultClient};
use soroban_sdk::{
	testutils::{Address as _, Ledger as _},
	Address, Env,
};

#[test]
fn admin_rotation_invalidates_old_governance_proposals() {
	let env = Env::default();
	env.mock_all_auths();
	let contract_id = env.register(SubscriptionVault, ());
	let client = SubscriptionVaultClient::new(&env, &contract_id);
	let admin = Address::generate(&env);
	let token = env
		.register_stellar_asset_contract_v2(admin.clone())
		.address();
	client.init(&token, &6, &admin, &1_000_000i128, &(7 * 24 * 60 * 60));

	env.ledger().set_timestamp(1_000_000);
	client.add_guardian(&admin, &admin, &1u32);

	let old_eta = 1_000_000 + 3 * 24 * 60 * 60;
	let stale_id = client
		.submit_proposal(
			&ProposalKind::EmergencyStop,
			&admin,
			&None::<Address>,
			&1u32,
			&5_000u32,
			&old_eta,
		)
		.unwrap();
	client.vote_proposal(&stale_id, &true, &0);

	let new_admin = Address::generate(&env);
	client.propose_admin(&admin, &new_admin);
	env.ledger().set_timestamp(1_000_000 + 24 * 60 * 60);
	client.claim_admin_role(&new_admin);
	client.add_guardian(&new_admin, &new_admin, &1u32);

	assert_eq!(
		client.try_vote_proposal(&stale_id, &true, &0),
		Ok(Err(Error::ProposalInvalidated))
	);

	let fresh_eta = 1_000_000 + 3 * 24 * 60 * 60;
	let fresh_id = client
		.submit_proposal(
			&ProposalKind::EmergencyStop,
			&new_admin,
			&None::<Address>,
			&1u32,
			&5_000u32,
			&fresh_eta,
		)
		.unwrap();
	client.vote_proposal(&fresh_id, &true, &0);

	env.ledger().set_timestamp(old_eta);
	assert_eq!(
		client.try_execute_proposal(&stale_id),
		Ok(Err(Error::ProposalInvalidated))
	);
	assert_eq!(client.try_execute_proposal(&fresh_id), Ok(Ok(())));
}

#[test]
fn direct_admin_rotation_invalidates_outstanding_proposals() {
	let env = Env::default();
	env.mock_all_auths();
	let contract_id = env.register(SubscriptionVault, ());
	let client = SubscriptionVaultClient::new(&env, &contract_id);
	let admin = Address::generate(&env);
	let token = env
		.register_stellar_asset_contract_v2(admin.clone())
		.address();
	client.init(&token, &6, &admin, &1_000_000i128, &(7 * 24 * 60 * 60));
	env.ledger().set_timestamp(1_000_000);
	client.add_guardian(&admin, &admin, &1u32);

	let eta = 1_000_000 + 3 * 24 * 60 * 60;
	let proposal_id = client
		.submit_proposal(
			&ProposalKind::EmergencyStop,
			&admin,
			&None::<Address>,
			&1u32,
			&5_000u32,
			&eta,
		)
		.unwrap();

	let new_admin = Address::generate(&env);
	let nonce = client.get_admin_nonce(&admin, 1);
	client.rotate_admin(&admin, &new_admin, &nonce);
	client.add_guardian(&new_admin, &new_admin, &1u32);

	assert_eq!(
		client.try_vote_proposal(&proposal_id, &true, &0),
		Ok(Err(Error::ProposalInvalidated))
	);
}
