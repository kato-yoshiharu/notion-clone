#![cfg(test)]

use super::{
    mock::{insert_mock, InsertMockResponse},
    *,
};
use crate::models::page::SimplePageRelationship;
use sqlx::{Executor, Postgres, Transaction};
use std::collections::HashSet;

async fn setup<'a>() -> anyhow::Result<(InsertMockResponse, Transaction<'a, Postgres>)> {
    let (insert_mock_response, tx) = insert_mock().await?;

    Ok((insert_mock_response, tx))
}

async fn teardown(tx: Transaction<'_, Postgres>) -> anyhow::Result<()> {
    tx.rollback().await?;

    Ok(())
}

async fn get_page_relationships<'e, 'c: 'e, E>(
    executor: E,
) -> anyhow::Result<HashSet<SimplePageRelationship>>
where
    E: 'e + Executor<'c, Database = Postgres>,
{
    let page_relationships =
        query_as::<_, PageRelationship>("SELECT * FROM notion.page_relationships")
            .fetch_all(executor)
            .await?
            .into_iter()
            .map(Into::into)
            .collect::<HashSet<crate::models::page::PageRelationship>>()
            .into_iter()
            .map(Into::into)
            .collect::<_>();

    Ok(page_relationships)
}

async fn get_page_sibling_relationships<'e, 'c: 'e, E>(
    executor: E,
) -> anyhow::Result<HashSet<SimplePageRelationship>>
where
    E: 'e + Executor<'c, Database = Postgres>,
{
    let page_sibling_relationships =
        query_as::<_, PageRelationship>("SELECT * FROM notion.page_sibling_relationships")
            .fetch_all(executor)
            .await?
            .into_iter()
            .map(Into::into)
            .collect::<HashSet<crate::models::page::PageRelationship>>()
            .into_iter()
            .map(Into::into)
            .collect::<_>();

    Ok(page_sibling_relationships)
}

#[tokio::test]
async fn find_roots_should_succeed() -> anyhow::Result<()> {
    let (
        InsertMockResponse {
            page_1,
            page_2,
            page_3,
            page_1_1: _,
            page_1_2: _,
            page_1_3: _,
            page_1_1_1: _,
        },
        mut tx,
    ) = setup().await?;

    let pages = InternalPageRepository::find_roots(&mut tx).await?;
    assert_eq!(
        pages.into_iter().map(|p| p.id).collect::<Vec<_>>(),
        vec![page_1.id, page_2.id, page_3.id]
    );

    teardown(tx).await?;

    Ok(())
}

#[tokio::test]
async fn find_children_should_succeed() -> anyhow::Result<()> {
    let (
        InsertMockResponse {
            page_1,
            page_2: _,
            page_3: _,
            page_1_1,
            page_1_2,
            page_1_3,
            page_1_1_1: _,
        },
        mut tx,
    ) = setup().await?;

    let pages = InternalPageRepository::find_children(&page_1.id, &mut tx).await?;
    assert_eq!(
        pages.into_iter().map(|p| p.id).collect::<Vec<_>>(),
        vec![page_1_1.id, page_1_2.id, page_1_3.id]
    );

    teardown(tx).await?;

    Ok(())
}

#[tokio::test]
async fn find_ancestors_should_succeed() -> anyhow::Result<()> {
    let (
        InsertMockResponse {
            page_1,
            page_2: _,
            page_3: _,
            page_1_1,
            page_1_2: _,
            page_1_3: _,
            page_1_1_1,
        },
        mut tx,
    ) = setup().await?;

    let pages = InternalPageRepository::find_ancestors(&page_1_1_1.id, &mut tx).await?;
    assert_eq!(
        pages.into_iter().map(|p| p.id).collect::<Vec<_>>(),
        vec![page_1.id, page_1_1.id, page_1_1_1.id]
    );

    teardown(tx).await?;

    Ok(())
}

