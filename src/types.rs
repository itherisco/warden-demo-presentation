use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct EvaluateRequest {
    pub identity: String,
    pub command: String,
    pub capability: String,
    pub priority: f64,
    pub reward: f64,
    pub risk: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChallengeRequest {
    pub challenge_id: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Gate {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub warden: String,
    pub mode: String,
}

#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub service: String,
    pub version: String,
    pub warden_sdk_version: String,
    pub mode: String,
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub approved: u64,
    pub blocked: u64,
    pub audit_count: u64,
    pub capabilities_count: u64,
}

#[derive(Debug, Serialize)]
pub struct EvaluateResponse {
    pub decision: String,
    pub identity: String,
    pub command: String,
    pub risk: String,
    pub lep_score: f64,
    pub gates: Vec<Gate>,
}

#[derive(Debug, Serialize)]
pub struct BlockedResponse {
    pub decision: String,
    pub identity: String,
    pub command: String,
    pub reason: String,
    pub gates: Vec<Gate>,
}

#[derive(Debug, Serialize)]
pub struct ChallengeResponse {
    pub challenge_id: String,
    pub category: String,
    pub payload: String,
    pub decision: String,
    pub gate: String,
    pub reason_code: String,
    pub audit_id: String,
    pub gates: Vec<Gate>,
}