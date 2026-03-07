#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};
use stellar_tokens::rwa::{
    compliance::ComplianceHook,
    compliance_modules::{common::set_compliance_address, max_balance::MaxBalance},
};

#[contract]
pub struct MaxBalanceContract;

#[contractimpl]
impl MaxBalanceContract {
    pub fn __constructor(e: &Env, compliance: Address) {
        set_compliance_address(e, &compliance);
    }
}

#[contractimpl(contracttrait)]
impl MaxBalance for MaxBalanceContract {}
