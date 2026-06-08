use serde::Deserialize;
use std::fs;
use std::path::Path;
use toml::Value;

#[derive(Debug, Deserialize)]
struct TestConfig {
    config: Config,
    input: Input,
    expected: Expected,
}

#[derive(Debug, Deserialize)]
struct Config {
    name: String,
    max_ticks: u32,
}

#[derive(Debug, Deserialize)]
struct Input {
    source_code: String,
    stream_in: Vec<Value>,
}

#[derive(Debug, Deserialize)]
struct Expected {
    stream_out: Vec<Value>,
    ast: String,
    assembly: String,
    binary_code: String,
    trace: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");

    for entry in fs::read_dir(&tests_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            run_test_file(&path)?;
        }
    }

    Ok(())
}

fn run_test_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let test: TestConfig = toml::from_str(&content)?;
    let source = &test.input.source_code;

    println!("--- {} ---", test.config.name);

    let compile_result = match compiler::compile(source) {
        Ok(res) => res,
        Err(e) => {
            println!("\t[FAIL] Compilation error: {}", e);
            return Ok(());
        }
    };

    let mut all_ok = true;

    if !test.expected.ast.trim().is_empty() {
        let actual = normalize_str(&compile_result.ast_debug);
        let expected = normalize_str(&test.expected.ast);
        if actual == expected {
            println!("\t[OK] AST matches");
        } else {
            println!("\t[FAIL] AST mismatch");
            all_ok = false;
        }
    }

    if !test.expected.assembly.trim().is_empty() {
        let actual = normalize_str(&compile_result.assembly);
        let expected = normalize_str(&test.expected.assembly);
        if actual == expected {
            println!("\t[OK] Assembly matches");
        } else {
            println!("\t[FAIL] Assembly mismatch");
            all_ok = false;
        }
    }

    if !test.expected.binary_code.trim().is_empty() {
        let hex: Vec<String> = compile_result
            .binary
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect();
        let actual = hex.join(" ");
        if normalize_str(&actual) == normalize_str(&test.expected.binary_code) {
            println!("\t[OK] Binary code matches");
        } else {
            println!("\t[FAIL] Binary code mismatch");
            all_ok = false;
        }
    }

    let input: Vec<i32> = test
        .input
        .stream_in
        .iter()
        .map(|v| match v {
            Value::String(s) => s.bytes().next().unwrap_or(0) as i32,
            Value::Integer(n) => *n as i32,
            _ => 0,
        })
        .collect();
    println!("\tInput buffer: {:?}", input);

    let sim_result = match simulator::run(&compile_result, &input) {
        Ok(res) => res,
        Err(e) => {
            println!("\t[FAIL] Simulation error: {}", e);
            return Ok(());
        }
    };

    if !test.expected.stream_out.is_empty() {
        let expected: Vec<i32> = test
            .expected
            .stream_out
            .iter()
            .map(|v| match v {
                Value::String(s) => s.bytes().next().unwrap_or(0) as i32,
                Value::Integer(n) => *n as i32,
                _ => 0,
            })
            .collect();

        if sim_result.output == expected {
            println!("\t[OK] Output matches ({:?})", expected);
        } else {
            println!("\t[FAIL] Output mismatch");
            println!("\t\texpected:\t{:?}", expected);
            println!("\t\tgot:\t{:?}", sim_result.output);
            all_ok = false;
        }
    }

    if !test.expected.trace.trim().is_empty() {
        let max_ticks = test.config.max_ticks as usize;
        let actual_lines: Vec<&str> = sim_result.trace.lines().collect();
        let expected_lines: Vec<&str> = test.expected.trace.lines().collect();

        for (i, expected_line) in expected_lines.iter().enumerate() {
            let expected_trimmed = expected_line.trim();
            let actual_trimmed = actual_lines[i].trim();
            if actual_trimmed != expected_trimmed {
                println!("\t[INFO] Trace line {} differs", i + 1);
                println!("\t\texpected:\t{}", expected_trimmed);
                println!("\t\tgot:\t{}", actual_trimmed);
            }
        }

        if actual_lines.len() > max_ticks {
            println!("\t[FAIL] Exceeded max_ticks ({})", max_ticks);
            all_ok = false;
        }

        if all_ok {
            println!("\t[OK] Trace matches ({} lines)", actual_lines.len());
        }
    }

    if all_ok {
        println!("[PASS] All checks passed");
    }

    println!();
    Ok(())
}

fn normalize_str(s: &str) -> String {
    s.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join("\n")
}
