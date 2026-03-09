extern crate std;

use soroban_sdk::{
    contract, contractimpl, contracttype, testutils::Address as _, Address, Env, Symbol, Vec,
};

use super::*;
use crate::rwa::{
    compliance::ComplianceHook,
    compliance_modules::common::{
        hooks_verified, set_compliance_address, set_irs_address, ComplianceHookCheck, IRSRead,
    },
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
struct MockComplianceContract;

#[contracttype]
#[derive(Clone)]
enum MockComplianceStorageKey {
    Registered(ComplianceHook, Address),
}

#[contractimpl]
impl ComplianceHookCheck for MockComplianceContract {
    fn is_module_registered(e: &Env, hook: ComplianceHook, module: Address) -> bool {
        e.storage().persistent().has(&MockComplianceStorageKey::Registered(hook, module))
    }
}

#[contractimpl]
impl MockComplianceContract {
    pub fn register_hook(e: &Env, hook: ComplianceHook, module: Address) {
        e.storage().persistent().set(&MockComplianceStorageKey::Registered(hook, module), &true);
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
fn verify_hook_wiring_sets_cache_when_registered() {
    let e = Env::default();
    let module_id = e.register(TestTimeTransfersLimitsContract, ());
    let compliance_id = e.register(MockComplianceContract, ());
    let compliance = MockComplianceContractClient::new(&e, &compliance_id);

    for hook in [ComplianceHook::CanTransfer, ComplianceHook::Transferred] {
        compliance.register_hook(&hook, &module_id);
    }

    e.as_contract(&module_id, || {
        set_compliance_address(&e, &compliance_id);

        <TestTimeTransfersLimitsContract as TimeTransfersLimits>::verify_hook_wiring(&e);

        assert!(hooks_verified(&e));
    });
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

#[test]
#[should_panic(expected = "Error(Contract, #400)")]
fn set_time_transfer_limit_rejects_more_than_four_limits() {
    let e = Env::default();
    e.mock_all_auths();

    let module_id = e.register(TestTimeTransfersLimitsContract, ());
    let compliance = Address::generate(&e);
    let token = Address::generate(&e);

    e.as_contract(&module_id, || {
        set_compliance_address(&e, &compliance);

        for limit_time in [60_u64, 120, 180, 240] {
            <TestTimeTransfersLimitsContract as TimeTransfersLimits>::set_time_transfer_limit(
                &e,
                token.clone(),
                Limit { limit_time, limit_value: 100 },
            );
        }

        <TestTimeTransfersLimitsContract as TimeTransfersLimits>::set_time_transfer_limit(
            &e,
            token,
            Limit { limit_time: 300, limit_value: 100 },
        );
    });
}
