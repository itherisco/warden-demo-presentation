use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

mod types;

use crate::types::{
    BlockedResponse, ChallengeRequest, ChallengeResponse, EvaluateRequest, EvaluateResponse, Gate,
    HealthResponse, StatsResponse, VersionResponse,
};
use chrono::Utc;
use warden_sdk::{
    sandbox::validate_command_security, compute_lep_score, ActionType, KernelAction, RiskLevel,
    WardenKernel,
};

#[derive(Clone)]
struct AppState {
    kernel: Arc<WardenKernel>,
    stats: Arc<std::sync::atomic::AtomicU64>,
}

fn compute_risk_level(risk: f64) -> RiskLevel {
    if risk < 0.3 {
        RiskLevel::Low
    } else if risk < 0.6 {
        RiskLevel::Medium
    } else if risk < 0.9 {
        RiskLevel::High
    } else {
        RiskLevel::Critical
    }
}

fn build_gate_trace_from_reason(reason: &str, request: &EvaluateRequest) -> Vec<Gate> {
    let mut gates = Vec::new();

    if reason.contains("UNKNOWN_IDENTITY") {
        gates.push(Gate {
            name: "Identity".to_string(),
            status: "FAIL".to_string(),
        });
        gates.push(Gate {
            name: "Audit".to_string(),
            status: "RECORDED".to_string(),
        });
        return gates;
    }

    if reason.contains("DENIED") {
        gates.push(Gate {
            name: "Identity".to_string(),
            status: "PASS".to_string(),
        });
        gates.push(Gate {
            name: "Capability".to_string(),
            status: "FAIL".to_string(),
        });
        gates.push(Gate {
            name: "Audit".to_string(),
            status: "RECORDED".to_string(),
        });
        return gates;
    }

    if reason.contains("SHELL_INJECTION") || validate_command_security(&request.command).is_err() {
        gates.push(Gate {
            name: "Identity".to_string(),
            status: "PASS".to_string(),
        });
        gates.push(Gate {
            name: "Capability".to_string(),
            status: "PASS".to_string(),
        });
        gates.push(Gate {
            name: "Shell Injection".to_string(),
            status: "FAIL".to_string(),
        });
        gates.push(Gate {
            name: "Audit".to_string(),
            status: "RECORDED".to_string(),
        });
        return gates;
    }

    if reason.contains("VETOED") {
        let _lep_score = compute_lep_score(request.priority, request.reward, request.risk).0;
        gates.push(Gate {
            name: "Identity".to_string(),
            status: "PASS".to_string(),
        });
        gates.push(Gate {
            name: "Capability".to_string(),
            status: "PASS".to_string(),
        });
        gates.push(Gate {
            name: "Shell Injection".to_string(),
            status: "PASS".to_string(),
        });
        gates.push(Gate {
            name: "LEP".to_string(),
            status: "FAIL".to_string(),
        });
        gates.push(Gate {
            name: "Audit".to_string(),
            status: "RECORDED".to_string(),
        });
        return gates;
    }

    gates.push(Gate {
        name: "Identity".to_string(),
        status: "PASS".to_string(),
    });
    gates.push(Gate {
        name: "Capability".to_string(),
        status: "PASS".to_string(),
    });
    gates.push(Gate {
        name: "Shell Injection".to_string(),
        status: "PASS".to_string(),
    });
    gates.push(Gate {
        name: "LEP".to_string(),
        status: "PASS".to_string(),
    });
    gates.push(Gate {
        name: "Audit".to_string(),
        status: "RECORDED".to_string(),
    });
    gates
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        warden: "ready".to_string(),
        mode: "demo".to_string(),
    })
}

async fn version_handler() -> Json<VersionResponse> {
    Json(VersionResponse {
        service: "warden-demo-server".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        warden_sdk_version: "1.0.0".to_string(),
        mode: "demo".to_string(),
    })
}

async fn stats_handler(State(state): State<AppState>) -> Json<StatsResponse> {
    let stats = state.kernel.get_stats().await;
    Json(StatsResponse {
        approved: stats.approved,
        blocked: stats.blocked,
        audit_count: stats.audit_warnings,
        capabilities_count: stats.capabilities_count,
    })
}

