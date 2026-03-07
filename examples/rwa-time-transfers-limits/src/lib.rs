#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};
use stellar_tokens::rwa::{
    compliance::ComplianceHook,
    compliance_modules::{
        common::set_compliance_address,
        time_transfers_limits::{Limit, TimeTransfersLimits},
    },
};

#[contract]
pub struct TimeTransfersLimitsContract;

#[contractimpl]
impl TimeTransfersLimitsContract {
    pub fn __constructor(e: &Env, compliance: Address) {
        set_compliance_address(e, &compliance);
    }
}

#[contractimpl(contracttrait)]
impl TimeTransfersLimits for TimeTransfersLimitsContract {}
