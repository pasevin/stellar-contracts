#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};
use stellar_tokens::rwa::{
    compliance::ComplianceHook,
    compliance_modules::{common::set_compliance_address, supply_limit::SupplyLimit},
};

#[contract]
pub struct SupplyLimitContract;

#[contractimpl]
impl SupplyLimitContract {
    pub fn __constructor(e: &Env, compliance: Address) {
        set_compliance_address(e, &compliance);
    }
}

#[contractimpl(contracttrait)]
impl SupplyLimit for SupplyLimitContract {}
