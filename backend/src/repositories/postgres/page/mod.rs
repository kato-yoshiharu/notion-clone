pub mod mock;
mod tests;

use super::{
    super::{error::RepositoryError, interfaces::page::IPageRepository},
    common::DateTimeUtc,
};
use async_trait::async_trait;
use sqlx::{query, query_as, Acquire, FromRow, PgPool, Postgres};
use std::sync::Arc;

define_id!(PageId, models::page::PageId);

#[derive(Debug, FromRow)]
struct Page {
    id: PageId,
    title: String,
    text: String,
    created_at: DateTimeUtc,
    updated_at: DateTimeUtc,
}

impl From<Page> for models::page::Page {
    fn from(value: Page) -> Self {
        Self {
            id: value.id.into(),
            title: value.title,
            text: value.text,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
        }
    }
}

#[derive(Debug, FromRow)]
struct PageRelationship {
    ancestor: PageId,
    descendant: PageId,
    // TODO: usize
    weight: i32,
}

impl From<PageRelationship> for models::page::PageRelationship {
    fn from(value: PageRelationship) -> Self {
        Self {
            ancestor: value.ancestor.into(),
            descendant: value.descendant.into(),
            weight: value.weight,
        }
    }
}

struct InternalPageRepository;

impl InternalPageRepository {
    async fn find_roots(
        acquire: impl Acquire<'_, Database = Postgres>,
    ) -> Result<Vec<models::page::Page>, RepositoryError> {
        let mut acquire = acquire.acquire().await?;

        let pages = query_as::<_, Page>(
            "
            WITH roots AS (
                SELECT descendant AS id
                FROM notion.page_relationships
                GROUP BY descendant
                HAVING COUNT(*) = 1
            ),
            sibling_descendant_counts AS (
                SELECT descendant, COUNT(*) AS count
                FROM notion.page_sibling_relationships
                GROUP BY descendant
            )
            SELECT notion.pages.id, title, text, created_at, updated_at
            FROM notion.pages
            JOIN roots
            ON notion.pages.id = roots.id
            JOIN sibling_descendant_counts
            ON notion.pages.id = sibling_descendant_counts.descendant
            ORDER BY sibling_descendant_counts.count
            ",
        )
        .fetch_all(&mut *acquire)
        .await?;

        Ok(pages.into_iter().map(Into::into).collect())
    }

    async fn find_children(
        id: &models::page::PageId,
        acquire: impl Acquire<'_, Database = Postgres>,
    ) -> Result<Vec<models::page::Page>, RepositoryError> {
        let mut acquire = acquire.acquire().await?;

        let pages = query_as::<_, Page>(
            "
            WITH children AS (
                SELECT descendant AS id
                FROM notion.page_relationships
                WHERE ancestor = $1
                AND weight = 1
            ),
            sibling_descendant_counts AS (
                SELECT descendant, COUNT(*) AS count
                FROM notion.page_sibling_relationships
                GROUP BY descendant
            )
            SELECT notion.pages.id, title, text, created_at, updated_at
            FROM notion.pages
            JOIN children
            ON notion.pages.id = children.id
            JOIN sibling_descendant_counts
            ON notion.pages.id = sibling_descendant_counts.descendant
            ORDER BY sibling_descendant_counts.count
            ",
        )
        .bind(id.0)
        .fetch_all(&mut *acquire)
        .await?;

        Ok(pages.into_iter().map(Into::into).collect())
    }

    async fn find_ancestors(
        id: &models::page::PageId,
        acquire: impl Acquire<'_, Database = Postgres>,
    ) -> Result<Vec<models::page::Page>, RepositoryError> {
        let mut acquire = acquire.acquire().await?;

        let pages = query_as::<_, Page>(
            "
            SELECT id, title, text, created_at, updated_at
            FROM notion.pages
            JOIN notion.page_relationships
            ON notion.pages.id = notion.page_relationships.ancestor
            AND notion.page_relationships.descendant = $1
            ORDER BY notion.page_relationships.weight DESC
            ",
        )
        .bind(id.0)
        .fetch_all(&mut *acquire)
        .await?;

        Ok(pages.into_iter().map(Into::into).collect())
    }

