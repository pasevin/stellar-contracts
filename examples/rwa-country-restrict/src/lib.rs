#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};
use stellar_tokens::rwa::compliance_modules::{
    common::set_compliance_address, country_restrict::CountryRestrict,
};

#[contract]
pub struct CountryRestrictContract;

#[contractimpl]
impl CountryRestrictContract {
    pub fn __constructor(e: &Env, compliance: Address) {
        set_compliance_address(e, &compliance);
    }
}

#[contractimpl(contracttrait)]
impl CountryRestrict for CountryRestrictContract {}
