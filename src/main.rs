use rusqlite::{Connection, types::ValueRef};
use std::error::Error;
use std::io::{self, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let db_path = "hollow.db";
    let db_already_exists = Path::new(db_path).exists();
    let conn = Connection::open(db_path)?;

    if !db_already_exists {
        initialize_schema(&conn)?;
        println!("Initialized new database at {db_path}.");
    } else {
        println!("Using existing database at {db_path}.");
    }

    print_help();

    let stdin = io::stdin();
    loop {
        print!("hollow-rust> ");
        io::stdout().flush()?;

        let mut line = String::new();
        stdin.read_line(&mut line)?;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if matches!(trimmed.to_ascii_lowercase().as_str(), "exit" | "quit") {
            println!("Goodbye.");
            break;
        }

        if trimmed.eq_ignore_ascii_case("help") {
            print_help();
            continue;
        }

        if let Err(err) = handle_command(&conn, trimmed) {
            eprintln!("Error: {err}");
        }
    }

    Ok(())
}

fn initialize_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS charms_locations (
            id INTEGER PRIMARY KEY,
            location_name TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS charms_requirements (
            id INTEGER PRIMARY KEY,
            condition_text TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS charms_vendor (
            id INTEGER PRIMARY KEY,
            vendor_name TEXT NOT NULL,
            cost INTEGER
        );

        CREATE TABLE IF NOT EXISTS mask_shards_locations (
            id INTEGER PRIMARY KEY,
            location_name TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS mask_shards_requirements (
            id INTEGER PRIMARY KEY,
            condition_text TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS mask_shards_vendor (
            id INTEGER PRIMARY KEY,
            vendor_name TEXT NOT NULL,
            cost INTEGER
        );

        CREATE TABLE IF NOT EXISTS vendor_locations (
            vendor_name TEXT PRIMARY KEY,
            location_name TEXT NOT NULL
        );
        "#,
    )?;

    Ok(())
}

fn handle_command(conn: &Connection, input: &str) -> Result<(), Box<dyn Error>> {
    let mut parts = input.splitn(2, char::is_whitespace);
    let command = parts.next().unwrap_or_default().to_ascii_lowercase();
    let rest = parts.next().unwrap_or("").trim();

    match command.as_str() {
        "create" => handle_create(conn, rest)?,
        "read" => handle_read(conn, rest)?,
        "update" => handle_update(conn, rest)?,
        "delete" => handle_delete(conn, rest)?,
        "sql" => handle_sql(conn, rest)?,
        _ => {
            println!("Unknown command. Type 'help' for usage.");
        }
    }

    Ok(())
}

fn handle_create(conn: &Connection, args: &str) -> Result<(), Box<dyn Error>> {
    let mut tokens = args.split_whitespace();
    let table = tokens.next().ok_or("Usage: create <table> <column=value> ...")?;
    validate_identifier(table)?;

    let assignments: Vec<&str> = tokens.collect();
    if assignments.is_empty() {
        return Err("Usage: create <table> <column=value> ...".into());
    }

    let (columns, values) = parse_assignments(&assignments)?;
    let sql = format!(
        "INSERT INTO {table} ({}) VALUES ({})",
        columns.join(", "),
        values.join(", ")
    );

    let changed = conn.execute(&sql, [])?;
    println!("Inserted {changed} row(s).");
    Ok(())
}

fn handle_read(conn: &Connection, args: &str) -> Result<(), Box<dyn Error>> {
    let mut parts = args.splitn(2, char::is_whitespace);
    let table = parts.next().ok_or("Usage: read <table> [where <condition>]")?;
    validate_identifier(table)?;

    let remainder = parts.next().unwrap_or("").trim();
    let condition = strip_prefix_case_insensitive(remainder, "where ").unwrap_or("");
    let sql = if condition.is_empty() {
        format!("SELECT * FROM {table}")
    } else {
        format!("SELECT * FROM {table} WHERE {condition}")
    };

    run_select_query(conn, &sql)?;
    Ok(())
}

fn handle_update(conn: &Connection, args: &str) -> Result<(), Box<dyn Error>> {
    let mut parts = args.splitn(2, char::is_whitespace);
    let table = parts
        .next()
        .ok_or("Usage: update <table> set <column=value> [column=value ...] [where <condition>]")?;
    validate_identifier(table)?;

    let remainder = parts.next().unwrap_or("").trim();
    let set_block = strip_prefix_case_insensitive(remainder, "set ").ok_or(
        "Usage: update <table> set <column=value> [column=value ...] [where <condition>]",
    )?;

    let lower_set = set_block.to_ascii_lowercase();
    let (assignment_text, where_condition) = if let Some(idx) = lower_set.find(" where ") {
        (&set_block[..idx], Some(set_block[idx + 7..].trim()))
    } else {
        (set_block, None)
    };

    let assignment_tokens: Vec<&str> = if assignment_text.contains(',') {
        assignment_text
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        assignment_text.split_whitespace().collect()
    };

    if assignment_tokens.is_empty() {
        return Err("No assignments provided for update.".into());
    }

    let mut set_parts = Vec::new();
    for token in assignment_tokens {
        let (column, value) = token
            .split_once('=')
            .ok_or("Assignments must use column=value format.")?;
        validate_identifier(column)?;
        set_parts.push(format!("{column}={}", to_sql_literal(value)));
    }

    let sql = if let Some(condition) = where_condition {
        format!("UPDATE {table} SET {} WHERE {condition}", set_parts.join(", "))
    } else {
        format!("UPDATE {table} SET {}", set_parts.join(", "))
    };

    let changed = conn.execute(&sql, [])?;
    println!("Updated {changed} row(s).");
    Ok(())
}

fn handle_delete(conn: &Connection, args: &str) -> Result<(), Box<dyn Error>> {
    let mut parts = args.splitn(2, char::is_whitespace);
    let table = parts.next().ok_or("Usage: delete <table> [where <condition>]")?;
    validate_identifier(table)?;

    let remainder = parts.next().unwrap_or("").trim();
    let condition = strip_prefix_case_insensitive(remainder, "where ");
    let sql = if let Some(where_clause) = condition {
        if where_clause.is_empty() {
            return Err("WHERE condition cannot be empty.".into());
        }
        format!("DELETE FROM {table} WHERE {where_clause}")
    } else {
        format!("DELETE FROM {table}")
    };

    let changed = conn.execute(&sql, [])?;
    println!("Deleted {changed} row(s).");
    Ok(())
}

fn handle_sql(conn: &Connection, query: &str) -> Result<(), Box<dyn Error>> {
    if query.is_empty() {
        return Err("Usage: sql <query>".into());
    }

    if is_select_query(query) {
        run_select_query(conn, query)?;
    } else {
        conn.execute_batch(query)?;
        println!("SQL executed successfully.");
    }

    Ok(())
}

fn run_select_query(conn: &Connection, query: &str) -> Result<(), Box<dyn Error>> {
    let mut stmt = conn.prepare(query)?;
    let column_names: Vec<String> = stmt
        .column_names()
        .into_iter()
        .map(ToString::to_string)
        .collect();

    println!("{}", column_names.join(" | "));

    let mut rows = stmt.query([])?;
    let mut count = 0usize;
    while let Some(row) = rows.next()? {
        let mut rendered = Vec::with_capacity(column_names.len());
        for index in 0..column_names.len() {
            let value = row.get_ref(index)?;
            rendered.push(value_ref_to_string(value));
        }
        println!("{}", rendered.join(" | "));
        count += 1;
    }

    println!("{count} row(s).");
    Ok(())
}

fn parse_assignments(tokens: &[&str]) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    let mut columns = Vec::new();
    let mut values = Vec::new();

    for token in tokens {
        let (column, value) = token
            .split_once('=')
            .ok_or("Assignments must use column=value format.")?;
        validate_identifier(column)?;
        columns.push(column.to_string());
        values.push(to_sql_literal(value));
    }

    Ok((columns, values))
}

fn validate_identifier(identifier: &str) -> Result<(), Box<dyn Error>> {
    if identifier.is_empty()
        || !identifier
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(format!("Invalid identifier: {identifier}").into());
    }
    Ok(())
}

