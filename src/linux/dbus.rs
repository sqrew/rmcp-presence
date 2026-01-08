//! Generic D-Bus access

use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;
use zbus::{
    proxy::Proxy,
    zvariant::{OwnedValue, Value},
    Connection,
};

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BusParams {
    #[schemars(description = "Use session bus instead of system bus (default: false = system bus)")]
    #[serde(default)]
    pub session: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IntrospectParams {
    #[schemars(description = "D-Bus service name (e.g., \"org.freedesktop.NetworkManager\")")]
    pub destination: String,
    #[schemars(description = "Object path (e.g., \"/org/freedesktop/NetworkManager\")")]
    pub path: String,
    #[schemars(description = "Use session bus instead of system bus (default: false)")]
    #[serde(default)]
    pub session: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MethodParams {
    #[schemars(description = "D-Bus service name (e.g., \"org.freedesktop.NetworkManager\")")]
    pub destination: String,
    #[schemars(description = "Object path (e.g., \"/org/freedesktop/NetworkManager\")")]
    pub path: String,
    #[schemars(description = "Interface name (e.g., \"org.freedesktop.NetworkManager\")")]
    pub interface: String,
    #[schemars(description = "Method name to call")]
    pub method: String,
    #[schemars(
        description = "Arguments as JSON array (e.g., [\"arg1\", 42, true]). Supports strings, numbers, booleans."
    )]
    #[serde(default)]
    pub args: Option<String>,
    #[schemars(description = "Use session bus instead of system bus (default: false)")]
    #[serde(default)]
    pub session: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PropertyParams {
    #[schemars(description = "D-Bus service name (e.g., \"org.freedesktop.NetworkManager\")")]
    pub destination: String,
    #[schemars(description = "Object path (e.g., \"/org/freedesktop/NetworkManager\")")]
    pub path: String,
    #[schemars(description = "Interface name containing the property")]
    pub interface: String,
    #[schemars(description = "Property name to get")]
    pub property: String,
    #[schemars(description = "Use session bus instead of system bus (default: false)")]
    #[serde(default)]
    pub session: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetPropertyParams {
    #[schemars(description = "D-Bus service name (e.g., \"org.freedesktop.NetworkManager\")")]
    pub destination: String,
    #[schemars(description = "Object path (e.g., \"/org/freedesktop/NetworkManager\")")]
    pub path: String,
    #[schemars(description = "Interface name containing the property")]
    pub interface: String,
    #[schemars(description = "Property name to set")]
    pub property: String,
    #[schemars(description = "Value as JSON (e.g., \"string\", 42, true)")]
    pub value: String,
    #[schemars(description = "Use session bus instead of system bus (default: false)")]
    #[serde(default)]
    pub session: bool,
}

// === Helper Functions ===

async fn get_connection(session: bool) -> Result<Connection, String> {
    if session {
        Connection::session()
            .await
            .map_err(|e| format!("Failed to connect to session bus: {}", e))
    } else {
        Connection::system()
            .await
            .map_err(|e| format!("Failed to connect to system bus: {}", e))
    }
}

fn json_to_value(json: &serde_json::Value) -> Result<Value<'static>, String> {
    match json {
        serde_json::Value::Null => Ok(Value::Str("".into())),
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::I64(i))
            } else if let Some(u) = n.as_u64() {
                Ok(Value::U64(u))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::F64(f))
            } else {
                Err("Invalid number".into())
            }
        }
        serde_json::Value::String(s) => Ok(Value::Str(s.clone().into())),
        serde_json::Value::Array(arr) => {
            let values: Result<Vec<Value<'static>>, String> =
                arr.iter().map(json_to_value).collect();
            Ok(Value::Array(values?.into()))
        }
        serde_json::Value::Object(_) => Err("Object types not supported in D-Bus args".into()),
    }
}

fn owned_value_to_string(value: &OwnedValue) -> String {
    format!("{:?}", value)
}

// === Tool Functions ===

pub async fn list_names(params: BusParams) -> Result<CallToolResult, McpError> {
    let conn = match get_connection(params.session).await {
        Ok(c) => c,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let proxy = match Proxy::new(
        &conn,
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to create proxy: {}",
                e
            ))]))
        }
    };

    let names: Vec<String> = match proxy.call("ListNames", &()).await {
        Ok(n) => n,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to list names: {}",
                e
            ))]))
        }
    };

    let bus_type = if params.session { "session" } else { "system" };
    let mut output = format!("{} D-Bus names on {} bus:\n", names.len(), bus_type);

    let well_known: Vec<&String> = names.iter().filter(|n| !n.starts_with(':')).collect();
    let unique_count = names.len() - well_known.len();

    output.push_str(&format!("\nWell-known names ({}):\n", well_known.len()));
    for name in well_known {
        output.push_str(&format!("  {}\n", name));
    }
    output.push_str(&format!(
        "\n({} unique connection names hidden)\n",
        unique_count
    ));

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn introspect(params: IntrospectParams) -> Result<CallToolResult, McpError> {
    let conn = match get_connection(params.session).await {
        Ok(c) => c,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let proxy = match Proxy::new(
        &conn,
        params.destination.as_str(),
        params.path.as_str(),
        "org.freedesktop.DBus.Introspectable",
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to create proxy: {}",
                e
            ))]))
        }
    };

    let xml: String = match proxy.call("Introspect", &()).await {
        Ok(x) => x,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to introspect {}: {}",
                params.destination, e
            ))]))
        }
    };

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Introspection of {} at {}:\n\n{}",
        params.destination, params.path, xml
    ))]))
}