#[tokio::test]
async fn find_descendants_should_succeed() -> anyhow::Result<()> {
    let (
        InsertMockResponse {
            page_1,
            page_2: _,
            page_3: _,
            page_1_1,
            page_1_2,
            page_1_3,
            page_1_1_1,
        },
        mut tx,
    ) = setup().await?;

    let (pages, page_relationships) =
        InternalPageRepository::find_descendants(&page_1.id, &mut tx).await?;
    assert_eq!(
        pages.into_iter().map(|p| p.id).collect::<Vec<_>>(),
        vec![
            page_1.id,
            page_1_1.id,
            page_1_2.id,
            page_1_3.id,
            page_1_1_1.id,
        ]
    );
    assert_eq!(
        page_relationships.into_iter().collect::<Vec<_>>(),
        vec![
            crate::models::page::PageRelationship {
                ancestor: page_1.id,
                descendant: page_1_1.id,
                weight: 1
            },
            crate::models::page::PageRelationship {
                ancestor: page_1.id,
                descendant: page_1_2.id,
                weight: 1
            },
            crate::models::page::PageRelationship {
                ancestor: page_1.id,
                descendant: page_1_3.id,
                weight: 1
            },
            crate::models::page::PageRelationship {
                ancestor: page_1_1.id,
                descendant: page_1_1_1.id,
                weight: 1
            },
        ]
    );

    teardown(tx).await?;

    Ok(())
}

#[tokio::test]
async fn find_by_id_should_succeed() -> anyhow::Result<()> {
    let (
        InsertMockResponse {
            page_1,
            page_2: _,
            page_3: _,
            page_1_1: _,
            page_1_2: _,
            page_1_3: _,
            page_1_1_1: _,
        },
        mut tx,
    ) = setup().await?;

    let page = InternalPageRepository::find_by_id(&page_1.id, &mut tx).await?;
    assert_eq!(page_1.id, page.id);

    teardown(tx).await?;

    Ok(())
}

#[tokio::test]
async fn add_should_succeed() -> anyhow::Result<()> {
    let (
        InsertMockResponse {
            page_1,
            page_2,
            page_3,
            page_1_1,
            page_1_2,
            page_1_3,
            page_1_1_1,
        },
        mut tx,
    ) = setup().await?;

    let page_relationships = get_page_relationships(&mut tx).await?;
    assert_eq!(
        HashSet::from([
            SimplePageRelationship(page_1.id, page_1.id, 0),
            SimplePageRelationship(page_1.id, page_1_1.id, 1),
            SimplePageRelationship(page_1.id, page_1_2.id, 1),
            SimplePageRelationship(page_1.id, page_1_3.id, 1),
            SimplePageRelationship(page_1.id, page_1_1_1.id, 2),
            SimplePageRelationship(page_2.id, page_2.id, 0),
            SimplePageRelationship(page_3.id, page_3.id, 0),
            SimplePageRelationship(page_1_1.id, page_1_1.id, 0),
            SimplePageRelationship(page_1_1.id, page_1_1_1.id, 1),
            SimplePageRelationship(page_1_2.id, page_1_2.id, 0),
            SimplePageRelationship(page_1_3.id, page_1_3.id, 0),
            SimplePageRelationship(page_1_1_1.id, page_1_1_1.id, 0)
        ]),
        page_relationships
    );
    let page_sibling_relationships = get_page_sibling_relationships(&mut tx).await?;
    assert_eq!(
        HashSet::from([
            SimplePageRelationship(page_1.id, page_1.id, 0),
            SimplePageRelationship(page_1.id, page_2.id, 1),
            SimplePageRelationship(page_1.id, page_3.id, 2),
            SimplePageRelationship(page_2.id, page_2.id, 0),
            SimplePageRelationship(page_2.id, page_3.id, 1),
            SimplePageRelationship(page_3.id, page_3.id, 0),
            SimplePageRelationship(page_1_1.id, page_1_1.id, 0),
            SimplePageRelationship(page_1_1.id, page_1_2.id, 1),
            SimplePageRelationship(page_1_1.id, page_1_3.id, 2),
            SimplePageRelationship(page_1_2.id, page_1_2.id, 0),
            SimplePageRelationship(page_1_2.id, page_1_3.id, 1),
            SimplePageRelationship(page_1_3.id, page_1_3.id, 0),
            SimplePageRelationship(page_1_1_1.id, page_1_1_1.id, 0)
        ]),
        page_sibling_relationships
    );

    teardown(tx).await?;

    Ok(())
}

#[cfg(test)]
mod update_tests {
    use super::*;

