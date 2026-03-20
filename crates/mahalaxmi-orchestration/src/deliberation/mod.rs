//! Adversarial multi-agent manager deliberation.
//!
//! Implements the three-turn Proposer/Challenger/Synthesizer protocol that
//! drives adversarial consensus across domain-specific manager teams.

pub mod api_client;
pub mod cross_team;
pub mod discovery;
pub mod gate;
pub mod team;