async fn evaluate_handler(
    State(state): State<AppState>,
    Json(request): Json<EvaluateRequest>,
) -> Result<Json<EvaluateResponse>, Json<BlockedResponse>> {
    let shell_safe = validate_command_security(&request.command).is_ok();

    if !shell_safe {
        let gates = vec![
            Gate {
                name: "Identity".to_string(),
                status: "PASS".to_string(),
            },
            Gate {
                name: "Capability".to_string(),
                status: "PASS".to_string(),
            },
            Gate {
                name: "Shell Injection".to_string(),
                status: "FAIL".to_string(),
            },
            Gate {
                name: "Audit".to_string(),
                status: "RECORDED".to_string(),
            },
        ];
        return Err(Json(BlockedResponse {
            decision: "BLOCKED".to_string(),
            identity: request.identity.clone(),
            command: request.command.clone(),
            reason: "SHELL_INJECTION".to_string(),
            gates,
        }));
    }

    let action = KernelAction {
        id: Uuid::new_v4().to_string(),
        action_type: match request.capability.as_str() {
            "execute" => ActionType::Execute,
            "read_file" => ActionType::ReadFile,
            "network_request" => ActionType::NetworkRequest,
            _ => ActionType::QueryExternal,
        },
        target: request.command.clone(),
        payload: None,
        requested_by: request.identity.clone(),
        risk_level: compute_risk_level(request.risk),
        timestamp: Utc::now(),
        priority: Some(request.priority),
        reward: Some(request.reward),
        risk: Some(request.risk),
    };

    let result = state.kernel.approve(action).await;

    if result.approved {
        let gates = build_gate_trace_from_reason(&result.reason, &request);
        let _lep_score = compute_lep_score(request.priority, request.reward, request.risk).0;
        state.stats.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(Json(EvaluateResponse {
            decision: "APPROVED".to_string(),
            identity: request.identity,
            command: request.command,
            risk: match compute_risk_level(request.risk) {
                RiskLevel::Low => "LOW",
                RiskLevel::Medium => "MEDIUM",
                RiskLevel::High => "HIGH",
                RiskLevel::Critical => "CRITICAL",
            }
            .to_string(),
            lep_score: _lep_score,
            gates,
        }))
    } else {
        let reason = result.reason.clone();
        let gates = build_gate_trace_from_reason(&reason, &request);
        Err(Json(BlockedResponse {
            decision: "BLOCKED".to_string(),
            identity: request.identity,
            command: request.command,
            reason,
            gates,
        }))
    }
}