    #[tokio::test]
    async fn update_should_succeed() -> anyhow::Result<()> {
        let (
            InsertMockResponse {
                page_1,
                page_2: _,
                page_3: _,
                page_1_1: _,
                page_1_2: _,
                page_1_3: _,
                page_1_1_1: _,
            },
            mut tx,
        ) = setup().await?;

        let page = InternalPageRepository::update(
            &page_1.id,
            crate::models::page::UpdatePage {
                title: Some("new title".to_string()),
                text: Some("new text".to_string()),
            },
            &mut tx,
        )
        .await?;
        assert_eq!("new title", page.title);
        assert_eq!("new text", page.text);

        teardown(tx).await?;

        Ok(())
    }

    #[tokio::test]
    async fn do_not_update_when_title_and_text_are_none() -> anyhow::Result<()> {
        let (
            InsertMockResponse {
                page_1,
                page_2: _,
                page_3: _,
                page_1_1: _,
                page_1_2: _,
                page_1_3: _,
                page_1_1_1: _,
            },
            mut tx,
        ) = setup().await?;

        let page = InternalPageRepository::update(
            &page_1.id,
            crate::models::page::UpdatePage {
                title: None,
                text: None,
            },
            &mut tx,
        )
        .await?;
        assert_eq!("1", page.title);
        assert_eq!("", page.text);

        teardown(tx).await?;

        Ok(())
    }
}

#[tokio::test]
async fn remove_should_succeed() -> anyhow::Result<()> {
    let (
        InsertMockResponse {
            page_1,
            page_2,
            page_3,
            page_1_1,
            page_1_2,
            page_1_3,
            page_1_1_1: _,
        },
        mut tx,
    ) = setup().await?;

    InternalPageRepository::remove(&page_1_1.id, &mut tx).await?;

    let page_relationships = get_page_relationships(&mut tx).await?;
    assert_eq!(
        HashSet::from([
            SimplePageRelationship(page_1.id, page_1.id, 0),
            SimplePageRelationship(page_1.id, page_1_2.id, 1),
            SimplePageRelationship(page_1.id, page_1_3.id, 1),
            SimplePageRelationship(page_2.id, page_2.id, 0),
            SimplePageRelationship(page_3.id, page_3.id, 0),
            SimplePageRelationship(page_1_2.id, page_1_2.id, 0),
            SimplePageRelationship(page_1_3.id, page_1_3.id, 0)
        ]),
        page_relationships
    );
    let page_sibling_relationships = get_page_sibling_relationships(&mut tx).await?;
    assert_eq!(
        HashSet::from([
            SimplePageRelationship(page_1.id, page_1.id, 0),
            SimplePageRelationship(page_1.id, page_2.id, 1),
            SimplePageRelationship(page_1.id, page_3.id, 2),
            SimplePageRelationship(page_2.id, page_2.id, 0),
            SimplePageRelationship(page_2.id, page_3.id, 1),
            SimplePageRelationship(page_3.id, page_3.id, 0),
            SimplePageRelationship(page_1_2.id, page_1_2.id, 0),
            SimplePageRelationship(page_1_2.id, page_1_3.id, 1),
            SimplePageRelationship(page_1_3.id, page_1_3.id, 0)
        ]),
        page_sibling_relationships
    );

    teardown(tx).await?;

    Ok(())
}

#[tokio::test]
async fn remove_middle_should_succeed() -> anyhow::Result<()> {
    let (
        InsertMockResponse {
            page_1,
            page_2,
            page_3,
            page_1_1,
            page_1_2,
            page_1_3,
            page_1_1_1,
        },
        mut tx,
    ) = setup().await?;

    InternalPageRepository::remove(&page_1_2.id, &mut tx).await?;

    let page_relationships = get_page_relationships(&mut tx).await?;
    assert_eq!(
        HashSet::from([
            SimplePageRelationship(page_1.id, page_1.id, 0),
            SimplePageRelationship(page_1.id, page_1_1.id, 1),
            SimplePageRelationship(page_1.id, page_1_3.id, 1),
            SimplePageRelationship(page_1.id, page_1_1_1.id, 2),
            SimplePageRelationship(page_2.id, page_2.id, 0),
            SimplePageRelationship(page_3.id, page_3.id, 0),
            SimplePageRelationship(page_1_1.id, page_1_1.id, 0),
            SimplePageRelationship(page_1_1.id, page_1_1_1.id, 1),
            SimplePageRelationship(page_1_3.id, page_1_3.id, 0),
            SimplePageRelationship(page_1_1_1.id, page_1_1_1.id, 0)
        ]),
        page_relationships
    );
    let page_sibling_relationships = get_page_sibling_relationships(&mut tx).await?;
    assert_eq!(
        HashSet::from([
            SimplePageRelationship(page_1.id, page_1.id, 0),
            SimplePageRelationship(page_1.id, page_2.id, 1),
            SimplePageRelationship(page_1.id, page_3.id, 2),
            SimplePageRelationship(page_2.id, page_2.id, 0),
            SimplePageRelationship(page_2.id, page_3.id, 1),
            SimplePageRelationship(page_3.id, page_3.id, 0),
            SimplePageRelationship(page_1_1.id, page_1_1.id, 0),
            SimplePageRelationship(page_1_1.id, page_1_3.id, 1),
            SimplePageRelationship(page_1_3.id, page_1_3.id, 0),
            SimplePageRelationship(page_1_1_1.id, page_1_1_1.id, 0)
        ]),
        page_sibling_relationships
    );

    teardown(tx).await?;

    Ok(())
}

