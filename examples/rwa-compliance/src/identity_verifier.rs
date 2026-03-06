//! Simplified Identity Verifier contract.
//!
//! Checks that an account has an identity registered in the Identity Registry
//! Storage (IRS). Does **not** perform claim-based verification — suitable for
//! demonstrating the compliance module stack without the full claims pipeline.

<<<<<<< HEAD
use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, Env, Symbol, Vec,
};
use stellar_access::access_control::{self as access_control, AccessControl};
use stellar_macros::only_role;
use stellar_tokens::rwa::{
    emit_claim_topics_and_issuers_set, identity_verifier::IdentityVerifier, RWAError,
};
=======
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};
use stellar_tokens::rwa::identity_verifier::IdentityVerifier;
>>>>>>> 3cfdd149 (test(rwa): add compliance module integration test suite)

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Irs,
    ClaimTopicsAndIssuers,
}

#[soroban_sdk::contractclient(name = "IRSClient")]
#[allow(dead_code)]
trait IRSView {
    fn stored_identity(e: &Env, account: Address) -> Address;
    fn get_recovered_to(e: &Env, old: Address) -> Option<Address>;
}

#[contract]
pub struct SimpleIdentityVerifier;

<<<<<<< HEAD
fn identity_registry_storage(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&DataKey::Irs)
        .unwrap_or_else(|| panic_with_error!(e, RWAError::IdentityRegistryStorageNotSet))
}

#[contractimpl]
impl SimpleIdentityVerifier {
    pub fn __constructor(e: &Env, admin: Address, irs: Address) {
        access_control::set_admin(e, &admin);
        access_control::grant_role_no_auth(e, &admin, &symbol_short!("admin"), &admin);
=======
#[contractimpl]
impl SimpleIdentityVerifier {
    pub fn __constructor(e: &Env, irs: Address) {
>>>>>>> 3cfdd149 (test(rwa): add compliance module integration test suite)
        e.storage().instance().set(&DataKey::Irs, &irs);
    }
}

#[contractimpl]
impl IdentityVerifier for SimpleIdentityVerifier {
    fn verify_identity(e: &Env, account: &Address) {
<<<<<<< HEAD
        let irs = identity_registry_storage(e);
        let client = IRSClient::new(e, &irs);
        if client.try_stored_identity(account).is_err() {
            panic_with_error!(e, RWAError::IdentityVerificationFailed);
        }
    }

    fn recovery_target(e: &Env, old_account: &Address) -> Option<Address> {
        let irs = identity_registry_storage(e);
=======
        let irs: Address = e.storage().instance().get(&DataKey::Irs).expect("IRS not set");
        let client = IRSClient::new(e, &irs);
        // Panics if identity not registered — which satisfies the
        // IdentityVerifier contract: unverified accounts are rejected.
        client.stored_identity(account);
    }

    fn recovery_target(e: &Env, old_account: &Address) -> Option<Address> {
        let irs: Address = e.storage().instance().get(&DataKey::Irs).expect("IRS not set");
>>>>>>> 3cfdd149 (test(rwa): add compliance module integration test suite)
        let client = IRSClient::new(e, &irs);
        client.get_recovered_to(old_account)
    }

<<<<<<< HEAD
    #[only_role(operator, "admin")]
    fn set_claim_topics_and_issuers(e: &Env, claim_topics_and_issuers: Address, operator: Address) {
        e.storage().instance().set(&DataKey::ClaimTopicsAndIssuers, &claim_topics_and_issuers);
        emit_claim_topics_and_issuers_set(e, &claim_topics_and_issuers);
=======
    fn set_claim_topics_and_issuers(
        e: &Env,
        claim_topics_and_issuers: Address,
        _operator: Address,
    ) {
        e.storage().instance().set(&DataKey::ClaimTopicsAndIssuers, &claim_topics_and_issuers);
>>>>>>> 3cfdd149 (test(rwa): add compliance module integration test suite)
    }

    fn claim_topics_and_issuers(e: &Env) -> Address {
        e.storage()
            .instance()
            .get(&DataKey::ClaimTopicsAndIssuers)
<<<<<<< HEAD
            .unwrap_or_else(|| panic_with_error!(e, RWAError::ClaimTopicsAndIssuersNotSet))
    }
}

#[contractimpl(contracttrait)]
impl AccessControl for SimpleIdentityVerifier {}
=======
            .expect("ClaimTopicsAndIssuers not set")
    }
}
>>>>>>> 3cfdd149 (test(rwa): add compliance module integration test suite)
