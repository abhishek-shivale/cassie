use anchor_lang::prelude::*;

#[error_code]
pub enum CassieError {
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
    UnauthorizedAdmin
}
