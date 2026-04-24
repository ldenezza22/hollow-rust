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
    println!("  Before update (charms_vendor where id = 4):");
    run_select_query(
        &conn,
        "SELECT id, vendor_name, cost FROM charms_vendor WHERE id = 4",
    )?;
    let updated = conn.execute(
        "UPDATE charms_vendor SET cost = ?1 WHERE vendor_name = ?2 AND cost = ?3",
        params![230, "Salubra", 220],
    )?;
    println!(
        "  Updated {updated} row(s): Salubra Shaman Stone offer adjusted from 220 to 230 geo (see wiki)."
    );
    println!("  After update (charms_vendor where id = 4):");
    run_select_query(
        &conn,
        "SELECT id, vendor_name, cost FROM charms_vendor WHERE id = 4",
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
        "SELECT id, vendor_name, cost FROM charms_vendor WHERE vendor_name = 'Leg Eater' ORDER BY id",
    )?;
    let deleted = conn.execute(
        "DELETE FROM charms_vendor WHERE vendor_name = ?1",
        params!["Leg Eater"],
    )?;
    println!("  Deleted {deleted} row(s) from charms_vendor for Leg Eater.");
    println!("  After delete (charms_vendor where vendor_name = 'Leg Eater'):");
    run_select_query(
        &conn,
        "SELECT id, vendor_name, cost FROM charms_vendor WHERE vendor_name = 'Leg Eater' ORDER BY id",
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

    // charms_vendor: Salubra shop table (wiki Salubra page)
    for (id, cost) in [
        (1, 250),  // Lifeblood Heart
        (2, 300),  // Longnail
        (3, 120),  // Steady Body
        (4, 220),  // Shaman Stone
        (5, 800),  // Quick Focus
        (6, 120),  // Charm Notch (own 5 Charms)
        (7, 500),  // Charm Notch (own 10 Charms)
        (8, 900),  // Charm Notch (own 18 Charms)
        (9, 1400), // Charm Notch (own 25 Charms)
        (10, 800), // Salubra's Blessing (own 40 Charms)
    ] {
        conn.execute(
            "INSERT INTO charms_vendor (id, vendor_name, cost) VALUES (?1, ?2, ?3)",
            params![id, "Salubra", cost],
        )?;
    }

    // Insert all Leg Eater charms
    for (id, cost) in [(11, 350), (12, 250), (13, 600)] {
        conn.execute(
            "INSERT INTO charms_vendor (id, vendor_name, cost) VALUES (?1, ?2, ?3)",
            params![id, "Leg Eater", cost],
        )?;
    }

    // charms_locations / requirements: List of Charms entries
    // Source: https://hollowknight.fandom.com/wiki/Category:Charms (List of Charms)
    let charm_locs = [
        (1, "Wayward Compass — Sold by Iselda for 220."),
        (2, "Gathering Swarm — Sold by Sly for 300."),
        (3, "Stalwart Shell — Sold by Sly for 200."),
        (4, "Soul Catcher — Ancestral Mound, west of Elder Baldur."),
        (5, "Shaman Stone — Sold by Salubra for 220."),
        (6, "Soul Eater — Resting Grounds."),
        (7, "Dashmaster — Fungal Wastes, south of Mantis Village."),
        (8, "Sprintmaster — Sold by Sly for 400."),
        (9, "Grubsong — Reward from Grubfather for freeing 10 Grubs."),
        (10, "Grubberfly's Elegy — Reward from Grubfather for freeing all Grubs."),
        (11, "Fragile Heart — Sold by Leg Eater for 350/280."),
        (12, "Unbreakable Heart — Received from Divine for 12000."),
        (13, "Fragile Greed — Sold by Leg Eater for 250/200."),
    ];

    // Insert all charm locations with INSERT INTO
    for (id, loc) in charm_locs {
        conn.execute(
            "INSERT INTO charms_locations (id, location_name) VALUES (?1, ?2)",
            params![id, loc],
        )?;
    }
    let charm_req = [
        (1, "Encounter Cornifer first."),
        (2, "None."),
        (3, "None."),
        (4, "None."),
        (5, "None."),
        (6, "Requires Desolate Dive."),
        (7, "None."),
        (8, "Requires Shopkeeper's Key."),
        (9, "Free 10 Grubs."),
        (10, "Free all Grubs."),
        (11, "None."),
        (12, "Requires Fragile Heart."),
        (13, "None."),
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
        (1, "Dirtmouth (bought from Sly)"),
        (2, "Dirtmouth (bought from Sly)"),
        (3, "Dirtmouth (bought from Sly)"),
        (4, "Dirtmouth (bought from Sly)"),
        (5, "Forgotten Crossroads (far west end)"),
        (6, "Forgotten Crossroads (Grubfather)"),
        (7, "Forgotten Crossroads (south of False Knight, Goam pit)"),
        (8, "Queen's Station (east side, Fungal Wastes)"),
        (9, "Dirtmouth (Bretta's house)"),
        (10, "Greenpath (Stone Sanctuary, north-east of No Eyes)"),
        (11, "Royal Waterways (north-west section, swim under main path)"),
        (12, "Deepnest (via Fungal Core, near Mantis Lords)"),
        (13, "Crystal Peak (Enraged Guardian reward)"),
        (14, "The Hive (wall broken by Hive Guardian)"),
        (15, "Resting Grounds (Seer)"),
        (16, "Resting Grounds (Grey Mourner)"),
    ];

    // Insert all mask shard locations with INSERT INTO
    for (id, loc) in mask_locs {
        conn.execute(
            "INSERT INTO mask_shards_locations (id, location_name) VALUES (?1, ?2)",
            params![id, loc],
        )?;
    }
    let mask_req = [
        (1, "Requires finding Sly in Forgotten Crossroads"),
        (2, "Requires finding Sly in Forgotten Crossroads"),
        (3, "Requires finding Sly and the Shopkeeper's Key"),
        (4, "Requires finding Sly and the Shopkeeper's Key"),
        (5, "Reward for defeating Brooding Mawlek"),
        (6, "Requires rescuing 5 Grubs"),
        (7, "Requires Mantis Claw"),
        (8, "Requires Mantis Claw"),
        (9, "Requires rescuing Bretta from Fungal Wastes"),
        (10, "Lumafly Lantern recommended"),
        (11, "n/a"),
        (12, "Requires Monarch Wings"),
        (13, "Requires Monarch Wings"),
        (14, "Requires baiting a Hive Guardian into breaking a wall"),
        (15, "Requires collecting 1500 Essence"),
        (16, "Requires completing the Delicate Flower quest"),
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
