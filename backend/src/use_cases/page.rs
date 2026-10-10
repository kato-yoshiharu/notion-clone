use super::error::UseCaseError;
use crate::repositories::interfaces::page::IPageRepository;
use std::sync::Arc;

pub struct PageUseCase {
    page_repository: Arc<dyn IPageRepository>,
}

impl PageUseCase {
    pub fn new(page_repository: Arc<dyn IPageRepository>) -> Self {
        Self { page_repository }
    }

    pub async fn list_roots(&self) -> Result<Vec<crate::models::page::Page>, UseCaseError> {
        let pages = self.page_repository.find_roots().await?;

        Ok(pages)
    }

    pub async fn list_children(
        &self,
        id: &crate::models::page::PageId,
    ) -> Result<Vec<crate::models::page::Page>, UseCaseError> {
        let pages = self.page_repository.find_children(id).await?;

        Ok(pages)
    }

    pub async fn list_ancestors(
        &self,
        id: &crate::models::page::PageId,
    ) -> Result<Vec<crate::models::page::Page>, UseCaseError> {
        let pages = self.page_repository.find_ancestors(id).await?;

        Ok(pages)
    }

    pub async fn list_descendants(
        &self,
        id: &crate::models::page::PageId,
    ) -> Result<crate::models::page::PageTree, UseCaseError> {
        let (pages, parent_child_relationships) = self.page_repository.find_descendants(id).await?;
        let page_tree =
            crate::models::page::PageTree::build(pages, &parent_child_relationships, id);

        Ok(page_tree)
    }

    pub async fn get(
        &self,
        id: &crate::models::page::PageId,
    ) -> Result<crate::models::page::Page, UseCaseError> {
        let page = self.page_repository.find_by_id(id).await?;

        Ok(page)
    }

    pub async fn add(
        &self,
        parent_id: &Option<crate::models::page::PageId>,
        add_page: crate::models::page::AddPage,
    ) -> Result<crate::models::page::Page, UseCaseError> {
        let page = self.page_repository.add(parent_id, add_page).await?;

        Ok(page)
    }

    pub async fn update(
        &self,
        id: &crate::models::page::PageId,
        update_page: crate::models::page::UpdatePage,
    ) -> Result<crate::models::page::Page, UseCaseError> {
        let page = self.page_repository.update(id, update_page).await?;

        Ok(page)
    }

    pub async fn remove(&self, id: &crate::models::page::PageId) -> Result<(), UseCaseError> {
        self.page_repository.remove(id).await?;

        Ok(())
    }

    pub async fn move_(
        &self,
        id: &crate::models::page::PageId,
        target: &crate::models::page::MoveTarget,
    ) -> Result<(), UseCaseError> {
        self.page_repository.move_(id, target).await?;

        Ok(())
    }
}
