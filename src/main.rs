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

    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    println!("Step 1: Initialize the database");
    initialize_schema(&conn)?;
    seed_demo_rows(&conn)?;
    println!("  Created schema and seed rows at {db_path}.");

    println!("Step 2: Update the database");
    let updated = conn.execute(
        "UPDATE charms_vendor SET cost = ?1 WHERE vendor_name = ?2 AND cost = ?3",
        params![230, "Salubra", 220],
    )?;
    println!(
        "  Updated {updated} row(s): Salubra Shaman Stone offer adjusted from 220 to 230 geo (see wiki)."
    );

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

    println!("Step 4: Delete from the database");
    let deleted = conn.execute(
        "DELETE FROM charms_locations WHERE id = ?1",
        params![8],
    )?;
    println!("  Deleted {deleted} row(s) from charms_locations (id 8: Gathering Swarm row).");

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

/// Seed data transcribed from the Hollow Knight Wiki (Fandom):
/// - Charm vendors & prices: https://hollowknight.fandom.com/wiki/Salubra
/// - Charm notch sources: https://hollowknight.fandom.com/wiki/Charms (Notches table)
/// - Mask shard acquisition: https://hollowknight.fandom.com/wiki/Mask_Shard
/// - Leg Eater fragile charm prices: https://hollowknight.fandom.com/wiki/Leg_Eater
///   (Fragile Heart / Fragile Greed / Fragile Strength shop entries)
fn seed_demo_rows(conn: &Connection) -> rusqlite::Result<()> {
    // --- vendor_locations (wiki shop NPC → area) ---
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

    // --- charms_vendor: Salubra shop table (wiki Salubra page) ---
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
    // Leg Eater (wiki Leg Eater: Fragile Heart / Greed / Strength geo costs)
    for (id, cost) in [(11, 350), (12, 250), (13, 600)] {
        conn.execute(
            "INSERT INTO charms_vendor (id, vendor_name, cost) VALUES (?1, ?2, ?3)",
            params![id, "Leg Eater", cost],
        )?;
    }

    // --- charms_locations / requirements: Charm Notch pickups (wiki Charms page, Notches) ---
    let charm_locs = [
        (1, "Fog Canyon (hidden area north-east of Cornifer)"),
        (2, "Fungal Wastes"),
        (3, "Colosseum of Fools"),
        (4, "Dirtmouth (inside the Grimm Troupe tent)"),
        (5, "Greenpath (near Moss Knight, east of Stone Sanctuary)"),
        (6, "Ancestral Mound (Howling Wraiths)"),
        (7, "Crystal Peak"),
        (
            8,
            "Forgotten Crossroads (south-east, near King's Pass entrance — Gathering Swarm)",
        ),
    ];
    for (id, loc) in charm_locs {
        conn.execute(
            "INSERT INTO charms_locations (id, location_name) VALUES (?1, ?2)",
            params![id, loc],
        )?;
    }
    let charm_req = [
        (1, "Unlock Isma's Tear or Monarch Wings"),
        (2, "Defeat 2 Shrumal Ogres"),
        (3, "Complete the Trial of the Warrior"),
        (4, "Defeat Grimm"),
        (5, "None (ground pickup)"),
        (6, "None (after defeating Gruz Mother)"),
        (7, "Crystal Heart or similar traversal"),
        (8, "None (early-game ground pickup)"),
    ];
    for (id, req) in charm_req {
        conn.execute(
            "INSERT INTO charms_requirements (id, condition_text) VALUES (?1, ?2)",
            params![id, req],
        )?;
    }

    // --- mask_shards_vendor: Sly in Dirtmouth (wiki Mask Shard — shards 1–4) ---
    for (id, cost) in [(1, 150), (2, 500), (3, 800), (4, 1500)] {
        conn.execute(
            "INSERT INTO mask_shards_vendor (id, vendor_name, cost) VALUES (?1, ?2, ?3)",
            params![id, "Sly", cost],
        )?;
    }

    // --- mask_shards_locations & requirements (wiki Mask Shard, How to Acquire) ---
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
