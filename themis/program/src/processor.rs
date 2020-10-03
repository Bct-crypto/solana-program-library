//! Themis program
use crate::{
    error::ThemisError,
    instruction::ThemisInstruction,
    state::{Policies, User},
};
use bn::{Fr, G1};
use elgamal_bn::public::PublicKey;
use solana_sdk::{
    account_info::{next_account_info, AccountInfo},
    program_error::ProgramError,
    pubkey::Pubkey,
};

fn process_initialize_user_account(
    user_info: &AccountInfo,
    public_key: PublicKey,
) -> Result<(), ProgramError> {
    // TODO: verify the program ID
    if let Ok(user) = User::deserialize(&user_info.data.borrow()) {
        if user.is_initialized {
            return Err(ThemisError::AccountInUse.into());
        }
    }
    let user = User::new(public_key);
    user.serialize(&mut user_info.data.borrow_mut())
}

fn process_initialize_policies_account(
    num_scalars: u8,
    policies_info: &AccountInfo,
) -> Result<(), ProgramError> {
    if let Ok(policies) = Policies::deserialize(&policies_info.data.borrow()) {
        if policies.is_initialized {
            return Err(ThemisError::AccountInUse.into());
        }
    }
    let policies = Policies::new(num_scalars);
    policies.serialize(&mut policies_info.data.borrow_mut())
}

fn process_store_policies(
    scalars: Vec<(u8, Fr)>,
    policies_info: &AccountInfo,
) -> Result<(), ProgramError> {
    let mut policies = Policies::deserialize(&policies_info.data.borrow())?;
    for (i, scalar) in scalars {
        policies.scalars[i as usize] = scalar;
    }
    policies.serialize(&mut policies_info.data.borrow_mut())
}

fn process_submit_interactions(
    encrypted_interactions: &[(u8, (G1, G1))],
    user_info: &AccountInfo,
    policies_info: &AccountInfo,
) -> Result<(), ProgramError> {
    let mut user = User::deserialize(&user_info.data.borrow())?;
    let policies = Policies::deserialize(&policies_info.data.borrow())?;
    user.submit_interactions(encrypted_interactions, &policies.scalars);
    user.serialize(&mut user_info.data.borrow_mut())
}

fn process_submit_proof_decryption(
    plaintext: G1,
    announcement: (G1, G1),
    response: Fr,
    user_info: &AccountInfo,
) -> Result<(), ProgramError> {
    let mut user = User::deserialize(&user_info.data.borrow())?;
    user.submit_proof_decryption(plaintext, announcement.0, announcement.1, response);
    user.serialize(&mut user_info.data.borrow_mut())
}

fn process_request_payment(
    encrypted_aggregate: (G1, G1),
    decrypted_aggregate: G1,
    proof_correct_decryption: G1,
    user_info: &AccountInfo,
) -> Result<(), ProgramError> {
    let mut user = User::deserialize(&user_info.data.borrow())?;
    user.request_payment(
        encrypted_aggregate,
        decrypted_aggregate,
        proof_correct_decryption,
    );
    user.serialize(&mut user_info.data.borrow_mut())
}

/// Process the given transaction instruction
pub fn process_instruction<'a>(
    _program_id: &Pubkey,
    account_infos: &'a [AccountInfo<'a>],
    input: &[u8],
) -> Result<(), ProgramError> {
    let account_infos_iter = &mut account_infos.iter();
    let instruction = ThemisInstruction::deserialize(input)?;

    match instruction {
        ThemisInstruction::InitializeUserAccount { public_key } => {
            let user_info = next_account_info(account_infos_iter)?;
            process_initialize_user_account(&user_info, public_key)
        }
        ThemisInstruction::InitializePoliciesAccount { num_scalars } => {
            let policies_info = next_account_info(account_infos_iter)?;
            process_initialize_policies_account(num_scalars, &policies_info)
        }
        ThemisInstruction::StorePolicies { scalars } => {
            let policies_info = next_account_info(account_infos_iter)?;
            process_store_policies(scalars, &policies_info)
        }
        ThemisInstruction::SubmitInteractions {
            encrypted_interactions,
        } => {
            let user_info = next_account_info(account_infos_iter)?;
            let policies_info = next_account_info(account_infos_iter)?;
            process_submit_interactions(&encrypted_interactions, &user_info, &policies_info)
        }
        ThemisInstruction::SubmitProofDecryption {
            plaintext,
            announcement,
            response,
        } => {
            let user_info = next_account_info(account_infos_iter)?;
            process_submit_proof_decryption(plaintext, *announcement, response, &user_info)
        }
        ThemisInstruction::RequestPayment {
            encrypted_aggregate,
            decrypted_aggregate,
            proof_correct_decryption,
        } => {
            let user_info = next_account_info(account_infos_iter)?;
            process_request_payment(
                *encrypted_aggregate,
                decrypted_aggregate,
                proof_correct_decryption,
                &user_info,
            )
        }
    }
}