async fn challenge_handler(
    State(state): State<AppState>,
    Json(request): Json<ChallengeRequest>,
) -> Json<ChallengeResponse> {
    let challenges: std::collections::HashMap<&str, (&str, &str, &str)> = [
        (
            "command-injection-01",
            ("shell_injection", "echo hello; cat /etc/passwd", "DETECTED"),
        ),
        (
            "path-traversal-01",
            ("path_traversal", "cat ../../../etc/passwd", "DETECTED"),
        ),
        (
            "unknown-identity-01",
            ("identity", "ls", "UNKNOWN_IDENTITY"),
        ),
        (
            "capability-escalation-01",
            ("capability", "rm -rf /", "DENIED"),
        ),
        (
            "low-lep-01",
            ("lep", "echo test", "VETOED"),
        ),
        (
            "env-abuse-01",
            ("env_injection", "echo $PATH", "DETECTED"),
        ),
        (
            "network-bypass-01",
            ("network", "curl http://private.internal", "DETECTED"),
        ),
    ]
    .iter()
    .cloned()
    .collect();

    let challenge_id = request.challenge_id.as_str();

    if let Some((category, payload, reason_code)) = challenges.get(challenge_id) {
        let identity = match *reason_code {
            "UNKNOWN_IDENTITY" => "attacker",
            "DENIED" => "readonly-agent",
            _ => "demo-agent",
        };

        let capability = "execute";

        let eval_request = EvaluateRequest {
            identity: identity.to_string(),
            command: payload.to_string(),
            capability: capability.to_string(),
            priority: if *reason_code == "VETOED" { 0.1 } else { 1.0 },
            reward: if *reason_code == "VETOED" { 0.1 } else { 1.0 },
            risk: if *reason_code == "VETOED" { 0.9 } else { 0.1 },
        };

        let action = KernelAction {
            id: Uuid::new_v4().to_string(),
            action_type: match capability {
                "execute" => ActionType::Execute,
                "read_file" => ActionType::ReadFile,
                "network_request" => ActionType::NetworkRequest,
                _ => ActionType::QueryExternal,
            },
            target: payload.to_string(),
            payload: None,
            requested_by: identity.to_string(),
            risk_level: compute_risk_level(eval_request.risk),
            timestamp: Utc::now(),
            priority: Some(eval_request.priority),
            reward: Some(eval_request.reward),
            risk: Some(eval_request.risk),
        };

        let shell_safe = validate_command_security(payload).is_ok();
        let result = if !shell_safe {
            warden_sdk::ApprovalResult {
                action_id: action.id.clone(),
                approved: false,
                reason: "SHELL_INJECTION".to_string(),
                warden_signature: None,
                timestamp: chrono::Utc::now(),
            }
        } else {
            state.kernel.approve(action).await
        };

        let gates = if !shell_safe {
            vec![
                Gate {
                    name: "Identity".to_string(),
                    status: "PASS".to_string(),
                },
                Gate {
                    name: "Capability".to_string(),
                    status: "PASS".to_string(),
                },
                Gate {
                    name: "Shell Injection".to_string(),
                    status: "FAIL".to_string(),
                },
                Gate {
                    name: "Audit".to_string(),
                    status: "RECORDED".to_string(),
                },
            ]
        } else {
            build_gate_trace_from_reason(&result.reason, &eval_request)
        };

        let gate = match *reason_code {
            "UNKNOWN_IDENTITY" => "IDENTITY",
            "DENIED" => "CAPABILITY",
            "DETECTED" | "VETOED" => "SANDBOX",
            _ => "POLICY",
        };

        Json(ChallengeResponse {
            challenge_id: challenge_id.to_string(),
            category: category.to_string(),
            payload: payload.to_string(),
            decision: if result.approved { "APPROVED" } else { "BLOCKED" }.to_string(),
            gate: gate.to_string(),
            reason_code: reason_code.to_string(),
            audit_id: Uuid::new_v4().to_string(),
            gates,
        })
    } else {
        Json(ChallengeResponse {
            challenge_id: challenge_id.to_string(),
            category: "unknown".to_string(),
            payload: "".to_string(),
            decision: "BLOCKED".to_string(),
            gate: "UNKNOWN".to_string(),
            reason_code: "UNKNOWN_CHALLENGE".to_string(),
            audit_id: Uuid::new_v4().to_string(),
            gates: vec![Gate {
                name: "Audit".to_string(),
                status: "RECORDED".to_string(),
            }],
        })
    }
}

fn build_app(kernel: WardenKernel) -> Router {
    let state = AppState {
        kernel: Arc::new(kernel),
        stats: Arc::new(std::sync::atomic::AtomicU64::new(0)),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_handler))
        .route("/version", get(version_handler))
        .route("/stats", get(stats_handler))
        .route("/evaluate", post(evaluate_handler))
        .route("/challenge", post(challenge_handler))
        .with_state(state)
        .layer(cors)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let kernel = WardenKernel::new()?;

    kernel
        .register_identity("demo-agent", vec![ActionType::Execute, ActionType::ReadFile])
        .await?;
    kernel
        .register_identity("readonly-agent", vec![ActionType::ReadFile])
        .await?;

    let app = build_app(kernel);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = SocketAddr::from(([0, 0, 0, 0], port.parse()?));

    println!("Server listening on {}", addr);
    axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app.into_make_service(),
    )
    .await?;

    Ok(())
}