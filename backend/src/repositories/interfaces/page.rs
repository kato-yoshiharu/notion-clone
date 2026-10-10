use super::super::error::RepositoryError;
use async_trait::async_trait;

#[async_trait]
pub trait IPageRepository: Send + Sync {
    async fn find_roots(&self) -> Result<Vec<crate::models::page::Page>, RepositoryError>;

    async fn find_children(
        &self,
        id: &crate::models::page::PageId,
    ) -> Result<Vec<crate::models::page::Page>, RepositoryError>;

    async fn find_ancestors(
        &self,
        id: &crate::models::page::PageId,
    ) -> Result<Vec<crate::models::page::Page>, RepositoryError>;

    async fn find_descendants(
        &self,
        id: &crate::models::page::PageId,
    ) -> Result<
        (
            Vec<crate::models::page::Page>,
            Vec<crate::models::page::PageRelationship>,
        ),
        RepositoryError,
    >;

    async fn find_by_id(
        &self,
        id: &crate::models::page::PageId,
    ) -> Result<crate::models::page::Page, RepositoryError>;

    async fn add(
        &self,
        parent_id: &Option<crate::models::page::PageId>,
        add_page: crate::models::page::AddPage,
    ) -> Result<crate::models::page::Page, RepositoryError>;

    async fn update(
        &self,
        id: &crate::models::page::PageId,
        update_page: crate::models::page::UpdatePage,
    ) -> Result<crate::models::page::Page, RepositoryError>;

    async fn remove(&self, id: &crate::models::page::PageId) -> Result<(), RepositoryError>;

    async fn move_(
        &self,
        id: &crate::models::page::PageId,
        target: &crate::models::page::MoveTarget,
    ) -> Result<(), RepositoryError>;
}
