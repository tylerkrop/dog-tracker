use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr};

pub async fn run(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.execute_unprepared("PRAGMA journal_mode=WAL").await?;
    db.execute_unprepared("PRAGMA busy_timeout=5000").await?;

    db.execute_unprepared(
        "CREATE TABLE IF NOT EXISTS feeding (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            amount_half_scoops INTEGER NOT NULL CHECK(amount_half_scoops IN (1, 2)),
            fed_at TEXT NOT NULL,
            edited INTEGER NOT NULL DEFAULT 0
        )",
    )
    .await?;

    db.execute_unprepared(
        "CREATE INDEX IF NOT EXISTS idx_feeding_fed_at ON feeding (fed_at)",
    )
    .await?;

    db.execute_unprepared(
        "CREATE TABLE IF NOT EXISTS treat (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            given_at TEXT NOT NULL,
            edited INTEGER NOT NULL DEFAULT 0
        )",
    )
    .await?;

    db.execute_unprepared(
        "CREATE INDEX IF NOT EXISTS idx_treat_given_at ON treat (given_at)",
    )
    .await?;

    Ok(())
}
