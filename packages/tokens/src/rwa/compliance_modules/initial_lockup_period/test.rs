extern crate std;

use soroban_sdk::{contract, contractimpl, testutils::Address as _, vec, Address, Env, Symbol};

use super::*;
use crate::rwa::compliance_modules::common::set_compliance_address;

#[contract]
struct TestInitialLockupPeriodContract;

#[contractimpl(contracttrait)]
impl InitialLockupPeriod for TestInitialLockupPeriodContract {
    fn set_compliance_address(_e: &Env, _compliance: Address) {
        unreachable!("set_compliance_address is not used in these tests");
    }
}

fn arm_hooks(e: &Env) {
    e.storage().persistent().set(&Symbol::new(e, "hooks_verified"), &true);
}

#[test]
fn pre_set_lockup_state_seeds_existing_locked_balance() {
    let e = Env::default();
    e.mock_all_auths();

    let module_id = e.register(TestInitialLockupPeriodContract, ());
    let compliance = Address::generate(&e);
    let token = Address::generate(&e);
    let wallet = Address::generate(&e);

    e.as_contract(&module_id, || {
        set_compliance_address(&e, &compliance);
        arm_hooks(&e);

        <TestInitialLockupPeriodContract as InitialLockupPeriod>::pre_set_lockup_state(
            &e,
            token.clone(),
            wallet.clone(),
            100,
            vec![
                &e,
                LockedTokens {
                    amount: 80,
                    release_timestamp: e.ledger().timestamp().saturating_add(60),
                },
            ],
        );

        assert_eq!(
            <TestInitialLockupPeriodContract as InitialLockupPeriod>::get_internal_balance(
                &e,
                token.clone(),
                wallet.clone(),
            ),
            100
        );
        assert_eq!(
            <TestInitialLockupPeriodContract as InitialLockupPeriod>::get_total_locked(
                &e,
                token.clone(),
                wallet.clone(),
            ),
            80
        );
        assert!(!<TestInitialLockupPeriodContract as InitialLockupPeriod>::can_transfer(
            &e,
            wallet.clone(),
            Address::generate(&e),
            21,
            token.clone(),
        ));
        assert!(<TestInitialLockupPeriodContract as InitialLockupPeriod>::can_transfer(
            &e,
            wallet,
            Address::generate(&e),
            20,
            token,
        ));
    });
}