pub async fn call_method(params: MethodParams) -> Result<CallToolResult, McpError> {
    let conn = match get_connection(params.session).await {
        Ok(c) => c,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let proxy = match Proxy::new(
        &conn,
        params.destination.as_str(),
        params.path.as_str(),
        params.interface.as_str(),
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to create proxy: {}",
                e
            ))]))
        }
    };

    let method_name = params.method.as_str();

    let result: Result<OwnedValue, zbus::Error> = if let Some(args_json) = &params.args {
        let json_value: serde_json::Value = match serde_json::from_str(args_json) {
            Ok(v) => v,
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Invalid JSON args: {}",
                    e
                ))]))
            }
        };

        match &json_value {
            serde_json::Value::Array(arr) if arr.is_empty() => proxy.call(method_name, &()).await,
            serde_json::Value::Array(arr) if arr.len() == 1 => match json_to_value(&arr[0]) {
                Ok(v) => proxy.call(method_name, &(v,)).await,
                Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
            },
            serde_json::Value::Array(arr) => {
                let values: Result<Vec<Value>, String> = arr.iter().map(json_to_value).collect();
                match values {
                    Ok(vals) => match vals.len() {
                        2 => proxy.call(method_name, &(&vals[0], &vals[1])).await,
                        3 => {
                            proxy
                                .call(method_name, &(&vals[0], &vals[1], &vals[2]))
                                .await
                        }
                        _ => {
                            return Ok(CallToolResult::success(vec![Content::text(
                                "Only 0-3 arguments currently supported",
                            )]))
                        }
                    },
                    Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
                }
            }
            _ => {
                return Ok(CallToolResult::success(vec![Content::text(
                    "Args must be a JSON array",
                )]))
            }
        }
    } else {
        proxy.call(method_name, &()).await
    };

    match result {
        Ok(value) => {
            let output = format!(
                "Called {}.{}() on {}{}:\n\nResult: {}",
                params.interface,
                params.method,
                params.destination,
                params.path,
                owned_value_to_string(&value)
            );
            Ok(CallToolResult::success(vec![Content::text(output)]))
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Method call failed: {}",
            e
        ))])),
    }
}

pub async fn get_property(params: PropertyParams) -> Result<CallToolResult, McpError> {
    let conn = match get_connection(params.session).await {
        Ok(c) => c,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let proxy = match Proxy::new(
        &conn,
        params.destination.as_str(),
        params.path.as_str(),
        "org.freedesktop.DBus.Properties",
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to create proxy: {}",
                e
            ))]))
        }
    };

    let value: OwnedValue = match proxy
        .call(
            "Get",
            &(params.interface.as_str(), params.property.as_str()),
        )
        .await
    {
        Ok(v) => v,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to get property: {}",
                e
            ))]))
        }
    };

    let output = format!(
        "{}.{} on {}{}:\n\n{}",
        params.interface,
        params.property,
        params.destination,
        params.path,
        owned_value_to_string(&value)
    );

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn set_property(params: SetPropertyParams) -> Result<CallToolResult, McpError> {
    let conn = match get_connection(params.session).await {
        Ok(c) => c,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let proxy = match Proxy::new(
        &conn,
        params.destination.as_str(),
        params.path.as_str(),
        "org.freedesktop.DBus.Properties",
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to create proxy: {}",
                e
            ))]))
        }
    };

    let json_value: serde_json::Value = match serde_json::from_str(&params.value) {
        Ok(v) => v,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Invalid JSON value: {}",
                e
            ))]))
        }
    };

    let value = match json_to_value(&json_value) {
        Ok(v) => v,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let variant_value = Value::new(value);

    let result: Result<(), zbus::Error> = proxy
        .call(
            "Set",
            &(
                params.interface.as_str(),
                params.property.as_str(),
                variant_value,
            ),
        )
        .await;

    match result {
        Ok(()) => {
            let output = format!(
                "Set {}.{} = {} on {}{}",
                params.interface, params.property, params.value, params.destination, params.path,
            );
            Ok(CallToolResult::success(vec![Content::text(output)]))
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to set property: {}",
            e
        ))])),
    }
}
