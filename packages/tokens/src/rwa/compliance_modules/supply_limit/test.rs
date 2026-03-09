extern crate std;

use soroban_sdk::{
    contract, contractimpl, contracttype, testutils::Address as _, Address, Env, Symbol,
};

use super::*;
use crate::rwa::{
    compliance::ComplianceHook,
    compliance_modules::common::{hooks_verified, set_compliance_address, ComplianceHookCheck},
};

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
struct TestSupplyLimitContract;

#[contractimpl(contracttrait)]
impl SupplyLimit for TestSupplyLimitContract {
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
    let module_id = e.register(TestSupplyLimitContract, ());
    let compliance_id = e.register(MockComplianceContract, ());
    let compliance = MockComplianceContractClient::new(&e, &compliance_id);

    for hook in [ComplianceHook::CanCreate, ComplianceHook::Created, ComplianceHook::Destroyed] {
        compliance.register_hook(&hook, &module_id);
    }

    e.as_contract(&module_id, || {
        set_compliance_address(&e, &compliance_id);

        <TestSupplyLimitContract as SupplyLimit>::verify_hook_wiring(&e);

        assert!(hooks_verified(&e));
    });
}

#[test]
fn get_supply_limit_returns_zero_when_unconfigured() {
    let e = Env::default();
    let module_id = e.register(TestSupplyLimitContract, ());
    let token = Address::generate(&e);

    e.as_contract(&module_id, || {
        assert_eq!(<TestSupplyLimitContract as SupplyLimit>::get_supply_limit(&e, token), 0);
    });
}

#[test]
fn can_create_allows_when_limit_is_unset_and_rejects_negative_amount() {
    let e = Env::default();
    let module_id = e.register(TestSupplyLimitContract, ());
    let token = Address::generate(&e);
    let recipient = Address::generate(&e);

    e.as_contract(&module_id, || {
        arm_hooks(&e);

        assert!(<TestSupplyLimitContract as SupplyLimit>::can_create(
            &e,
            recipient.clone(),
            100,
            token.clone(),
        ));
        assert!(!<TestSupplyLimitContract as SupplyLimit>::can_create(
            &e,
            recipient,
            -1,
            token.clone(),
        ));
    });
}

#[test]
fn hooks_update_internal_supply_and_cap_future_mints() {
    let e = Env::default();
    e.mock_all_auths();

    let module_id = e.register(TestSupplyLimitContract, ());
    let compliance_id = e.register(MockComplianceContract, ());
    let token = Address::generate(&e);
    let recipient = Address::generate(&e);

    e.as_contract(&module_id, || {
        set_compliance_address(&e, &compliance_id);
        arm_hooks(&e);

        <TestSupplyLimitContract as SupplyLimit>::set_supply_limit(&e, token.clone(), 100);

        assert!(<TestSupplyLimitContract as SupplyLimit>::can_create(
            &e,
            recipient.clone(),
            80,
            token.clone(),
        ));
        <TestSupplyLimitContract as SupplyLimit>::on_created(
            &e,
            recipient.clone(),
            80,
            token.clone(),
        );
        assert_eq!(
            <TestSupplyLimitContract as SupplyLimit>::get_internal_supply(&e, token.clone()),
            80
        );

        assert!(!<TestSupplyLimitContract as SupplyLimit>::can_create(
            &e,
            recipient.clone(),
            30,
            token.clone(),
        ));

        <TestSupplyLimitContract as SupplyLimit>::on_destroyed(
            &e,
            recipient.clone(),
            20,
            token.clone(),
        );
        assert_eq!(
            <TestSupplyLimitContract as SupplyLimit>::get_internal_supply(&e, token.clone()),
            60
        );
        assert!(<TestSupplyLimitContract as SupplyLimit>::can_create(
            &e,
            recipient,
            40,
            token.clone(),
        ));
    });
}
