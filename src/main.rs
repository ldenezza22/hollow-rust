use rusqlite::{params, Connection, types::ValueRef};
use std::error::Error;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let db_path = "hollow.db"; 
    if Path::new(db_path).exists() {
        fs::remove_file(db_path)?;
        println!("  Removed existing database file at {db_path}.");
    }

    // Open database connection
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    // Initialize the database
    println!("Step 1: Initialize the database");
    initialize_schema(&conn)?;
    seed_demo_rows(&conn)?;
    println!("  Created schema and seed rows at {db_path}.");
    print_seeded_tables(&conn)?;

    // Change charms vendor cost to 230 geo
    println!("Step 2: Change charms vendor cost to 230 geo");
    println!("  Before update (charms_vendor where id = 5):");
    run_select_query(
        &conn,
        "SELECT id, vendor_name, cost FROM charms_vendor WHERE id = 5",
    )?;
    let updated = conn.execute(
        "UPDATE charms_vendor SET cost = ?1 WHERE vendor_name = ?2 AND cost = ?3",
        params![230, "Salubra", 220],
    )?;
    println!(
        "  Updated {updated} row(s): Salubra Shaman Stone offer adjusted from 220 to 230 geo (see wiki)."
    );
    println!("  After update (charms_vendor where id = 5):");
    run_select_query(
        &conn,
        "SELECT id, vendor_name, cost FROM charms_vendor WHERE id = 5",
    )?;

    // Join read (charms vendor with vendor location)
    println!("Step 3: Join read (charms vendor with vendor location)");
    let join_sql = r#"
        SELECT cv.id,
               cv.vendor_name,
               cv.cost,
               vl.location_name AS shop_location
        FROM charms_vendor AS cv
        INNER JOIN vendor_locations AS vl
            ON cv.vendor_name = vl.vendor_name
        ORDER BY cv.id
    "#;
    run_select_query(&conn, join_sql)?;


    // Delete from the database
    println!("Step 4: Delete all charms sold by Leg Eater");
    println!("  Before delete (charms_vendor where vendor_name = 'Leg Eater'):");
    run_select_query(
        &conn,
        "SELECT id, vendor_name, cost FROM charms_vendor ORDER BY id",
    )?;
    let deleted = conn.execute(
        "DELETE FROM charms_vendor WHERE vendor_name = ?1",
        params!["Leg Eater"],
    )?;
    println!("  Deleted {deleted} row(s) from charms_vendor for Leg Eater.");
    println!("  After delete (charms_vendor where vendor_name = 'Leg Eater'):");
    run_select_query(
        &conn,
        "SELECT id, vendor_name, cost FROM charms_vendor ORDER BY id",
    )?;

    Ok(())
}