#[cfg(test)]
mod move_tests {
    use super::*;

    #[tokio::test]
    async fn move_to_root_should_succeed() -> anyhow::Result<()> {
        let (
            InsertMockResponse {
                page_1,
                page_2,
                page_3,
                page_1_1,
                page_1_2,
                page_1_3,
                page_1_1_1,
            },
            mut tx,
        ) = setup().await?;

        InternalPageRepository::move_(
            &page_1_1.id,
            &crate::models::page::MoveTarget::Root,
            &mut tx,
        )
        .await?;

        let page_relationships = get_page_relationships(&mut tx).await?;
        assert_eq!(
            HashSet::from([
                SimplePageRelationship(page_1.id, page_1.id, 0),
                SimplePageRelationship(page_1.id, page_1_2.id, 1),
                SimplePageRelationship(page_1.id, page_1_3.id, 1),
                SimplePageRelationship(page_2.id, page_2.id, 0),
                SimplePageRelationship(page_3.id, page_3.id, 0),
                SimplePageRelationship(page_1_1.id, page_1_1.id, 0),
                SimplePageRelationship(page_1_1.id, page_1_1_1.id, 1),
                SimplePageRelationship(page_1_2.id, page_1_2.id, 0),
                SimplePageRelationship(page_1_3.id, page_1_3.id, 0),
                SimplePageRelationship(page_1_1_1.id, page_1_1_1.id, 0)
            ]),
            page_relationships
        );
        let page_sibling_relationships = get_page_sibling_relationships(&mut tx).await?;
        assert_eq!(
            HashSet::from([
                SimplePageRelationship(page_1.id, page_1.id, 0),
                SimplePageRelationship(page_1.id, page_2.id, 1),
                SimplePageRelationship(page_1.id, page_3.id, 2),
                SimplePageRelationship(page_1.id, page_1_1.id, 3),
                SimplePageRelationship(page_2.id, page_2.id, 0),
                SimplePageRelationship(page_2.id, page_3.id, 1),
                SimplePageRelationship(page_2.id, page_1_1.id, 2),
                SimplePageRelationship(page_3.id, page_3.id, 0),
                SimplePageRelationship(page_3.id, page_1_1.id, 1),
                SimplePageRelationship(page_1_1.id, page_1_1.id, 0),
                SimplePageRelationship(page_1_2.id, page_1_2.id, 0),
                SimplePageRelationship(page_1_2.id, page_1_3.id, 1),
                SimplePageRelationship(page_1_3.id, page_1_3.id, 0),
                SimplePageRelationship(page_1_1_1.id, page_1_1_1.id, 0)
            ]),
            page_sibling_relationships
        );

        teardown(tx).await?;

        Ok(())
    }

