//! XML-RPC utilities for OpenNebula API
//!
//! OpenNebula uses XML-RPC for its API. This module provides utilities
//! for encoding requests and decoding responses.

use anyhow::{Context, Result};
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use serde_json::{Map, Value};
use std::io::Cursor;

/// Resolve a general entity reference name to its character
fn resolve_entity(name: &str) -> &str {
    match name {
        "lt" => "<",
        "gt" => ">",
        "amp" => "&",
        "apos" => "'",
        "quot" => "\"",
        _ => "",
    }
}

/// Build an XML-RPC method call
pub fn build_method_call(method: &str, params: &[XmlRpcValue]) -> Result<String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // XML declaration
    writer
        .write_event(Event::Decl(quick_xml::events::BytesDecl::new(
            "1.0", None, None,
        )))
        .context("Failed to write XML declaration")?;

    // methodCall
    writer
        .write_event(Event::Start(BytesStart::new("methodCall")))
        .context("Failed to write methodCall start")?;

    // methodName
    writer
        .write_event(Event::Start(BytesStart::new("methodName")))
        .context("Failed to write methodName start")?;
    writer
        .write_event(Event::Text(BytesText::new(method)))
        .context("Failed to write method name")?;
    writer
        .write_event(Event::End(BytesEnd::new("methodName")))
        .context("Failed to write methodName end")?;

    // params
    writer
        .write_event(Event::Start(BytesStart::new("params")))
        .context("Failed to write params start")?;

    for param in params {
        writer
            .write_event(Event::Start(BytesStart::new("param")))
            .context("Failed to write param start")?;
        write_value(&mut writer, param)?;
        writer
            .write_event(Event::End(BytesEnd::new("param")))
            .context("Failed to write param end")?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("params")))
        .context("Failed to write params end")?;

    writer
        .write_event(Event::End(BytesEnd::new("methodCall")))
        .context("Failed to write methodCall end")?;

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).context("Failed to convert XML to UTF-8")
}

/// XML-RPC value types
#[derive(Debug, Clone)]
pub enum XmlRpcValue {
    String(String),
    Int(i32),
    Boolean(bool),
    Double(f64),
    Array(Vec<XmlRpcValue>),
    Struct(Vec<(String, XmlRpcValue)>),
}

impl From<&str> for XmlRpcValue {
    fn from(s: &str) -> Self {
        XmlRpcValue::String(s.to_string())
    }
}

impl From<String> for XmlRpcValue {
    fn from(s: String) -> Self {
        XmlRpcValue::String(s)
    }
}

impl From<i32> for XmlRpcValue {
    fn from(i: i32) -> Self {
        XmlRpcValue::Int(i)
    }
}

impl From<bool> for XmlRpcValue {
    fn from(b: bool) -> Self {
        XmlRpcValue::Boolean(b)
    }
}

fn write_value<W: std::io::Write>(writer: &mut Writer<W>, value: &XmlRpcValue) -> Result<()> {
    writer
        .write_event(Event::Start(BytesStart::new("value")))
        .context("Failed to write value start")?;

    match value {
        XmlRpcValue::String(s) => {
            writer
                .write_event(Event::Start(BytesStart::new("string")))
                .context("Failed to write string start")?;
            writer
                .write_event(Event::Text(BytesText::new(s)))
                .context("Failed to write string value")?;
            writer
                .write_event(Event::End(BytesEnd::new("string")))
                .context("Failed to write string end")?;
        }
        XmlRpcValue::Int(i) => {
            writer
                .write_event(Event::Start(BytesStart::new("int")))
                .context("Failed to write int start")?;
            writer
                .write_event(Event::Text(BytesText::new(&i.to_string())))
                .context("Failed to write int value")?;
            writer
                .write_event(Event::End(BytesEnd::new("int")))
                .context("Failed to write int end")?;
        }
        XmlRpcValue::Boolean(b) => {
            writer
                .write_event(Event::Start(BytesStart::new("boolean")))
                .context("Failed to write boolean start")?;
            writer
                .write_event(Event::Text(BytesText::new(if *b { "1" } else { "0" })))
                .context("Failed to write boolean value")?;
            writer
                .write_event(Event::End(BytesEnd::new("boolean")))
                .context("Failed to write boolean end")?;
        }
        XmlRpcValue::Double(d) => {
            writer
                .write_event(Event::Start(BytesStart::new("double")))
                .context("Failed to write double start")?;
            writer
                .write_event(Event::Text(BytesText::new(&d.to_string())))
                .context("Failed to write double value")?;
            writer
                .write_event(Event::End(BytesEnd::new("double")))
                .context("Failed to write double end")?;
        }
        XmlRpcValue::Array(arr) => {
            writer
                .write_event(Event::Start(BytesStart::new("array")))
                .context("Failed to write array start")?;
            writer
                .write_event(Event::Start(BytesStart::new("data")))
                .context("Failed to write data start")?;
            for item in arr {
                write_value(writer, item)?;
            }
            writer
                .write_event(Event::End(BytesEnd::new("data")))
                .context("Failed to write data end")?;
            writer
                .write_event(Event::End(BytesEnd::new("array")))
                .context("Failed to write array end")?;
        }
        XmlRpcValue::Struct(members) => {
            writer
                .write_event(Event::Start(BytesStart::new("struct")))
                .context("Failed to write struct start")?;
            for (name, val) in members {
                writer
                    .write_event(Event::Start(BytesStart::new("member")))
                    .context("Failed to write member start")?;
                writer
                    .write_event(Event::Start(BytesStart::new("name")))
                    .context("Failed to write name start")?;
                writer
                    .write_event(Event::Text(BytesText::new(name)))
                    .context("Failed to write name value")?;
                writer
                    .write_event(Event::End(BytesEnd::new("name")))
                    .context("Failed to write name end")?;
                write_value(writer, val)?;
                writer
                    .write_event(Event::End(BytesEnd::new("member")))
                    .context("Failed to write member end")?;
            }
            writer
                .write_event(Event::End(BytesEnd::new("struct")))
                .context("Failed to write struct end")?;
        }
    }

    writer
        .write_event(Event::End(BytesEnd::new("value")))
        .context("Failed to write value end")?;

    Ok(())
}

