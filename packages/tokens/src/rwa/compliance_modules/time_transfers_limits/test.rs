extern crate std;

use soroban_sdk::{
    contract, contractimpl, contracttype, testutils::Address as _, Address, Env, Symbol, Vec,
};

use super::*;
use crate::rwa::{
    compliance_modules::common::{set_compliance_address, set_irs_address, IRSRead},
    identity_registry_storage::CountryData,
};

#[contract]
struct MockIRSContract;

#[contracttype]
#[derive(Clone)]
enum MockIRSStorageKey {
    Identity(Address),
}

#[contractimpl]
impl IRSRead for MockIRSContract {
    fn stored_identity(e: &Env, account: Address) -> Address {
        e.storage()
            .persistent()
            .get(&MockIRSStorageKey::Identity(account.clone()))
            .unwrap_or(account)
    }

    fn get_country_data_entries(e: &Env, _account: Address) -> Vec<CountryData> {
        Vec::new(e)
    }
}

#[contractimpl]
impl MockIRSContract {
    pub fn set_identity(e: &Env, account: Address, identity: Address) {
        e.storage().persistent().set(&MockIRSStorageKey::Identity(account), &identity);
    }
}

#[contract]
struct TestTimeTransfersLimitsContract;

#[contractimpl(contracttrait)]
impl TimeTransfersLimits for TestTimeTransfersLimitsContract {
    fn set_compliance_address(_e: &Env, _compliance: Address) {
        unreachable!("set_compliance_address is not used in these tests");
    }
}

fn arm_hooks(e: &Env) {
    e.storage().persistent().set(&Symbol::new(e, "hooks_verified"), &true);
}

#[test]
fn pre_set_transfer_counter_blocks_transfers_within_active_window() {
    let e = Env::default();
    e.mock_all_auths();

    let module_id = e.register(TestTimeTransfersLimitsContract, ());
    let irs_id = e.register(MockIRSContract, ());
    let irs = MockIRSContractClient::new(&e, &irs_id);
    let compliance = Address::generate(&e);
    let token = Address::generate(&e);
    let sender = Address::generate(&e);
    let sender_identity = Address::generate(&e);
    let recipient = Address::generate(&e);

    irs.set_identity(&sender, &sender_identity);

    e.as_contract(&module_id, || {
        set_compliance_address(&e, &compliance);
        set_irs_address(&e, &token, &irs_id);
        arm_hooks(&e);

        <TestTimeTransfersLimitsContract as TimeTransfersLimits>::set_time_transfer_limit(
            &e,
            token.clone(),
            Limit { limit_time: 60, limit_value: 100 },
        );
        <TestTimeTransfersLimitsContract as TimeTransfersLimits>::pre_set_transfer_counter(
            &e,
            token.clone(),
            sender_identity.clone(),
            60,
            TransferCounter { value: 90, timer: e.ledger().timestamp().saturating_add(60) },
        );

        assert!(!<TestTimeTransfersLimitsContract as TimeTransfersLimits>::can_transfer(
            &e,
            sender.clone(),
            recipient.clone(),
            11,
            token.clone(),
        ));
        assert!(<TestTimeTransfersLimitsContract as TimeTransfersLimits>::can_transfer(
            &e, sender, recipient, 10, token,
        ));
    });
}
