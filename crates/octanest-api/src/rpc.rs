use octanest_core::{
    AppError, EchoRequest, EchoResponse, HealthResponse, RpcRequest, RpcResponse, ECHO_MAX_BYTES,
    RPC_PROTOCOL_VERSION,
};
use octanest_db::Database;

pub const VERSION_HEADER: &str = "Octanest-RPC-Version";

pub fn check_version_header(value: Option<&str>) -> Result<(), AppError> {
    match value {
        Some(v) if v.trim() == RPC_PROTOCOL_VERSION.to_string() => Ok(()),
        Some(v) => Err(AppError::new(
            "rpc.version_mismatch",
            format!("expected Octanest-RPC-Version {RPC_PROTOCOL_VERSION}, got {v}"),
        )),
        None => Err(AppError::new(
            "rpc.version_mismatch",
            format!("missing Octanest-RPC-Version header (expected {RPC_PROTOCOL_VERSION})"),
        )),
    }
}

pub async fn dispatch(db: &Database, req: RpcRequest) -> RpcResponse {
    match req.procedure.as_str() {
        "system.health" => {
            let database = db.ping().await.to_string();
            RpcResponse::ok(HealthResponse {
                status: "ok".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                database,
            })
        }
        "system.echo" => {
            let echo: EchoRequest = match serde_json::from_value(req.input) {
                Ok(v) => v,
                Err(e) => {
                    return RpcResponse::err(AppError::new(
                        "rpc.bad_input",
                        format!("invalid echo input: {e}"),
                    ))
                }
            };
            if echo.message.len() > ECHO_MAX_BYTES {
                return RpcResponse::err(AppError::new(
                    "rpc.payload_too_large",
                    format!("echo message exceeds {ECHO_MAX_BYTES} bytes"),
                ));
            }
            RpcResponse::ok(EchoResponse {
                message: echo.message,
            })
        }
        "system.db_probe" => match db.probe().await {
            Ok(result) => RpcResponse::ok(result),
            Err(e) if e == "database not configured" => RpcResponse::err(AppError::new(
                "db.not_configured",
                "no database configured for this instance",
            )),
            Err(e) => {
                tracing::error!("db probe failed: {e}");
                RpcResponse::err(AppError::new("db.probe_failed", "database probe failed"))
            }
        },
        other => RpcResponse::err(AppError::new(
            "rpc.unknown_procedure",
            format!("unknown procedure: {other}"),
        )),
    }
}
