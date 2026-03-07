#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};
use stellar_tokens::rwa::compliance_modules::{
    common::set_compliance_address, transfer_restrict::TransferRestrict,
};

#[contract]
pub struct TransferRestrictContract;

#[contractimpl]
impl TransferRestrictContract {
    pub fn __constructor(e: &Env, compliance: Address) {
        set_compliance_address(e, &compliance);
    }
}

#[contractimpl(contracttrait)]
impl TransferRestrict for TransferRestrictContract {}
