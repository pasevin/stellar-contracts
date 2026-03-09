extern crate std;

use soroban_sdk::{contract, contractimpl, testutils::Address as _, vec, Address, Env};

use super::*;
use crate::rwa::compliance_modules::common::set_compliance_address;

#[contract]
struct TestTransferRestrictContract;

#[contractimpl(contracttrait)]
impl TransferRestrict for TestTransferRestrictContract {
    fn set_compliance_address(_e: &Env, _compliance: Address) {
        unreachable!("set_compliance_address is not used in these tests");
    }
}

#[test]
fn can_transfer_allows_sender_or_recipient_when_allowlisted() {
    let e = Env::default();
    e.mock_all_auths();

    let module_id = e.register(TestTransferRestrictContract, ());
    let compliance = Address::generate(&e);
    let token = Address::generate(&e);
    let sender = Address::generate(&e);
    let recipient = Address::generate(&e);
    let outsider = Address::generate(&e);

    e.as_contract(&module_id, || {
        set_compliance_address(&e, &compliance);

        assert!(!<TestTransferRestrictContract as TransferRestrict>::can_transfer(
            &e,
            sender.clone(),
            recipient.clone(),
            100,
            token.clone(),
        ));

        <TestTransferRestrictContract as TransferRestrict>::allow_user(
            &e,
            token.clone(),
            sender.clone(),
        );
        assert!(<TestTransferRestrictContract as TransferRestrict>::can_transfer(
            &e,
            sender.clone(),
            outsider.clone(),
            100,
            token.clone(),
        ));

        <TestTransferRestrictContract as TransferRestrict>::disallow_user(
            &e,
            token.clone(),
            sender.clone(),
        );
        <TestTransferRestrictContract as TransferRestrict>::allow_user(
            &e,
            token.clone(),
            recipient.clone(),
        );
        assert!(<TestTransferRestrictContract as TransferRestrict>::can_transfer(
            &e, outsider, recipient, 100, token,
        ));
    });
}

#[test]
fn batch_allow_and_disallow_update_allowlist_entries() {
    let e = Env::default();
    e.mock_all_auths();

    let module_id = e.register(TestTransferRestrictContract, ());
    let compliance = Address::generate(&e);
    let token = Address::generate(&e);
    let user_a = Address::generate(&e);
    let user_b = Address::generate(&e);

    e.as_contract(&module_id, || {
        set_compliance_address(&e, &compliance);

        <TestTransferRestrictContract as TransferRestrict>::batch_allow_users(
            &e,
            token.clone(),
            vec![&e, user_a.clone(), user_b.clone()],
        );

        assert!(<TestTransferRestrictContract as TransferRestrict>::is_user_allowed(
            &e,
            token.clone(),
            user_a.clone(),
        ));
        assert!(<TestTransferRestrictContract as TransferRestrict>::is_user_allowed(
            &e,
            token.clone(),
            user_b.clone(),
        ));

        <TestTransferRestrictContract as TransferRestrict>::batch_disallow_users(
            &e,
            token.clone(),
            vec![&e, user_a.clone(), user_b.clone()],
        );

        assert!(!<TestTransferRestrictContract as TransferRestrict>::is_user_allowed(
            &e,
            token.clone(),
            user_a,
        ));
        assert!(!<TestTransferRestrictContract as TransferRestrict>::is_user_allowed(
            &e, token, user_b,
        ));
    });
}
