use anyhow::Result;
use serde_json::Value;
use std::path::Path;
use std::fs;

use crate::providers::ToolDefinition;
use super::safe_resolve_path;

pub fn definitions() -> Vec<ToolDefinition> {
    vec![ToolDefinition {
        name: "AstAnalysis".to_string(),
        description: "Analyze code structure using AST (Abstract Syntax Tree). Use 'analyze_file' to get a structural summary of a file (functions, classes). Use 'get_call_graph' with a specific symbol to find callers and callees in the current file.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "Action to perform: 'analyze_file' or 'get_call_graph'",
                    "enum": ["analyze_file", "get_call_graph"]
                },
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to analyze (e.g., src/main.rs)"
                },
                "symbol": {
                    "type": "string",
                    "description": "Target symbol (e.g., function or class name) required for get_call_graph"
                }
            },
            "required": ["action", "file_path"]
        }),
    }]
}

pub async fn ast_analysis(project_root: &Path, args: &Value) -> Result<String> {
    let action = args["action"].as_str().unwrap_or("").to_string();
    let file_path = args["file_path"].as_str().unwrap_or("");
    let _symbol = args["symbol"].as_str().map(|s| s.to_string());

    if action.is_empty() || file_path.is_empty() {
        return Ok("Error: action and file_path are required.".to_string());
    }

    let absolute_path = match safe_resolve_path(project_root, file_path) {
        Ok(path) => path,
        Err(_) => return Ok(format!("Error: Invalid file path '{}'", file_path)),
    };

    if !absolute_path.exists() {
        return Ok(format!("Error: File not found: {}", file_path));
    }

    let source_code = match fs::read_to_string(&absolute_path) {
        Ok(c) => c,
        Err(e) => return Ok(format!("Error: Failed to read file: {}", e)),
    };

    let extension = absolute_path.extension().and_then(|s| s.to_str()).unwrap_or("");

    let mut parser = tree_sitter::Parser::new();
    let language = match extension {
        "rs" => tree_sitter_rust::LANGUAGE.into(),
        "py" => tree_sitter_python::LANGUAGE.into(),
        _ => return Ok(format!("Error: Unsupported file type '.{}'. AstAnalysis currently supports .rs and .py", extension)),
    };

    if let Err(e) = parser.set_language(&language) {
        return Ok(format!("Error initializing parser language: {}", e));
    }

    let tree = match parser.parse(&source_code, None) {
        Some(t) => t,
        None => return Ok("Error: Failed to parse file into AST".to_string()),
    };

    // For now, we will just implement a simple structure dump for 'analyze_file'
    // by manually traversing the tree, before writing full Tree-sitter Queries.
    
    if action == "analyze_file" {
        return analyze_file_structure(&tree, source_code.as_bytes(), extension);
    } else if action == "get_call_graph" {
        return Ok("The 'get_call_graph' action is still under development in Phase 1.".to_string());
    }

    Ok(format!("Error: Unknown action '{}'", action))
}

fn analyze_file_structure(tree: &tree_sitter::Tree, source: &[u8], extension: &str) -> Result<String> {
    let mut output = String::new();
    output.push_str("### AST Structure Summary\n\n");

    let root_node = tree.root_node();
    let mut cursor = root_node.walk();

    let (func_type, class_type, name_field) = match extension {
        "rs" => ("function_item", "struct_item", "name"),
        "py" => ("function_definition", "class_definition", "name"),
        _ => ("","",""),
    };

    let mut found_functions = Vec::new();
    let mut found_classes = Vec::new();

    // A very simple recursive traversal
    fn traverse(
        cursor: &mut tree_sitter::TreeCursor, 
        source: &[u8], 
        func_type: &str, 
        class_type: &str, 
        name_field: &str,
        funcs: &mut Vec<String>,
        classes: &mut Vec<String>
    ) {
        let node = cursor.node();
        let kind = node.kind();

        if kind == func_type {
            if let Some(name_node) = node.child_by_field_name(name_field) {
                if let Ok(name) = std::str::from_utf8(&source[name_node.byte_range()]) {
                    funcs.push(format!("- `{}` (Line {})", name, name_node.start_position().row + 1));
                }
            }
        } else if kind == class_type {
            if let Some(name_node) = node.child_by_field_name(name_field) {
                if let Ok(name) = std::str::from_utf8(&source[name_node.byte_range()]) {
                    classes.push(format!("- `{}` (Line {})", name, name_node.start_position().row + 1));
                }
            }
        }

        if cursor.goto_first_child() {
            traverse(cursor, source, func_type, class_type, name_field, funcs, classes);
            while cursor.goto_next_sibling() {
                traverse(cursor, source, func_type, class_type, name_field, funcs, classes);
            }
            cursor.goto_parent();
        }
    }

    traverse(&mut cursor, source, func_type, class_type, name_field, &mut found_functions, &mut found_classes);

    if !found_classes.is_empty() {
        output.push_str("**Classes / Structs:**\n");
        for c in found_classes {
            output.push_str(&format!("{}\n", c));
        }
        output.push('\n');
    }

    if !found_functions.is_empty() {
        output.push_str("**Functions:**\n");
        for f in found_functions {
            output.push_str(&format!("{}\n", f));
        }
    }

    if output.len() < 50 {
        output.push_str("No major structures found.");
    }

    Ok(output)
}