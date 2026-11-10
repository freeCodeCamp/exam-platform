use crate::Database;

pub async fn create(
    db: &Database,
    id: &str,
    slug: &str,
    title: &str,
    instructions: &str,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO exams (id, slug, title, instructions, status) VALUES (?, ?, ?, ?, 'disabled')",
    )
    .bind(id)
    .bind(slug)
    .bind(title)
    .bind(instructions)
    .execute(&db.pool)
    .await?;
    Ok(())
}
