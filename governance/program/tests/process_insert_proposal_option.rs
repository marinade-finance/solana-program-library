#![cfg(feature = "test-sbf")]

mod program_test;

use solana_program_test::tokio;

use program_test::*;
use spl_governance::state::enums::ProposalState;
use spl_governance::{error::GovernanceError, state::proposal::VoteType};
use spl_governance_test_sdk::tools::NopOverride;

/// Prefetch size calculation test calculation
// TODO: calculation dependends on hardcoded values from with_proposal_using_instruction_impl and proposal.rs
fn calculate_proposal_space(options: &Vec<String>) -> usize {
    // ProposalV2: self.name.len() + self.description_link.len() + options_size + 296; options_size is 19 + label length
    let proposal_base_space: usize = "Proposal #X".len() + "Proposal Description".len() + 296;
    let proposal_options_space: usize =
        19 * options.len() + options.iter().map(|o| o.len()).sum::<usize>();
    let proposal_prefetch_space = proposal_base_space + proposal_options_space;
    println!(
        "Test calculated space for proposal account - base space: {}, options space: {}, sum: {}",
        proposal_base_space, proposal_options_space, proposal_prefetch_space
    );
    proposal_prefetch_space
}

#[tokio::test]
async fn test_insert_proposal_option() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_cookie = governance_test.with_realm().await;
    let governed_account_cookie = governance_test.with_governed_account().await;

    let token_owner_record_cookie = governance_test
        .with_community_token_deposit(&realm_cookie)
        .await
        .unwrap();

    let mut governance_cookie = governance_test
        .with_governance(
            &realm_cookie,
            &governed_account_cookie,
            &token_owner_record_cookie,
        )
        .await
        .unwrap();

    // Act
    let options = vec!["Option 1".to_string(), "Option 2".to_string()];
    let proposal_prefetch_space = calculate_proposal_space(&options);

    let mut proposal_cookie = governance_test
        .with_proposal_using_instruction_impl(
            &token_owner_record_cookie,
            &mut governance_cookie,
            vec![],
            true,
            VoteType::SingleChoice,
            Some(proposal_prefetch_space as u64),
            NopOverride,
        )
        .await
        .unwrap();

    // on creaton of the proposal there is no option inserted; no option on creation permitted
    assert_eq!(0, proposal_cookie.account.options.len());

    governance_test
        .with_proposal_options_using_instruction_impl(
            &token_owner_record_cookie,
            &mut governance_cookie,
            &mut proposal_cookie,
            options,
            NopOverride,
        )
        .await
        .unwrap();
    assert_eq!(2, proposal_cookie.account.options.len());

    let proposal_account = governance_test
        .get_proposal_account(&proposal_cookie.address)
        .await;
    assert_eq!(2, proposal_account.options.len());
}

#[tokio::test]
async fn test_insert_multi_proposal_option() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_cookie = governance_test.with_realm().await;
    let governed_account_cookie = governance_test.with_governed_account().await;

    let token_owner_record_cookie = governance_test
        .with_community_token_deposit(&realm_cookie)
        .await
        .unwrap();

    let mut governance_cookie = governance_test
        .with_governance(
            &realm_cookie,
            &governed_account_cookie,
            &token_owner_record_cookie,
        )
        .await
        .unwrap();

    // Act
    let options_set1 = vec!["Option 1".to_string()];
    let options_set2 = vec!["Option 1".to_string()];
    let options_set3 = vec!["Option 3".to_string()];
    let options = [
        options_set1.clone(),
        options_set1.clone(),
        options_set3.clone(),
    ]
    .concat();
    let proposal_prefetch_space = calculate_proposal_space(&options);

    let mut proposal_cookie = governance_test
        .with_proposal_using_instruction_impl(
            &token_owner_record_cookie,
            &mut governance_cookie,
            options_set1,
            true,
            VoteType::SingleChoice,
            Some(proposal_prefetch_space as u64),
            NopOverride,
        )
        .await
        .unwrap();

    assert_eq!(1, proposal_cookie.account.options.len());

    governance_test
        .with_proposal_options_using_instruction_impl(
            &token_owner_record_cookie,
            &mut governance_cookie,
            &mut proposal_cookie,
            options_set2,
            NopOverride,
        )
        .await
        .unwrap();
    assert_eq!(2, proposal_cookie.account.options.len());

    governance_test
        .with_proposal_options_using_instruction_impl(
            &token_owner_record_cookie,
            &mut governance_cookie,
            &mut proposal_cookie,
            options_set3,
            NopOverride,
        )
        .await
        .unwrap();
    assert_eq!(3, proposal_cookie.account.options.len());

    let proposal_account = governance_test
        .get_proposal_account(&proposal_cookie.address)
        .await;
    assert_eq!(3, proposal_account.options.len());
}

