#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};
use stellar_tokens::rwa::{
    compliance::ComplianceHook,
    compliance_modules::{
        common::set_compliance_address,
        initial_lockup_period::{InitialLockupPeriod, LockedTokens},
    },
};

#[contract]
pub struct InitialLockupPeriodContract;

#[contractimpl]
impl InitialLockupPeriodContract {
    pub fn __constructor(e: &Env, compliance: Address) {
        set_compliance_address(e, &compliance);
    }
}

#[contractimpl(contracttrait)]
impl InitialLockupPeriod for InitialLockupPeriodContract {}