    #[tokio::test]
    async fn move_to_non_root_should_succeed() -> anyhow::Result<()> {
        let (
            InsertMockResponse {
                page_1,
                page_2,
                page_3,
                page_1_1,
                page_1_2,
                page_1_3,
                page_1_1_1,
            },
            mut tx,
        ) = setup().await?;

        InternalPageRepository::move_(
            &page_1_1.id,
            &crate::models::page::MoveTarget::Parent(page_1.id),
            &mut tx,
        )
        .await?;

        let page_relationships = get_page_relationships(&mut tx).await?;
        assert_eq!(
            HashSet::from([
                SimplePageRelationship(page_1.id, page_1.id, 0),
                SimplePageRelationship(page_1.id, page_1_2.id, 1),
                SimplePageRelationship(page_1.id, page_1_3.id, 1),
                SimplePageRelationship(page_1.id, page_1_1.id, 1),
                SimplePageRelationship(page_1.id, page_1_1_1.id, 2),
                SimplePageRelationship(page_2.id, page_2.id, 0),
                SimplePageRelationship(page_3.id, page_3.id, 0),
                SimplePageRelationship(page_1_2.id, page_1_2.id, 0),
                SimplePageRelationship(page_1_3.id, page_1_3.id, 0),
                SimplePageRelationship(page_1_1.id, page_1_1.id, 0),
                SimplePageRelationship(page_1_1.id, page_1_1_1.id, 1),
                SimplePageRelationship(page_1_1_1.id, page_1_1_1.id, 0)
            ]),
            page_relationships
        );
        let page_sibling_relationships = get_page_sibling_relationships(&mut tx).await?;
        assert_eq!(
            HashSet::from([
                SimplePageRelationship(page_1.id, page_1.id, 0),
                SimplePageRelationship(page_1.id, page_2.id, 1),
                SimplePageRelationship(page_1.id, page_3.id, 2),
                SimplePageRelationship(page_2.id, page_2.id, 0),
                SimplePageRelationship(page_2.id, page_3.id, 1),
                SimplePageRelationship(page_3.id, page_3.id, 0),
                SimplePageRelationship(page_1_2.id, page_1_2.id, 0),
                SimplePageRelationship(page_1_2.id, page_1_3.id, 1),
                SimplePageRelationship(page_1_2.id, page_1_1.id, 2),
                SimplePageRelationship(page_1_3.id, page_1_3.id, 0),
                SimplePageRelationship(page_1_3.id, page_1_1.id, 1),
                SimplePageRelationship(page_1_1.id, page_1_1.id, 0),
                SimplePageRelationship(page_1_1_1.id, page_1_1_1.id, 0)
            ]),
            page_sibling_relationships
        );

        teardown(tx).await?;

        Ok(())
    }

    #[tokio::test]
    async fn move_to_sibling_parent_should_succeed() -> anyhow::Result<()> {
        let (
            InsertMockResponse {
                page_1,
                page_2,
                page_3,
                page_1_1,
                page_1_2,
                page_1_3,
                page_1_1_1,
            },
            mut tx,
        ) = setup().await?;

        InternalPageRepository::move_(
            &page_1_1.id,
            &crate::models::page::MoveTarget::SiblingParent(page_2.id),
            &mut tx,
        )
        .await?;

        let page_relationships = get_page_relationships(&mut tx).await?;
        assert_eq!(
            HashSet::from([
                SimplePageRelationship(page_1.id, page_1.id, 0),
                SimplePageRelationship(page_1.id, page_1_2.id, 1),
                SimplePageRelationship(page_1.id, page_1_3.id, 1),
                SimplePageRelationship(page_2.id, page_2.id, 0),
                SimplePageRelationship(page_1_1.id, page_1_1.id, 0),
                SimplePageRelationship(page_1_1.id, page_1_1_1.id, 1),
                SimplePageRelationship(page_3.id, page_3.id, 0),
                SimplePageRelationship(page_1_2.id, page_1_2.id, 0),
                SimplePageRelationship(page_1_3.id, page_1_3.id, 0),
                SimplePageRelationship(page_1_1_1.id, page_1_1_1.id, 0)
            ]),
            page_relationships
        );
        let page_sibling_relationships = get_page_sibling_relationships(&mut tx).await?;
        assert_eq!(
            HashSet::from([
                SimplePageRelationship(page_1.id, page_1.id, 0),
                SimplePageRelationship(page_1.id, page_2.id, 1),
                SimplePageRelationship(page_1.id, page_1_1.id, 2),
                SimplePageRelationship(page_1.id, page_3.id, 3),
                SimplePageRelationship(page_2.id, page_2.id, 0),
                SimplePageRelationship(page_2.id, page_1_1.id, 1),
                SimplePageRelationship(page_2.id, page_3.id, 2),
                SimplePageRelationship(page_1_1.id, page_1_1.id, 0),
                SimplePageRelationship(page_1_1.id, page_3.id, 1),
                SimplePageRelationship(page_3.id, page_3.id, 0),
                SimplePageRelationship(page_1_2.id, page_1_2.id, 0),
                SimplePageRelationship(page_1_2.id, page_1_3.id, 1),
                SimplePageRelationship(page_1_3.id, page_1_3.id, 0),
                SimplePageRelationship(page_1_1_1.id, page_1_1_1.id, 0)
            ]),
            page_sibling_relationships
        );

        teardown(tx).await?;

        Ok(())
    }

