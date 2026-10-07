use crate::common::DateTimeUtc;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

define_id!(PageId);

#[derive(Debug)]
pub struct Page {
    pub id: PageId,
    pub title: String,
    pub text: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Debug, Default)]
pub struct AddPage {
    /// 渡された場合はその値を使い、`None`の場合はDBの`DEFAULT`に任せる
    pub id: Option<PageId>,
    pub title: String,
    pub text: String,
}

#[derive(Debug, Default)]
pub struct UpdatePage {
    pub title: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct PageRelationship {
    pub ancestor: PageId,
    pub descendant: PageId,
    // TODO: usize
    pub weight: i32,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SimplePageRelationship(pub PageId, pub PageId, pub i32);

impl From<PageRelationship> for SimplePageRelationship {
    fn from(value: PageRelationship) -> Self {
        Self(value.ancestor, value.descendant, value.weight)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PageTree {
    pub id: PageId,
    pub title: String,
    pub text: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub children: Vec<PageTree>,
}

impl PageTree {
    pub fn build(
        pages: Vec<Page>,
        parent_child_relationships: &[PageRelationship],
        root_id: &PageId,
    ) -> PageTree {
        let page_tree_map: HashMap<PageId, Rc<RefCell<MutablePageTree>>> =
            pages.into_iter().map(|p| (p.id, p.into())).collect();

        parent_child_relationships.iter().for_each(|r| {
            assert_eq!(1, r.weight);

            let parent = page_tree_map.get(&r.ancestor).unwrap();
            let child = page_tree_map.get(&r.descendant).unwrap();
            parent.borrow_mut().children.push(Rc::clone(child));
        });

        Rc::clone(page_tree_map.get(root_id).unwrap()).into()
    }
}

#[derive(Debug)]
struct MutablePageTree {
    id: PageId,
    title: String,
    text: String,
    created_at: DateTimeUtc,
    updated_at: DateTimeUtc,
    children: Vec<Rc<RefCell<MutablePageTree>>>,
}

impl From<Page> for Rc<RefCell<MutablePageTree>> {
    fn from(value: Page) -> Self {
        Rc::new(RefCell::new(MutablePageTree {
            id: value.id,
            title: value.title,
            text: value.text,
            created_at: value.created_at,
            updated_at: value.updated_at,
            children: Vec::new(),
        }))
    }
}

impl From<Rc<RefCell<MutablePageTree>>> for PageTree {
    fn from(value: Rc<RefCell<MutablePageTree>>) -> Self {
        Self {
            id: value.borrow().id,
            title: value.borrow().title.clone(),
            text: value.borrow().text.clone(),
            created_at: value.borrow().created_at,
            updated_at: value.borrow().updated_at,
            children: value
                .borrow()
                .children
                .iter()
                .map(|c| Self::from(Rc::clone(c)))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_tree_build_should_succeed() {
        let page_1_id = PageId::new();
        let page_1_1_id = PageId::new();
        let page_1_2_id = PageId::new();
        let page_1_3_id = PageId::new();
        let page_1_1_1_id = PageId::new();
        let date_time_utc = DateTimeUtc::new();
        let pages = vec![
            Page {
                id: page_1_id,
                title: String::new(),
                text: String::new(),
                created_at: date_time_utc,
                updated_at: date_time_utc,
            },
            Page {
                id: page_1_1_id,
                title: String::new(),
                text: String::new(),
                created_at: date_time_utc,
                updated_at: date_time_utc,
            },
            Page {
                id: page_1_2_id,
                title: String::new(),
                text: String::new(),
                created_at: date_time_utc,
                updated_at: date_time_utc,
            },
            Page {
                id: page_1_3_id,
                title: String::new(),
                text: String::new(),
                created_at: date_time_utc,
                updated_at: date_time_utc,
            },
            Page {
                id: page_1_1_1_id,
                title: String::new(),
                text: String::new(),
                created_at: date_time_utc,
                updated_at: date_time_utc,
            },
        ];
        let parent_child_relationships = vec![
            PageRelationship {
                ancestor: page_1_id,
                descendant: page_1_1_id,
                weight: 1,
            },
            PageRelationship {
                ancestor: page_1_id,
                descendant: page_1_2_id,
                weight: 1,
            },
            PageRelationship {
                ancestor: page_1_id,
                descendant: page_1_3_id,
                weight: 1,
            },
            PageRelationship {
                ancestor: page_1_1_id,
                descendant: page_1_1_1_id,
                weight: 1,
            },
        ];

        let expected = PageTree {
            id: page_1_id,
            title: String::new(),
            text: String::new(),
            created_at: date_time_utc,
            updated_at: date_time_utc,
            children: vec![
                PageTree {
                    id: page_1_1_id,
                    title: String::new(),
                    text: String::new(),
                    created_at: date_time_utc,
                    updated_at: date_time_utc,
                    children: vec![PageTree {
                        id: page_1_1_1_id,
                        title: String::new(),
                        text: String::new(),
                        created_at: date_time_utc,
                        updated_at: date_time_utc,
                        children: Vec::new(),
                    }],
                },
                PageTree {
                    id: page_1_2_id,
                    title: String::new(),
                    text: String::new(),
                    created_at: date_time_utc,
                    updated_at: date_time_utc,
                    children: Vec::new(),
                },
                PageTree {
                    id: page_1_3_id,
                    title: String::new(),
                    text: String::new(),
                    created_at: date_time_utc,
                    updated_at: date_time_utc,
                    children: Vec::new(),
                },
            ],
        };
        let actual = PageTree::build(pages, &parent_child_relationships, &page_1_id);
        assert_eq!(expected, actual);
    }
}

pub enum MoveTarget {
    Root,
    Parent(PageId),
    SiblingParent(PageId),
    SiblingChild(PageId),
}
