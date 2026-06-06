use anchor_lang::prelude::*;

//  ----------------------------------------------------
//  |                 Errors                           |
//  ----------------------------------------------------
//  this is error enum use across protocol.
#[error_code]
pub enum CassieError {
    #[msg("Duplicate council member.")]
    DuplicateCouncilMember,
    #[msg("Question already exists.")]
    QuestionAlreadyExists,
    #[msg("Answer window closed.")]
    AnswerWindowClosed,
    #[msg("Answer window active.")]
    AnswerWindowActive,
    #[msg("Invalid state.")]
    InvalidState,
    #[msg("Insufficient stake.")]
    InsufficientStake,
    #[msg("Insufficient bounty.")]
    InsufficientBounty,
    #[msg("Invalid confidence.")]
    InvalidConfidence,
    #[msg("Already answered.")]
    AlreadyAnswered,
    #[msg("Dispute window closed.")]
    DisputeWindowClosed,
    #[msg("Dispute window still active.")]
    DisputeWindowActive,
    #[msg("Council voting window closed.")]
    CouncilWindowClosed,
    #[msg("Not council member.")]
    NotCouncilMember,
    #[msg("Already voted.")]
    AlreadyVoted,
    #[msg("Quorum not reached.")]
    QuorumNotReached,
    #[msg("Already claimed.")]
    AlreadyClaimed,
    #[msg("Callback invocation failed.")]
    CallbackInvocationFailed,
    #[msg("unauthorized admin.")]
    UnauthorizedAdmin,
    #[msg("MAX BPS reached.")]
    MaxBpsReached,
    #[msg("Invalid window timeframe")]
    InvalidWindow,
    #[msg("max council size reached.")]
    MaxCouncilSizeReached,
    #[msg("bounty size can not be lower that this.")]
    BountySizeCanNotBeLower,
    #[msg("council member should not be zero.")]
    CouncilMemberShouldNotBeZero,
    #[msg("Program frozen.")]
    ProgramFrozen,
    #[msg("Invalid claimed dispute outcome.")]
    InvalidDisputeOutcome,
}