#[tokio::test]
async fn test_insert_proposal_option_with_not_editable_proposal_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_cookie = governance_test.with_realm().await;
    let governed_account_cookie = governance_test.with_governed_account().await;

    let token_owner_record_cookie = governance_test
        .with_community_token_deposit(&realm_cookie)
        .await
        .unwrap();

    let mut governance_cookie = governance_test
        .with_governance(
            &realm_cookie,
            &governed_account_cookie,
            &token_owner_record_cookie,
        )
        .await
        .unwrap();

    let options = vec!["Option 1".to_string()];
    let proposal_prefetch_space = calculate_proposal_space(&options);

    let mut proposal_cookie = governance_test
        .with_proposal_using_instruction_impl(
            &token_owner_record_cookie,
            &mut governance_cookie,
            vec![],
            true,
            VoteType::SingleChoice,
            Some(proposal_prefetch_space as u64),
            NopOverride,
        )
        .await
        .unwrap();

    governance_test
        .with_proposal_options_using_instruction_impl(
            &token_owner_record_cookie,
            &mut governance_cookie,
            &mut proposal_cookie,
            options.clone(),
            NopOverride,
        )
        .await
        .unwrap();
    assert_eq!(1, proposal_cookie.account.options.len());

    governance_test
        .sign_off_proposal_by_owner(&proposal_cookie, &token_owner_record_cookie)
        .await
        .unwrap();
    let proposal_account = governance_test
        .get_proposal_account(&proposal_cookie.address)
        .await;
    assert_eq!(ProposalState::Voting, proposal_account.state);

    // Act
    let err = governance_test
        .with_proposal_options_using_instruction_impl(
            &token_owner_record_cookie,
            &mut governance_cookie,
            &mut proposal_cookie,
            options.clone(),
            NopOverride,
        )
        .await
        .err()
        .unwrap();

    assert_eq!(
        err,
        GovernanceError::InvalidStateCannotEditTransactions.into()
    );
}

#[tokio::test]
async fn test_sing_off_without_inserting_proposal_option_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_cookie = governance_test.with_realm().await;
    let governed_account_cookie = governance_test.with_governed_account().await;

    let token_owner_record_cookie = governance_test
        .with_community_token_deposit(&realm_cookie)
        .await
        .unwrap();

    let mut governance_cookie = governance_test
        .with_governance(
            &realm_cookie,
            &governed_account_cookie,
            &token_owner_record_cookie,
        )
        .await
        .unwrap();

    let options = vec!["Option 1".to_string()];
    let proposal_prefetch_space = calculate_proposal_space(&options);

    let proposal_cookie = governance_test
        .with_proposal_using_instruction_impl(
            &token_owner_record_cookie,
            &mut governance_cookie,
            vec![],
            true,
            VoteType::SingleChoice,
            Some(proposal_prefetch_space as u64),
            NopOverride,
        )
        .await
        .unwrap();

    let err = governance_test
        .sign_off_proposal_by_owner(&proposal_cookie, &token_owner_record_cookie)
        .await
        .err()
        .unwrap();

    assert_eq!(
        err,
        GovernanceError::AtLeastOneOptionInProposalRequired.into()
    );
}
