use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use opencv::core::{KeyPoint, Vector};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
struct ImageFeature {
    id: String,
    keypoints: Vec<KeyPointData>,
    descriptors: Vec<u8>,
    motion_mean: f64,
    motion_std: f64,
    created_at_utc: String,
    img_filename: Option<String>,
    camera_id: String,
}

#[derive(Serialize, Deserialize)]
struct KeyPointData {
    x: f32,
    y: f32,
    size: f32,
    angle: f32,
}

#[derive(Serialize, Deserialize)]
struct ImageDescription {
    image_name: String,
    datetime: String,
    camera_id: String,
    anomaly: Option<String>,
}

// Function to setup database
fn setup_database(db_name: &str) -> Result<()> {
    let conn = Connection::open(db_name)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS image_features (
            id TEXT PRIMARY KEY,
            keypoints BLOB,
            descriptors BLOB,
            motion_mean REAL,
            motion_std REAL,
            created_at_utc TEXT NOT NULL,
            img_filename TEXT,
            camera_id TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

// Function to setup image description table
fn image_description_table(db_name: &str) -> Result<()> {
    let conn = Connection::open(db_name)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS image_description (
            image_name TEXT PRIMARY KEY,
            datetime TEXT NOT NULL,
            camera_id TEXT NOT NULL,
            anomaly TEXT
        )",
        [],
    )?;
    Ok(())
}

// Function to insert image description
fn insert_image_description(image_description: &ImageDescription, db_name: &str) -> Result<()> {
    let conn = Connection::open(db_name)?;
    conn.execute(
        "INSERT INTO image_description (image_name, datetime, camera_id, anomaly)
        VALUES (?1, ?2, ?3, ?4)",
        params![
            image_description.image_name,
            image_description.datetime,
            image_description.camera_id,
            image_description.anomaly
        ],
    )?;
    Ok(())
}

// Function to clear test database
fn clear_test_db() -> Result<()> {
    let conn = Connection::open(TEST_DB)?;
    conn.execute("DROP TABLE IF EXISTS image_features", [])?;
    Ok(())
}

// Function to insert image feature
fn insert_image_feature(image_feature: &ImageFeature, db_name: &str) -> Result<()> {
    let conn = Connection::open(db_name)?;
    let keypoints = bincode::serialize(&image_feature.keypoints).unwrap();
    let descriptors = bincode::serialize(&image_feature.descriptors).unwrap();

    conn.execute(
        "INSERT INTO image_features (id, keypoints, descriptors, motion_mean, motion_std, created_at_utc, img_filename, camera_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            image_feature.id,
            keypoints,
            descriptors,
            image_feature.motion_mean,
            image_feature.motion_std,
            image_feature.created_at_utc,
            image_feature.img_filename,
            image_feature.camera_id
        ],
    )?;
    Ok(())
}

// Function to delete image feature
fn delete_image_feature(camera_id: &str, db_name: &str) -> Result<()> {
    let conn = Connection::open(db_name)?;
    conn.execute("DELETE FROM image_features WHERE camera_id = ?", params![camera_id])?;
    Ok(())
}

// Function to reset image feature
fn reset_image_feature(camera_id: &str, image_feature: &ImageFeature, db_name: &str) -> Result<()> {
    delete_image_feature(camera_id, db_name)?;
    insert_image_feature(image_feature, db_name)?;
    Ok(())
}

// Function to get image feature
fn get_image_feature(camera_id: &str, db_name: &str) -> Result<Option<ImageFeature>> {
    let conn = Connection::open(db_name)?;
    let mut stmt = conn.prepare("SELECT * FROM image_features WHERE camera_id = ?")?;
    let mut rows = stmt.query(params![camera_id])?;

    if let Some(row) = rows.next()? {
        let keypoints: Vec<KeyPointData> = bincode::deserialize(&row.get::<_, Vec<u8>>(1)?)?;
        let descriptors: Vec<u8> = bincode::deserialize(&row.get::<_, Vec<u8>>(2)?)?;
        let image_feature = ImageFeature {
            id: row.get(0)?,
            keypoints,
            descriptors,
            motion_mean: row.get(3)?,
            motion_std: row.get(4)?,
            created_at_utc: row.get(5)?,
            img_filename: row.get(6)?,
            camera_id: row.get(7)?,
        };
        Ok(Some(image_feature))
    } else {
        Ok(None)
    }
}

// Main function to test the setup
fn main() -> Result<()> {
    setup_database(PROD_DB)?;
    setup_database(TEST_DB)?;
    image_description_table(PROD_DB)?;
    image_description_table(TEST_DB)?;

    let image_description = ImageDescription {
        image_name: String::from("test_image.jpg"),
        datetime: String::from("2024-06-12T12:34:56Z"),
        camera_id: String::from("camera_1"),
        anomaly: None,
    };
    insert_image_description(&image_description, PROD_DB)?;

    let keypoints = vec![
        KeyPointData { x: 0.0, y: 0.0, size: 1.0, angle: 0.0 },
        KeyPointData { x: 1.0, y: 1.0, size: 2.0, angle: 45.0 },
    ];
    let descriptors = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    let image_feature = ImageFeature {
        id: String::from("1"),
        keypoints,
        descriptors,
        motion_mean: 0.5,
        motion_std: 0.1,
        created_at_utc: String::from("2024-06-12T12:34:56Z"),
        img_filename: Some(String::from("image_1.jpg")),
        camera_id: String::from("camera_1"),
    };

    insert_image_feature(&image_feature, PROD_DB)?;

    if let Some(feature) = get_image_feature("camera_1", PROD_DB)? {
        println!("Retrieved image feature: {:?}", feature);
    } else {
        println!("No image feature found for the given camera_id.");
    }

    Ok(())
}