fn strip_prefix_case_insensitive<'a>(input: &'a str, prefix: &str) -> Option<&'a str> {
    if input.len() < prefix.len() {
        return None;
    }

    let (head, tail) = input.split_at(prefix.len());
    if head.eq_ignore_ascii_case(prefix) {
        Some(tail.trim())
    } else {
        None
    }
}

fn to_sql_literal(value: &str) -> String {
    if value.eq_ignore_ascii_case("null") {
        "NULL".to_string()
    } else if value.parse::<f64>().is_ok() {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "''"))
    }
}

fn value_ref_to_string(value: ValueRef<'_>) -> String {
    match value {
        ValueRef::Null => "NULL".to_string(),
        ValueRef::Integer(v) => v.to_string(),
        ValueRef::Real(v) => v.to_string(),
        ValueRef::Text(v) => String::from_utf8_lossy(v).to_string(),
        ValueRef::Blob(v) => format!("<{} bytes>", v.len()),
    }
}

fn is_select_query(query: &str) -> bool {
    let trimmed = query.trim_start().to_ascii_lowercase();
    trimmed.starts_with("select")
        || trimmed.starts_with("with")
        || trimmed.starts_with("pragma")
        || trimmed.starts_with("explain")
}

fn print_help() {
    println!(
        r#"Commands:
  create <table> <column=value> ...                     Insert a row
  read <table> [where <condition>]                      Query rows
  update <table> set <column=value> ... [where <cond>]  Update rows
  delete <table> [where <condition>]                    Delete rows
  sql <query>                                            Run raw SQL
  help                                                   Show commands
  exit | quit                                            Leave program"#
    );
}
