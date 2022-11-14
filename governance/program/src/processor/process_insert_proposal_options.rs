//! Program state processor

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};
use spl_governance_tools::account::AccountMaxSize;

use crate::{
    error::GovernanceError,
    state::{
        governance::get_governance_data_for_realm,
        proposal::{get_proposal_data_for_governance, OptionVoteResult, ProposalOption},
        realm::get_realm_data_for_governing_token_mint,
        token_owner_record::get_token_owner_record_data_for_realm,
        vote_record::VoteKind,
    },
};

/// Processes InsertProposalOption
pub fn process_insert_proposal_options(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    options: Vec<String>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let realm_info = next_account_info(account_info_iter)?; // 0
    let governance_info = next_account_info(account_info_iter)?; // 1
    let proposal_info = next_account_info(account_info_iter)?; // 2
    let proposal_owner_record_info = next_account_info(account_info_iter)?; // 3
    let governing_token_mint_info = next_account_info(account_info_iter)?; // 4
    let governance_authority_info = next_account_info(account_info_iter)?; // 5

    let mut proposal_data =
        get_proposal_data_for_governance(program_id, proposal_info, governance_info.key)?;
    proposal_data.assert_can_insert_proposal_options()?;

    let realm_data = get_realm_data_for_governing_token_mint(
        program_id,
        realm_info,
        governing_token_mint_info.key,
    )?;

    let governance_data =
        get_governance_data_for_realm(program_id, governance_info, realm_info.key)?;

    governance_data.assert_governing_token_mint_can_vote(
        &realm_data,
        governing_token_mint_info.key,
        &VoteKind::Electorate,
    )?;

    let proposal_owner_record_data = get_token_owner_record_data_for_realm(
        program_id,
        proposal_owner_record_info,
        realm_info.key,
    )?;

    // Proposal owner (TokenOwner) or its governance_delegate must sign this transaction
    proposal_owner_record_data
        .assert_token_owner_or_delegate_is_signer(governance_authority_info)?;

    let mut proposal_data_size = if let Some(proposal_data_size) = proposal_data.get_max_size() {
        Ok(proposal_data_size)
    } else {
        Err(GovernanceError::CannotCalculateSizeOfProposalData)
    }?;
    let proposal_account_size = proposal_info.data.borrow().len();

    for option_str in options {
        let po = ProposalOption {
            label: option_str.to_string(),
            vote_weight: 0,
            vote_result: OptionVoteResult::None,
            transactions_executed_count: 0,
            transactions_count: 0,
            transactions_next_index: 0,
        };
        let new_data_size = po.get_max_size().unwrap();
        proposal_data_size += po.get_max_size().unwrap();
        if proposal_data_size > proposal_account_size {
            msg!(
                "new size: {}, current size: {}, account size: {}",
                new_data_size,
                proposal_data_size,
                proposal_account_size
            );
            return Err(GovernanceError::InsertProposalOptionsDataExceedsAccountSize.into());
        }
        proposal_data.options.push(po);
    }

    proposal_data.serialize(&mut *proposal_info.data.borrow_mut())?;
    Ok(())
}
