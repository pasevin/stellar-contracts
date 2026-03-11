extern crate std;

use soroban_sdk::{
    contract, contractimpl, contracttype, testutils::Address as _, Address, Env, Symbol,
};

use super::*;
use crate::rwa::{
    compliance::{Compliance, ComplianceHook},
    compliance_modules::common::{hooks_verified, set_compliance_address},
    utils::token_binder::TokenBinder,
};

#[contract]
struct MockComplianceContract;

#[contracttype]
#[derive(Clone)]
enum MockComplianceStorageKey {
    Registered(ComplianceHook, Address),
}

#[contractimpl]
impl Compliance for MockComplianceContract {
    fn add_module_to(_e: &Env, _hook: ComplianceHook, _module: Address, _operator: Address) {
        unreachable!("add_module_to is not used in these tests");
    }

    fn remove_module_from(_e: &Env, _hook: ComplianceHook, _module: Address, _operator: Address) {
        unreachable!("remove_module_from is not used in these tests");
    }

    fn get_modules_for_hook(_e: &Env, _hook: ComplianceHook) -> soroban_sdk::Vec<Address> {
        unreachable!("get_modules_for_hook is not used in these tests");
    }

    fn is_module_registered(e: &Env, hook: ComplianceHook, module: Address) -> bool {
        e.storage().persistent().has(&MockComplianceStorageKey::Registered(hook, module))
    }

    fn transferred(_e: &Env, _from: Address, _to: Address, _amount: i128, _token: Address) {
        unreachable!("transferred is not used in these tests");
    }

    fn created(_e: &Env, _to: Address, _amount: i128, _token: Address) {
        unreachable!("created is not used in these tests");
    }

    fn destroyed(_e: &Env, _from: Address, _amount: i128, _token: Address) {
        unreachable!("destroyed is not used in these tests");
    }

    fn can_transfer(
        _e: &Env,
        _from: Address,
        _to: Address,
        _amount: i128,
        _token: Address,
    ) -> bool {
        unreachable!("can_transfer is not used in these tests");
    }

    fn can_create(_e: &Env, _to: Address, _amount: i128, _token: Address) -> bool {
        unreachable!("can_create is not used in these tests");
    }
}

#[contractimpl]
impl TokenBinder for MockComplianceContract {
    fn linked_tokens(e: &Env) -> soroban_sdk::Vec<Address> {
        soroban_sdk::Vec::new(e)
    }

    fn bind_token(_e: &Env, _token: Address, _operator: Address) {
        unreachable!("bind_token is not used in these tests");
    }

    fn unbind_token(_e: &Env, _token: Address, _operator: Address) {
        unreachable!("unbind_token is not used in these tests");
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

#[test]
fn pre_set_internal_supply_seeds_existing_supply_for_cap_checks() {
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
        <TestSupplyLimitContract as SupplyLimit>::pre_set_internal_supply(&e, token.clone(), 90);

        assert_eq!(
            <TestSupplyLimitContract as SupplyLimit>::get_internal_supply(&e, token.clone()),
            90
        );
        assert!(!<TestSupplyLimitContract as SupplyLimit>::can_create(
            &e,
            recipient.clone(),
            11,
            token.clone(),
        ));
        assert!(<TestSupplyLimitContract as SupplyLimit>::can_create(
            &e,
            recipient,
            10,
            token.clone(),
        ));
    });
}