// This is the scema of the database from README.md
fn initialize_schema(conn: &Connection) -> rusqlite::Result<()> {

    // Create all tables with CREATE TABLE IF NOT EXISTS
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

/// Seed data transcribed from the Hollow Knight Wiki (Fandom):
/// - Charm vendors & prices: https://hollowknight.fandom.com/wiki/Salubra
/// - Charm sources: https://hollowknight.fandom.com/wiki/Category:Charms (List of Charms)
/// - Mask shard acquisition: https://hollowknight.fandom.com/wiki/Mask_Shard
/// - Leg Eater fragile charm prices: https://hollowknight.fandom.com/wiki/Leg_Eater
fn seed_demo_rows(conn: &Connection) -> rusqlite::Result<()> {
    // Seed vendor_locations (wiki shop NPC → area)
    for (vendor, area) in [
        ("Salubra", "Forgotten Crossroads"),
        ("Sly", "Dirtmouth"),
        ("Iselda", "Dirtmouth"),
        ("Divine", "Dirtmouth"),
        ("Leg Eater", "Fungal Wastes"),
        ("Grubfather", "Forgotten Crossroads"),
        ("Seer", "Resting Grounds"),
        ("Grey Mourner", "Resting Grounds"),
    ] {
        conn.execute(
            "INSERT INTO vendor_locations (vendor_name, location_name) VALUES (?1, ?2)",
            params![vendor, area],
        )?;
    }

    // charms_vendor: vendor-obtained charms keyed to List of Charms ids (1..45)
    for (id, vendor, cost) in [
        (1, "Iselda", 220),    // Wayward Compass
        (2, "Sly", 300),       // Gathering Swarm
        (3, "Sly", 200),       // Stalwart Shell
        (5, "Salubra", 220),   // Shaman Stone
        (8, "Sly", 400),       // Sprintmaster
        (11, "Leg Eater", 350), // Fragile Heart
        (12, "Divine", 12000), // Unbreakable Heart
        (13, "Leg Eater", 250), // Fragile Greed
        (14, "Divine", 9000),  // Unbreakable Greed
        (15, "Leg Eater", 600), // Fragile Strength
        (16, "Divine", 15000), // Unbreakable Strength
        (18, "Salubra", 120),  // Steady Body
        (19, "Sly", 350),      // Heavy Blow
        (21, "Salubra", 300),  // Longnail
        (29, "Salubra", 800),  // Quick Focus
        (31, "Salubra", 250),  // Lifeblood Heart
    ] {
        conn.execute(
            "INSERT INTO charms_vendor (id, vendor_name, cost) VALUES (?1, ?2, ?3)",
            params![id, vendor, cost],
        )?;
    }

    // charms_locations / requirements: List of Charms entries
    // Source: https://hollowknight.fandom.com/wiki/Category:Charms (List of Charms)
    let charm_locs = [
        (1, "Dirtmouth"),
        (2, "Dirtmouth"),
        (3, "Dirtmouth"),
        (4, "Forgotten Crossroads"),
        (5, "Forgotten Crossroads"),
        (6, "Resting Grounds"),
        (7, "Fungal Wastes"),
        (8, "Dirtmouth"),
        (9, "Forgotten Crossroads"),
        (10, "Forgotten Crossroads"),
        (11, "Fungal Wastes"),
        (12, "Dirtmouth"),
        (13, "Fungal Wastes"),
        (14, "Dirtmouth"),
        (15, "Fungal Wastes"),
        (16, "Dirtmouth"),
        (17, "City of Tears"),
        (18, "Forgotten Crossroads"),
        (19, "Crystal Peak"),
        (20, "Kingdom's Edge"),
        (21, "Forgotten Crossroads"),
        (22, "Fungal Wastes"),
        (23, "King's Pass"),
        (24, "Greenpath"),
        (25, "Howling Cliffs"),
        (26, "Royal Waterways"),
        (27, "Royal Waterways"),
        (28, "Forgotten Crossroads"),
        (29, "Forgotten Crossroads"),
        (30, "Crystal Peak"),
        (31, "Forgotten Crossroads"),
        (32, "Abyss"),
        (33, "Howling Cliffs"),
        (34, "Hive"),
        (35, "Fungal Wastes"),
        (36, "Deepnest"),
        (37, "Greenpath"),
        (38, "Dirtmouth"),
        (39, "Deepnest"),
        (40, "Resting Grounds"),
        (41, "Resting Grounds"),
        (42, "Dirtmouth"),
        (43, "Dirtmouth"),
        (44, "Queen's Gardens"),
        (45, "Abyss")
    ];

    // Insert all charm locations with INSERT INTO
    for (id, loc) in charm_locs {
        conn.execute(
            "INSERT INTO charms_locations (id, location_name) VALUES (?1, ?2)",
            params![id, loc],
        )?;
    }
    let charm_req: [(i32, Option<&str>); 24] = [
        (1, Some("After encountering Cornifer.")),
        (6, Some("Requires Desolate Dive.")),
        (8, Some("Requires Shopkeeper's Key.")),
        (9, Some("Free 10 Grubs.")),
        (10, Some("Free all Grubs.")),
        (12, Some("Requires Fragile Heart.")),
        (14, Some("Requires Fragile Greed.")),
        (16, Some("Requires Fragile Strength.")),
        (19, Some("Requires Shopkeeper's Key.")),
        (22, Some("Defeat the Mantis Lords.")),
        (24, Some("Requires Mothwing Cloak.")),
        (26, Some("Defeat Flukemarm.")),
        (27, Some("Defeat Dung Defender.")),
        (28, Some("Requires Crystal Heart.")),
        (30, Some("Requires Crystal Heart.")),
        (
            32,
            Some("Requires 14 Lifeblood masks (15 if Joni's Blessing is equipped)."),
        ),
        (34, Some("Defeat Hive Knight.")),
        (36, Some("Requires Shade Cloak.")),
        (37, Some("Requires Isma's Tear.")),
        (38, Some("Acquire all 3 Nail Arts.")),
        (40, Some("Gather 500 Essence.")),
        (43, Some("Banish the Grimm Troupe.")),
        (44, Some("Collect both Kingsoul halves.")),
        (45, Some("Requires Kingsoul equipped.")),
    ];

    // Insert all charm requirements with INSERT INTO
    for (id, req) in charm_req {
        conn.execute(
            "INSERT INTO charms_requirements (id, condition_text) VALUES (?1, ?2)",
            params![id, req],
        )?;
    }

    // Insert all mask shard vendors with INSERT INTO
    for (id, cost) in [(1, 150), (2, 500), (3, 800), (4, 1500)] {
        conn.execute(
            "INSERT INTO mask_shards_vendor (id, vendor_name, cost) VALUES (?1, ?2, ?3)",
            params![id, "Sly", cost],
        )?;
    }

    // mask_shards_locations & requirements (wiki Mask Shard, How to Acquire)
    let mask_locs = [
        (1, "Dirtmouth"),
        (2, "Dirtmouth"),
        (3, "Dirtmouth"),
        (4, "Dirtmouth"),
        (5, "Forgotten Crossroads"),
        (6, "Forgotten Crossroads"),
        (7, "Forgotten Crossroads"),
        (8, "Fungal Wastes"),
        (9, "Dirtmouth"),
        (10, "Greenpath"),
        (11, "Royal Waterways"),
        (12, "Deepnest"),
        (13, "Crystal Peak"),
        (14, "Hive"),
        (15, "Resting Grounds"),
        (16, "Resting Grounds"),
    ];

    // Insert all mask shard locations with INSERT INTO
    for (id, loc) in mask_locs {
        conn.execute(
            "INSERT INTO mask_shards_locations (id, location_name) VALUES (?1, ?2)",
            params![id, loc],
        )?;
    }
    let mask_req: [(i32, Option<&str>); 15] = [
        (1, Some("Requires finding Sly in Forgotten Crossroads")),
        (2, Some("Requires finding Sly in Forgotten Crossroads")),
        (3, Some("Requires finding Sly and the Shopkeeper's Key")),
        (4, Some("Requires finding Sly and the Shopkeeper's Key")),
        (5, Some("Reward for defeating Brooding Mawlek")),
        (6, Some("Requires rescuing 5 Grubs")),
        (7, Some("Requires Mantis Claw")),
        (8, Some("Requires Mantis Claw")),
        (9, Some("Requires rescuing Bretta from Fungal Wastes")),
        (10, Some("Lumafly Lantern recommended")),
        (12, Some("Requires Monarch Wings")),
        (13, Some("Requires Monarch Wings")),
        (14, Some("Requires baiting a Hive Guardian into breaking a wall")),
        (15, Some("Requires collecting 1500 Essence")),
        (16, Some("Requires completing the Delicate Flower quest")),
    ];

    // Insert all mask shard requirements with INSERT INTO
    for (id, req) in mask_req {
        conn.execute(
            "INSERT INTO mask_shards_requirements (id, condition_text) VALUES (?1, ?2)",
            params![id, req],
        )?;
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

    println!("  {}", column_names.join(" | "));

    let mut rows = stmt.query([])?;
    let mut count = 0usize;
    while let Some(row) = rows.next()? {
        let mut rendered = Vec::with_capacity(column_names.len());
        for index in 0..column_names.len() {
            let value = row.get_ref(index)?;
            rendered.push(value_ref_to_string(value));
        }
        println!("  {}", rendered.join(" | "));
        count += 1;
    }

    println!("  ({count} row(s))");
    Ok(())
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

fn print_seeded_tables(conn: &Connection) -> Result<(), Box<dyn Error>> {
    println!("  Seeded table snapshots:");
    for (name, query) in [
        (
            "charms_locations",
            "SELECT id, location_name FROM charms_locations ORDER BY id",
        ),
        (
            "charms_requirements",
            "SELECT id, condition_text FROM charms_requirements ORDER BY id",
        ),
        (
            "charms_vendor",
            "SELECT id, vendor_name, cost FROM charms_vendor ORDER BY id",
        ),
        (
            "mask_shards_locations",
            "SELECT id, location_name FROM mask_shards_locations ORDER BY id",
        ),
        (
            "mask_shards_requirements",
            "SELECT id, condition_text FROM mask_shards_requirements ORDER BY id",
        ),
        (
            "mask_shards_vendor",
            "SELECT id, vendor_name, cost FROM mask_shards_vendor ORDER BY id",
        ),
        (
            "vendor_locations",
            "SELECT vendor_name, location_name FROM vendor_locations ORDER BY vendor_name",
        ),
    ] {
        println!("  - {name}");
        run_select_query(conn, query)?;
    }
    Ok(())
}