/// Parse XML-RPC response
pub fn parse_response(xml: &str) -> Result<XmlRpcResponse> {
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut in_fault = false;
    let mut in_params = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"fault" => in_fault = true,
                b"params" => in_params = true,
                b"value" if in_fault || in_params => {
                    let value = parse_value_content(&mut reader)?;
                    if in_fault {
                        return Ok(XmlRpcResponse::Fault(value));
                    } else {
                        return Ok(XmlRpcResponse::Success(value));
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("XML parsing error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Err(anyhow::anyhow!("Invalid XML-RPC response"))
}

fn parse_value_content(reader: &mut quick_xml::Reader<&[u8]>) -> Result<XmlRpcValue> {
    let mut buf = Vec::new();
    let mut depth = 1;
    let mut current_type: Option<String> = None;
    let mut text_content = String::new();
    let mut array_items: Vec<XmlRpcValue> = Vec::new();
    let mut struct_members: Vec<(String, XmlRpcValue)> = Vec::new();
    let mut member_name = String::new();
    let mut in_name = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "value" => {
                        depth += 1;
                        if depth > 1 && current_type.as_deref() == Some("array") {
                            // Nested value in array
                            let nested = parse_value_content(reader)?;
                            array_items.push(nested);
                            depth -= 1;
                        } else if depth > 1 && current_type.as_deref() == Some("struct") {
                            // Nested value in struct
                            let nested = parse_value_content(reader)?;
                            struct_members.push((member_name.clone(), nested));
                            member_name.clear();
                            depth -= 1;
                        }
                    }
                    "name" => in_name = true,
                    "string" | "int" | "i4" | "boolean" | "double" | "array" | "struct"
                    | "data" | "member" => {
                        if current_type.is_none() {
                            current_type = Some(tag);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "value" => {
                        depth -= 1;
                        if depth == 0 {
                            // Return based on type
                            return match current_type.as_deref() {
                                Some("string") | None => Ok(XmlRpcValue::String(text_content)),
                                Some("int") | Some("i4") => {
                                    let i: i32 = text_content.parse().unwrap_or(0);
                                    Ok(XmlRpcValue::Int(i))
                                }
                                Some("boolean") => {
                                    let b = text_content == "1"
                                        || text_content.to_lowercase() == "true";
                                    Ok(XmlRpcValue::Boolean(b))
                                }
                                Some("double") => {
                                    let d: f64 = text_content.parse().unwrap_or(0.0);
                                    Ok(XmlRpcValue::Double(d))
                                }
                                Some("array") => Ok(XmlRpcValue::Array(array_items)),
                                Some("struct") => Ok(XmlRpcValue::Struct(struct_members)),
                                _ => Ok(XmlRpcValue::String(text_content)),
                            };
                        }
                    }
                    "name" => in_name = false,
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.decode().unwrap_or_default().to_string();
                if in_name {
                    member_name = text;
                } else {
                    text_content.push_str(&text);
                }
            }
            Ok(Event::GeneralRef(e)) => {
                let name = e.decode().unwrap_or_default();
                let resolved = resolve_entity(&name);
                if in_name {
                    member_name.push_str(resolved);
                } else {
                    text_content.push_str(resolved);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("XML parsing error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(XmlRpcValue::String(text_content))
}

/// XML-RPC response types
#[derive(Debug)]
pub enum XmlRpcResponse {
    Success(XmlRpcValue),
    Fault(XmlRpcValue),
}

/// Convert XmlRpcValue to serde_json::Value
pub fn xmlrpc_to_json(value: &XmlRpcValue) -> Value {
    match value {
        XmlRpcValue::String(s) => Value::String(s.clone()),
        XmlRpcValue::Int(i) => Value::Number((*i).into()),
        XmlRpcValue::Boolean(b) => Value::Bool(*b),
        XmlRpcValue::Double(d) => serde_json::Number::from_f64(*d)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        XmlRpcValue::Array(arr) => Value::Array(arr.iter().map(xmlrpc_to_json).collect()),
        XmlRpcValue::Struct(members) => {
            let map: Map<String, Value> = members
                .iter()
                .map(|(k, v)| (k.clone(), xmlrpc_to_json(v)))
                .collect();
            Value::Object(map)
        }
    }
}

/// Maximum XML nesting depth to prevent stack overflow from malicious input
const MAX_XML_DEPTH: usize = 128;

/// Parse OpenNebula's XML data format to JSON
/// OpenNebula returns XML data as a string in XML-RPC responses
pub fn parse_one_xml_to_json(xml: &str) -> Result<Value> {
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    parse_xml_element(&mut reader, 0)
}

fn parse_xml_element(reader: &mut quick_xml::Reader<&[u8]>, depth: usize) -> Result<Value> {
    if depth > MAX_XML_DEPTH {
        return Err(anyhow::anyhow!(
            "XML nesting depth exceeds maximum ({})",
            MAX_XML_DEPTH
        ));
    }

    let mut buf = Vec::new();
    let mut result: Map<String, Value> = Map::new();
    let mut text_content = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                // Parse nested element recursively
                let nested = parse_xml_element(reader, depth + 1)?;

                // Handle arrays (multiple elements with same name)
                if let Some(existing) = result.get_mut(&tag) {
                    match existing {
                        Value::Array(arr) => arr.push(nested),
                        _ => {
                            let old = existing.clone();
                            *existing = Value::Array(vec![old, nested]);
                        }
                    }
                } else {
                    result.insert(tag, nested);
                }
            }
            Ok(Event::End(_)) => {
                if result.is_empty() && !text_content.is_empty() {
                    // Leaf element with text content
                    return Ok(Value::String(text_content.trim().to_string()));
                }
                return Ok(Value::Object(result));
            }
            Ok(Event::Text(e)) => {
                text_content.push_str(&e.decode().unwrap_or_default());
            }
            Ok(Event::GeneralRef(e)) => {
                text_content.push_str(resolve_entity(&e.decode().unwrap_or_default()));
            }
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                result.insert(tag, Value::Null);
            }
            Ok(Event::Eof) => {
                if result.is_empty() && !text_content.is_empty() {
                    return Ok(Value::String(text_content.trim().to_string()));
                }
                return Ok(Value::Object(result));
            }
            Err(e) => return Err(anyhow::anyhow!("XML parsing error: {}", e)),
            _ => {}
        }
        buf.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_method_call() {
        let params = vec![
            XmlRpcValue::String("user:password".to_string()),
            XmlRpcValue::Int(-2),
            XmlRpcValue::Int(-1),
            XmlRpcValue::Int(-1),
        ];
        let xml = build_method_call("one.vmpool.info", &params).unwrap();
        assert!(xml.contains("one.vmpool.info"));
        assert!(xml.contains("user:password"));
    }

    #[test]
    fn test_parse_one_xml() {
        let xml = r#"<VM><ID>123</ID><NAME>test-vm</NAME></VM>"#;
        let json = parse_one_xml_to_json(xml).unwrap();
        assert_eq!(json["VM"]["ID"], "123");
        assert_eq!(json["VM"]["NAME"], "test-vm");
    }

    #[test]
    fn test_resolve_entity_all() {
        assert_eq!(resolve_entity("lt"), "<");
        assert_eq!(resolve_entity("gt"), ">");
        assert_eq!(resolve_entity("amp"), "&");
        assert_eq!(resolve_entity("apos"), "'");
        assert_eq!(resolve_entity("quot"), "\"");
        assert_eq!(resolve_entity("unknown"), "");
    }

    #[test]
    fn test_xmlrpc_to_json_all_types() {
        assert_eq!(
            xmlrpc_to_json(&XmlRpcValue::String("hello".into())),
            serde_json::json!("hello")
        );
        assert_eq!(
            xmlrpc_to_json(&XmlRpcValue::Int(42)),
            serde_json::json!(42)
        );
        assert_eq!(
            xmlrpc_to_json(&XmlRpcValue::Boolean(true)),
            serde_json::json!(true)
        );
        assert_eq!(
            xmlrpc_to_json(&XmlRpcValue::Double(3.14)),
            serde_json::json!(3.14)
        );
        assert_eq!(
            xmlrpc_to_json(&XmlRpcValue::Array(vec![
                XmlRpcValue::Int(1),
                XmlRpcValue::String("two".into()),
            ])),
            serde_json::json!([1, "two"])
        );
        assert_eq!(
            xmlrpc_to_json(&XmlRpcValue::Struct(vec![
                ("key".into(), XmlRpcValue::String("val".into())),
                ("num".into(), XmlRpcValue::Int(5)),
            ])),
            serde_json::json!({"key": "val", "num": 5})
        );
    }

    #[test]
    fn test_xmlrpc_to_json_double_nan() {
        // NaN can't be represented in JSON
        let result = xmlrpc_to_json(&XmlRpcValue::Double(f64::NAN));
        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn test_build_method_call_with_boolean_and_double() {
        let params = vec![
            XmlRpcValue::Boolean(true),
            XmlRpcValue::Double(1.5),
        ];
        let xml = build_method_call("test.method", &params).unwrap();
        assert!(xml.contains("test.method"));
        assert!(xml.contains("<boolean>1</boolean>"));
        assert!(xml.contains("<double>1.5</double>"));
    }

    #[test]
    fn test_build_method_call_with_array() {
        let params = vec![XmlRpcValue::Array(vec![
            XmlRpcValue::Int(1),
            XmlRpcValue::Int(2),
        ])];
        let xml = build_method_call("test.array", &params).unwrap();
        assert!(xml.contains("<array>"));
        assert!(xml.contains("<data>"));
        assert!(xml.contains("<int>1</int>"));
        assert!(xml.contains("<int>2</int>"));
    }

    #[test]
    fn test_build_method_call_with_struct() {
        let params = vec![XmlRpcValue::Struct(vec![(
            "name".into(),
            XmlRpcValue::String("value".into()),
        )])];
        let xml = build_method_call("test.struct", &params).unwrap();
        assert!(xml.contains("<struct>"));
        assert!(xml.contains("<member>"));
        assert!(xml.contains("<name>name</name>"));
        assert!(xml.contains("<string>value</string>"));
    }

    #[test]
    fn test_parse_response_fault() {
        let xml = r#"<?xml version="1.0"?>
<methodResponse><fault><value><struct>
  <member><name>faultCode</name><value><int>4</int></value></member>
  <member><name>faultString</name><value><string>Too many parameters.</string></value></member>
</struct></value></fault></methodResponse>"#;
        let result = parse_response(xml).unwrap();
        assert!(matches!(result, XmlRpcResponse::Fault(_)));
    }

    #[test]
    fn test_parse_response_invalid_xml() {
        let result = parse_response("not xml at all {{{");
        // Should return an error (no valid methodResponse found)
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_one_xml_nested_arrays() {
        // Duplicate tags should produce an array
        let xml = r#"<VM_POOL><VM><ID>1</ID></VM><VM><ID>2</ID></VM></VM_POOL>"#;
        let json = parse_one_xml_to_json(xml).unwrap();
        let vms = json["VM_POOL"]["VM"].as_array().expect("Should be array");
        assert_eq!(vms.len(), 2);
        assert_eq!(vms[0]["ID"], "1");
        assert_eq!(vms[1]["ID"], "2");
    }

    #[test]
    fn test_parse_one_xml_empty_element() {
        let xml = r#"<HOST><ID>1</ID><TEMPLATE/></HOST>"#;
        let json = parse_one_xml_to_json(xml).unwrap();
        assert_eq!(json["HOST"]["ID"], "1");
        assert!(json["HOST"]["TEMPLATE"].is_null());
    }

    #[test]
    fn test_parse_response_success_with_int() {
        let xml = r#"<?xml version="1.0"?>
<methodResponse><params><param><value><array><data>
  <value><boolean>1</boolean></value>
  <value><int>42</int></value>
  <value><int>0</int></value>
</data></array></value></param></params></methodResponse>"#;
        let result = parse_response(xml).unwrap();
        match result {
            XmlRpcResponse::Success(XmlRpcValue::Array(arr)) => {
                assert_eq!(arr.len(), 3);
                assert!(matches!(arr[0], XmlRpcValue::Boolean(true)));
                assert!(matches!(arr[1], XmlRpcValue::Int(42)));
            }
            _ => panic!("Expected Success with Array"),
        }
    }
}