    #[tokio::test]
    async fn move_to_sibling_child_should_succeed() -> anyhow::Result<()> {
        let (
            InsertMockResponse {
                page_1,
                page_2,
                page_3,
                page_1_1,
                page_1_2,
                page_1_3,
                page_1_1_1,
            },
            mut tx,
        ) = setup().await?;

        InternalPageRepository::move_(
            &page_1_1.id,
            &crate::models::page::MoveTarget::SiblingChild(page_2.id),
            &mut tx,
        )
        .await?;

        let page_relationships = get_page_relationships(&mut tx).await?;
        assert_eq!(
            HashSet::from([
                SimplePageRelationship(page_1.id, page_1.id, 0),
                SimplePageRelationship(page_1.id, page_1_2.id, 1),
                SimplePageRelationship(page_1.id, page_1_3.id, 1),
                SimplePageRelationship(page_1_1.id, page_1_1.id, 0),
                SimplePageRelationship(page_1_1.id, page_1_1_1.id, 1),
                SimplePageRelationship(page_2.id, page_2.id, 0),
                SimplePageRelationship(page_3.id, page_3.id, 0),
                SimplePageRelationship(page_1_2.id, page_1_2.id, 0),
                SimplePageRelationship(page_1_3.id, page_1_3.id, 0),
                SimplePageRelationship(page_1_1_1.id, page_1_1_1.id, 0)
            ]),
            page_relationships
        );
        let page_sibling_relationships = get_page_sibling_relationships(&mut tx).await?;
        assert_eq!(
            HashSet::from([
                SimplePageRelationship(page_1.id, page_1.id, 0),
                SimplePageRelationship(page_1.id, page_1_1.id, 1),
                SimplePageRelationship(page_1.id, page_2.id, 2),
                SimplePageRelationship(page_1.id, page_3.id, 3),
                SimplePageRelationship(page_1_1.id, page_1_1.id, 0),
                SimplePageRelationship(page_1_1.id, page_2.id, 1),
                SimplePageRelationship(page_1_1.id, page_3.id, 2),
                SimplePageRelationship(page_2.id, page_2.id, 0),
                SimplePageRelationship(page_2.id, page_3.id, 1),
                SimplePageRelationship(page_3.id, page_3.id, 0),
                SimplePageRelationship(page_1_2.id, page_1_2.id, 0),
                SimplePageRelationship(page_1_2.id, page_1_3.id, 1),
                SimplePageRelationship(page_1_3.id, page_1_3.id, 0),
                SimplePageRelationship(page_1_1_1.id, page_1_1_1.id, 0)
            ]),
            page_sibling_relationships
        );

        teardown(tx).await?;

        Ok(())
    }

