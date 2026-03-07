#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};
use stellar_tokens::rwa::compliance_modules::{
    common::set_compliance_address, country_allow::CountryAllow,
};

#[contract]
pub struct CountryAllowContract;

#[contractimpl]
impl CountryAllowContract {
    pub fn __constructor(e: &Env, compliance: Address) {
        set_compliance_address(e, &compliance);
    }
}

#[contractimpl(contracttrait)]
impl CountryAllow for CountryAllowContract {}