    async fn find_descendants(
        id: &models::page::PageId,
        acquire: impl Acquire<'_, Database = Postgres>,
    ) -> Result<(Vec<models::page::Page>, Vec<models::page::PageRelationship>), RepositoryError>
    {
        let mut acquire = acquire.acquire().await?;

        let pages = query_as::<_, Page>(
            "
            WITH descendants AS (
                SELECT descendant AS id
                FROM notion.page_relationships
                WHERE ancestor = $1
            ),
            descendant_counts AS (
                SELECT descendant, COUNT(*) AS count
                FROM notion.page_relationships
                GROUP BY descendant
            ),
            sibling_descendant_counts AS (
                SELECT descendant, COUNT(*) AS count
                FROM notion.page_sibling_relationships
                GROUP BY descendant
            )
            SELECT notion.pages.id, title, text, created_at, updated_at
            FROM notion.pages
            JOIN descendants
            ON notion.pages.id = descendants.id
            JOIN descendant_counts
            ON notion.pages.id = descendant_counts.descendant
            JOIN sibling_descendant_counts
            ON notion.pages.id = sibling_descendant_counts.descendant
            ORDER BY descendant_counts.count, sibling_descendant_counts.count
            ",
        )
        .bind(id.0)
        .fetch_all(&mut *acquire)
        .await?;

        // 以下でも可能
        // WITH descendants AS (
        //     SELECT descendant AS id
        //     FROM notion.page_relationships
        //     WHERE ancestor = $1
        // ),
        // parent_child_relationships AS (
        //     SELECT ancestor, descendant, weight
        //     FROM notion.page_relationships
        //     WHERE ancestor IN (
        //         SELECT id
        //         FROM descendants
        //     )
        //     AND weight = 1
        // ),

        let parent_child_relationships = query_as::<_, PageRelationship>(
            "
            WITH RECURSIVE parent_child_relationships AS (
                SELECT ancestor, descendant, weight
                FROM notion.page_relationships
                WHERE ancestor = $1
                AND weight = 1
                UNION ALL
                SELECT child.ancestor, child.descendant, child.weight
                FROM parent_child_relationships
                JOIN notion.page_relationships AS child
                ON parent_child_relationships.descendant = child.ancestor
                AND child.weight = 1
            ),
            descendant_counts AS (
                SELECT descendant, COUNT(*) AS count
                FROM notion.page_relationships
                GROUP BY descendant
            ),
            sibling_descendant_counts AS (
                SELECT descendant, COUNT(*) AS count
                FROM notion.page_sibling_relationships
                GROUP BY descendant
            )
            SELECT parent_child_relationships.ancestor, parent_child_relationships.descendant, parent_child_relationships.weight
            FROM parent_child_relationships
            JOIN descendant_counts
            ON parent_child_relationships.descendant = descendant_counts.descendant
            JOIN sibling_descendant_counts
            ON parent_child_relationships.descendant = sibling_descendant_counts.descendant
            ORDER BY descendant_counts.count, sibling_descendant_counts.count
            ",
        )
        .bind(id.0)
        .fetch_all(&mut *acquire)
        .await?;

        Ok((
            pages.into_iter().map(Into::into).collect(),
            parent_child_relationships
                .into_iter()
                .map(Into::into)
                .collect(),
        ))
    }

    async fn find_by_id(
        id: &models::page::PageId,
        acquire: impl Acquire<'_, Database = Postgres>,
    ) -> Result<models::page::Page, RepositoryError> {
        let mut acquire = acquire.acquire().await?;

        let page = query_as::<_, Page>(
            "
            SELECT id, title, text, created_at, updated_at
            FROM notion.pages
            WHERE id = $1
            ",
        )
        .bind(id.0)
        .fetch_one(&mut *acquire)
        .await?;

        Ok(page.into())
    }

