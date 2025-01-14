use serde::Serialize;
use std::collections::HashMap;

/// Generate Rust code for a `Lazy<HashMap<K, V>>`, including nested structures.
///
/// Note: The consume of this will depend once_cell crate: https://docs.rs/once_cell/latest/once_cell/
///
/// # Arguments
/// - `name`: Name of the Rust variable to generate.
/// - `map`: The `HashMap` to convert to Rust code.
///
/// # Returns
/// - A `String` containing the generated Rust code.
pub fn generate_hashmap_code<K, V>(name: &str, map: &HashMap<K, V>) -> String
where
    K: Serialize,
    V: Serialize,
{
    let key_type = sanitize_rust_type_name::<K>();
    let value_type = sanitize_rust_type_name::<V>();

    let mut code = String::new();

    // Declare the Lazy static variable using once_cell
    code.push_str(&format!(
        "pub static {}: Lazy<HashMap<{}, {}>> = Lazy::new(|| {{\n",
        name, key_type, value_type
    ));

    // Generate a single `HashMap::from` with all entries
    let entries = map
        .iter()
        .map(|(key, value)| {
            let key_literal = serialize_to_rust_literal(key);
            let value_literal = serialize_to_rust_literal(value);
            format!("({}, {})", key_literal, value_literal)
        })
        .collect::<Vec<_>>()
        .join(", ");

    // Use `HashMap::from` for optimal initialization
    code.push_str(&format!("    HashMap::from([{}])\n", entries));

    // Finalize the block
    code.push_str("});\n");
    code
}

/// Serialize a value into Rust-compatible literal code.
///
/// Supports strings, numbers, vectors, and nested `HashMap` structures.
///
/// This does *not* work with structs as `HashMap` values.
fn serialize_to_rust_literal<T: Serialize>(value: &T) -> String {
    if let Ok(serialized) = serde_json::to_value(value) {
        match serialized {
            serde_json::Value::String(s) => format!("\"{}\".to_string()", s), // Convert &str to String
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Array(arr) => {
                let elements = arr
                    .iter()
                    .map(|v| serialize_to_rust_literal(v))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("vec![{}]", elements)
            }
            serde_json::Value::Object(map) => {
                let entries = map
                    .into_iter()
                    .map(|(k, v)| {
                        format!(
                            "({}, {})",
                            serialize_to_rust_literal(&k),
                            serialize_to_rust_literal(&v)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("HashMap::from([{}])", entries)
            }
            _ => "null".to_string(),
        }
    } else {
        "null".to_string()
    }
}

/// Sanitize the Rust type name to avoid private module paths.
fn sanitize_rust_type_name<T>() -> String {
    let full_name = std::any::type_name::<T>();
    full_name
        .replace("std::collections::hash::map::", "std::collections::")
        .replace("std::collections::hash_map::", "std::collections::")
        .replace("alloc::vec::", "")
        .replace("alloc::string::", "")
}