    #[tokio::test]
    async fn move_middle_to_sibling_parent_should_succeed() -> anyhow::Result<()> {
        let (
            InsertMockResponse {
                page_1,
                page_2,
                page_3,
                page_1_1,
                page_1_2,
                page_1_3,
                page_1_1_1,
            },
            mut tx,
        ) = setup().await?;

        InternalPageRepository::move_(
            &page_1_2.id,
            &crate::models::page::MoveTarget::SiblingParent(page_1_3.id),
            &mut tx,
        )
        .await?;

        let page_relationships = get_page_relationships(&mut tx).await?;
        assert_eq!(
            HashSet::from([
                SimplePageRelationship(page_1.id, page_1.id, 0),
                SimplePageRelationship(page_1.id, page_1_1.id, 1),
                SimplePageRelationship(page_1.id, page_1_3.id, 1),
                SimplePageRelationship(page_1.id, page_1_2.id, 1),
                SimplePageRelationship(page_1.id, page_1_1_1.id, 2),
                SimplePageRelationship(page_2.id, page_2.id, 0),
                SimplePageRelationship(page_3.id, page_3.id, 0),
                SimplePageRelationship(page_1_1.id, page_1_1.id, 0),
                SimplePageRelationship(page_1_1.id, page_1_1_1.id, 1),
                SimplePageRelationship(page_1_3.id, page_1_3.id, 0),
                SimplePageRelationship(page_1_2.id, page_1_2.id, 0),
                SimplePageRelationship(page_1_1_1.id, page_1_1_1.id, 0)
            ]),
            page_relationships
        );
        let page_sibling_relationships = get_page_sibling_relationships(&mut tx).await?;
        assert_eq!(
            HashSet::from([
                SimplePageRelationship(page_1.id, page_1.id, 0),
                SimplePageRelationship(page_1.id, page_2.id, 1),
                SimplePageRelationship(page_1.id, page_3.id, 2),
                SimplePageRelationship(page_2.id, page_2.id, 0),
                SimplePageRelationship(page_2.id, page_3.id, 1),
                SimplePageRelationship(page_3.id, page_3.id, 0),
                SimplePageRelationship(page_1_1.id, page_1_1.id, 0),
                SimplePageRelationship(page_1_1.id, page_1_3.id, 1),
                SimplePageRelationship(page_1_1.id, page_1_2.id, 2),
                SimplePageRelationship(page_1_3.id, page_1_3.id, 0),
                SimplePageRelationship(page_1_3.id, page_1_2.id, 1),
                SimplePageRelationship(page_1_2.id, page_1_2.id, 0),
                SimplePageRelationship(page_1_1_1.id, page_1_1_1.id, 0)
            ]),
            page_sibling_relationships
        );

        teardown(tx).await?;

        Ok(())
    }

    #[tokio::test]
    async fn move_middle_to_sibling_child_should_succeed() -> anyhow::Result<()> {
        let (
            InsertMockResponse {
                page_1,
                page_2,
                page_3,
                page_1_1,
                page_1_2,
                page_1_3,
                page_1_1_1,
            },
            mut tx,
        ) = setup().await?;

        InternalPageRepository::move_(
            &page_1_2.id,
            &crate::models::page::MoveTarget::SiblingChild(page_1_1.id),
            &mut tx,
        )
        .await?;

        let page_relationships = get_page_relationships(&mut tx).await?;
        assert_eq!(
            HashSet::from([
                SimplePageRelationship(page_1.id, page_1.id, 0),
                SimplePageRelationship(page_1.id, page_1_2.id, 1),
                SimplePageRelationship(page_1.id, page_1_1.id, 1),
                SimplePageRelationship(page_1.id, page_1_3.id, 1),
                SimplePageRelationship(page_1.id, page_1_1_1.id, 2),
                SimplePageRelationship(page_2.id, page_2.id, 0),
                SimplePageRelationship(page_3.id, page_3.id, 0),
                SimplePageRelationship(page_1_2.id, page_1_2.id, 0),
                SimplePageRelationship(page_1_1.id, page_1_1.id, 0),
                SimplePageRelationship(page_1_1.id, page_1_1_1.id, 1),
                SimplePageRelationship(page_1_3.id, page_1_3.id, 0),
                SimplePageRelationship(page_1_1_1.id, page_1_1_1.id, 0)
            ]),
            page_relationships
        );
        let page_sibling_relationships = get_page_sibling_relationships(&mut tx).await?;
        assert_eq!(
            HashSet::from([
                SimplePageRelationship(page_1.id, page_1.id, 0),
                SimplePageRelationship(page_1.id, page_2.id, 1),
                SimplePageRelationship(page_1.id, page_3.id, 2),
                SimplePageRelationship(page_2.id, page_2.id, 0),
                SimplePageRelationship(page_2.id, page_3.id, 1),
                SimplePageRelationship(page_3.id, page_3.id, 0),
                SimplePageRelationship(page_1_2.id, page_1_2.id, 0),
                SimplePageRelationship(page_1_2.id, page_1_1.id, 1),
                SimplePageRelationship(page_1_2.id, page_1_3.id, 2),
                SimplePageRelationship(page_1_1.id, page_1_1.id, 0),
                SimplePageRelationship(page_1_1.id, page_1_3.id, 1),
                SimplePageRelationship(page_1_3.id, page_1_3.id, 0),
                SimplePageRelationship(page_1_1_1.id, page_1_1_1.id, 0)
            ]),
            page_sibling_relationships
        );

        teardown(tx).await?;

        Ok(())
    }
}