    async fn add(
        parent_id: &Option<models::page::PageId>,
        add_page: models::page::AddPage,
        acquire: impl Acquire<'_, Database = Postgres>,
    ) -> Result<models::page::Page, RepositoryError> {
        let mut tx = acquire.begin().await?;

        let page = query_as::<_, Page>(
            "
            INSERT INTO notion.pages (id, title, text)
            VALUES (COALESCE($1, uuid_generate_v4()), $2, $3)
            RETURNING id, title, text, created_at, updated_at
            ",
        )
        .bind(add_page.id.map(|i| i.0))
        .bind(add_page.title)
        .bind(add_page.text)
        .fetch_one(&mut *tx)
        .await?;

        query(
            "
            INSERT INTO notion.page_relationships (ancestor, descendant, weight)
            SELECT ancestor, $2, weight + 1
            FROM notion.page_relationships
            WHERE descendant = $1
            UNION ALL
            SELECT $2, $2, 0
            ",
        )
        .bind(parent_id.as_ref().map(|p| p.0))
        .bind(&page.id)
        .execute(&mut *tx)
        .await?;

        let sibling_parent_id: Option<PageId> = match parent_id {
            Some(parent_id) => query_as::<_, Page>(
                "
                WITH children AS (
                    SELECT descendant AS id
                    FROM notion.page_relationships
                    WHERE ancestor = $1
                    AND weight = 1
                ),
                sibling_leaves AS (
                    SELECT ancestor AS id
                    FROM notion.page_sibling_relationships
                    GROUP BY ancestor
                    HAVING COUNT(*) = 1
                )
                SELECT notion.pages.id, title, text, created_at, updated_at
                FROM notion.pages
                JOIN children
                ON notion.pages.id = children.id
                JOIN sibling_leaves
                ON notion.pages.id = sibling_leaves.id
                ",
            )
            .bind(parent_id.0)
            .fetch_optional(&mut *tx)
            .await?
            .map(|p| p.id),
            None => query_as::<_, Page>(
                "
                WITH roots AS (
                    SELECT descendant AS id
                    FROM notion.page_relationships
                    GROUP BY descendant
                    HAVING COUNT(*) = 1
                ),
                sibling_leaves AS (
                    SELECT ancestor AS id
                    FROM notion.page_sibling_relationships
                    GROUP BY ancestor
                    HAVING COUNT(*) = 1
                )
                SELECT notion.pages.id, title, text, created_at, updated_at
                FROM notion.pages
                JOIN roots
                ON notion.pages.id = roots.id
                JOIN sibling_leaves
                ON notion.pages.id = sibling_leaves.id
                ",
            )
            .fetch_optional(&mut *tx)
            .await?
            .map(|p| p.id),
        };
        query(
            "
            INSERT INTO notion.page_sibling_relationships (ancestor, descendant, weight)
            SELECT ancestor, $2, weight + 1
            FROM notion.page_sibling_relationships
            WHERE descendant = $1
            UNION ALL
            SELECT $2, $2, 0
            ",
        )
        .bind(sibling_parent_id)
        .bind(&page.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(page.into())
    }

    async fn update(
        id: &models::page::PageId,
        update_page: models::page::UpdatePage,
        acquire: impl Acquire<'_, Database = Postgres>,
    ) -> Result<models::page::Page, RepositoryError> {
        let mut acquire = acquire.acquire().await?;

        let page = query_as::<_, Page>(
            "
            UPDATE notion.pages
            SET title = COALESCE($2, title), text = COALESCE($3, text), updated_at = NOW()
            WHERE id = $1
            RETURNING id, title, text, created_at, updated_at
            ",
        )
        .bind(id.0)
        .bind(update_page.title)
        .bind(update_page.text)
        .fetch_one(&mut *acquire)
        .await?;

        Ok(page.into())
    }

    async fn remove(
        id: &models::page::PageId,
        acquire: impl Acquire<'_, Database = Postgres>,
    ) -> Result<(), RepositoryError> {
        let mut tx = acquire.begin().await?;

        query(
            "
            UPDATE notion.page_sibling_relationships
            SET weight = weight - 1
            WHERE ancestor IN (
                SELECT ancestor
                FROM notion.page_sibling_relationships
                WHERE descendant = $1
                AND ancestor != $1
            )
            AND descendant IN (
                SELECT descendant
                FROM notion.page_sibling_relationships
                WHERE ancestor = $1
                AND descendant != $1
            )
            ",
        )
        .bind(id.0)
        .execute(&mut *tx)
        .await?;

        query(
            "
            DELETE FROM notion.pages
            WHERE id IN (
                SELECT descendant
                FROM notion.page_relationships
                WHERE ancestor = $1
            )
            ",
        )
        .bind(id.0)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn move_(
        id: &models::page::PageId,
        target: &models::page::MoveTarget,
        acquire: impl Acquire<'_, Database = Postgres>,
    ) -> Result<(), RepositoryError> {
        let mut tx = acquire.begin().await?;

        query(
            "
            DELETE FROM notion.page_relationships
            WHERE ancestor IN (
                SELECT ancestor
                FROM notion.page_relationships
                WHERE descendant = $1
                AND ancestor != $1
            )
            AND descendant IN (
                SELECT descendant
                FROM notion.page_relationships
                WHERE ancestor = $1
            )
            ",
        )
        .bind(id.0)
        .execute(&mut *tx)
        .await?;
        let to_parent_id: Option<models::page::PageId> = match target {
            models::page::MoveTarget::Root => None,
            models::page::MoveTarget::Parent(to_parent_id) => Some(*to_parent_id),
            models::page::MoveTarget::SiblingParent(to_sibling_id)
            | models::page::MoveTarget::SiblingChild(to_sibling_id) => query_as::<_, Page>(
                "
                WITH parent AS (
                    SELECT ancestor AS id
                    FROM notion.page_relationships
                    WHERE descendant = $1
                    AND weight = 1
                )
                SELECT notion.pages.id, title, text, created_at, updated_at
                FROM notion.pages
                JOIN parent
                ON notion.pages.id = parent.id
                ",
            )
            .bind(to_sibling_id.0)
            .fetch_optional(&mut *tx)
            .await?
            .map(|p| p.id.into()),
        };
        if let Some(to_parent_id) = to_parent_id {
            query(
                "
                INSERT INTO notion.page_relationships (ancestor, descendant, weight)
                SELECT parent.ancestor, child.descendant, parent.weight + child.weight + 1
                FROM notion.page_relationships AS parent
                JOIN notion.page_relationships AS child
                ON parent.descendant = $1
                AND child.ancestor = $2
                ",
            )
            .bind(to_parent_id.0)
            .bind(id.0)
            .execute(&mut *tx)
            .await?;
        }

        query(
            "
            UPDATE notion.page_sibling_relationships
            SET weight = weight - 1
            WHERE ancestor IN (
                SELECT ancestor
                FROM notion.page_sibling_relationships
                WHERE descendant = $1
                AND ancestor != $1
            )
            AND descendant IN (
                SELECT descendant
                FROM notion.page_sibling_relationships
                WHERE ancestor = $1
                AND descendant != $1
            )
            ",
        )
        .bind(id.0)
        .execute(&mut *tx)
        .await?;
        query(
            "
            DELETE FROM notion.page_sibling_relationships
            WHERE (ancestor = $1 OR descendant = $1)
            AND ancestor != descendant
            ",
        )
        .bind(id.0)
        .execute(&mut *tx)
        .await?;
        match target {
            models::page::MoveTarget::Root | models::page::MoveTarget::Parent(_) => {
                let sibling_leave_id_of_to_parent = match target {
                    models::page::MoveTarget::Root => query_as::<_, Page>(
                        "
                        WITH roots AS (
                            SELECT descendant AS id
                            FROM notion.page_relationships
                            GROUP BY descendant
                            HAVING COUNT(*) = 1
                        ),
                        sibling_leaves AS (
                            SELECT ancestor AS id
                            FROM notion.page_sibling_relationships
                            GROUP BY ancestor
                            HAVING COUNT(*) = 1
                        )
                        SELECT notion.pages.id, title, text, created_at, updated_at
                        FROM notion.pages
                        JOIN roots
                        ON notion.pages.id = roots.id
                        JOIN sibling_leaves
                        ON notion.pages.id = sibling_leaves.id
                        WHERE notion.pages.id != $1
                        ",
                    )
                    .bind(id.0)
                    .fetch_optional(&mut *tx)
                    .await?
                    .map(|p| p.id),
                    models::page::MoveTarget::Parent(to_parent_id) => query_as::<_, Page>(
                        "
                        WITH children AS (
                            SELECT descendant AS id
                            FROM notion.page_relationships
                            WHERE ancestor = $1
                            AND weight = 1
                        ),
                        sibling_leaves AS (
                            SELECT ancestor AS id
                            FROM notion.page_sibling_relationships
                            GROUP BY ancestor
                            HAVING COUNT(*) = 1
                        )
                        SELECT notion.pages.id, title, text, created_at, updated_at
                        FROM notion.pages
                        JOIN children
                        ON notion.pages.id = children.id
                        JOIN sibling_leaves
                        ON notion.pages.id = sibling_leaves.id
                        WHERE notion.pages.id != $2
                        ",
                    )
                    .bind(to_parent_id.0)
                    .bind(id.0)
                    .fetch_optional(&mut *tx)
                    .await?
                    .map(|p| p.id),
                    _ => unreachable!(),
                };
                if let Some(sibling_leave_id_of_to_parent) = sibling_leave_id_of_to_parent {
                    query(
                        "
                        INSERT INTO notion.page_sibling_relationships (ancestor, descendant, weight)
                        SELECT ancestor, $2, weight + 1
                        FROM notion.page_sibling_relationships
                        WHERE descendant = $1
                        ",
                    )
                    .bind(sibling_leave_id_of_to_parent)
                    .bind(id.0)
                    .execute(&mut *tx)
                    .await?;
                }
            }
            models::page::MoveTarget::SiblingParent(to_sibling_parent_id) => {
                query(
                    "
                    INSERT INTO notion.page_sibling_relationships (ancestor, descendant, weight)
                    SELECT ancestor, $2, weight + 1
                    FROM notion.page_sibling_relationships
                    WHERE descendant = $1
                    UNION ALL
                    SELECT $2, descendant, weight
                    FROM notion.page_sibling_relationships
                    WHERE ancestor = $1
                    AND descendant != $1
                    ",
                )
                .bind(to_sibling_parent_id.0)
                .bind(id.0)
                .execute(&mut *tx)
                .await?;
            }
            models::page::MoveTarget::SiblingChild(to_sibling_child_id) => {
                query(
                    "
                    INSERT INTO notion.page_sibling_relationships (ancestor, descendant, weight)
                    SELECT ancestor, $2, weight
                    FROM notion.page_sibling_relationships
                    WHERE descendant = $1
                    AND ancestor != $1
                    UNION ALL
                    SELECT $2, descendant, weight + 1
                    FROM notion.page_sibling_relationships
                    WHERE ancestor = $1
                    ",
                )
                .bind(to_sibling_child_id.0)
                .bind(id.0)
                .execute(&mut *tx)
                .await?;
            }
        }
        query(
            "
            UPDATE notion.page_sibling_relationships
            SET weight = weight + 1
            WHERE ancestor IN (
                SELECT ancestor
                FROM notion.page_sibling_relationships
                WHERE descendant = $1
                AND ancestor != $1
            )
            AND descendant IN (
                SELECT descendant
                FROM notion.page_sibling_relationships
                WHERE ancestor = $1
                AND descendant != $1
            )
            ",
        )
        .bind(id.0)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }
}

pub struct PageRepository {
    pool: Arc<PgPool>,
}

impl PageRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IPageRepository for PageRepository {
    async fn find_roots(&self) -> Result<Vec<models::page::Page>, RepositoryError> {
        let pages = InternalPageRepository::find_roots(&*self.pool).await?;

        Ok(pages)
    }

    async fn find_children(
        &self,
        id: &models::page::PageId,
    ) -> Result<Vec<models::page::Page>, RepositoryError> {
        let pages = InternalPageRepository::find_children(id, &*self.pool).await?;

        Ok(pages)
    }

    async fn find_ancestors(
        &self,
        id: &models::page::PageId,
    ) -> Result<Vec<models::page::Page>, RepositoryError> {
        let pages = InternalPageRepository::find_ancestors(id, &*self.pool).await?;

        Ok(pages)
    }

    async fn find_descendants(
        &self,
        id: &models::page::PageId,
    ) -> Result<(Vec<models::page::Page>, Vec<models::page::PageRelationship>), RepositoryError>
    {
        let response = InternalPageRepository::find_descendants(id, &*self.pool).await?;

        Ok(response)
    }

    async fn find_by_id(
        &self,
        id: &models::page::PageId,
    ) -> Result<models::page::Page, RepositoryError> {
        let page = InternalPageRepository::find_by_id(id, &*self.pool).await?;

        Ok(page)
    }

    async fn add(
        &self,
        parent_id: &Option<models::page::PageId>,
        add_page: models::page::AddPage,
    ) -> Result<models::page::Page, RepositoryError> {
        let page = InternalPageRepository::add(parent_id, add_page, &*self.pool).await?;

        Ok(page)
    }

    async fn update(
        &self,
        id: &models::page::PageId,
        update_page: models::page::UpdatePage,
    ) -> Result<models::page::Page, RepositoryError> {
        let page = InternalPageRepository::update(id, update_page, &*self.pool).await?;

        Ok(page)
    }

    async fn remove(&self, id: &models::page::PageId) -> Result<(), RepositoryError> {
        InternalPageRepository::remove(id, &*self.pool).await?;

        Ok(())
    }

    async fn move_(
        &self,
        id: &models::page::PageId,
        target: &models::page::MoveTarget,
    ) -> Result<(), RepositoryError> {
        InternalPageRepository::move_(id, target, &*self.pool).await?;

        Ok(())
    }
}
