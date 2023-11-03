//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "node")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub node_id: i64,
    pub git_id: String,
    pub last_commit: String,
    pub node_type: String,
    pub name: Option<String>,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub mode: Vec<u8>,
    pub content_sha: Option<String>,
    pub size: i32,
    pub repo_path: String,
    pub full_path: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

use std::cmp::Ordering;

impl PartialOrd for Model {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Model {
    fn cmp(&self, other: &Self) -> Ordering {
        let node_type_order = match (self.node_type.as_str(), other.node_type.as_str()) {
            ("commit", "commit") | ("tree", "tree") | ("blob", "blob") | ("tag", "tag") => {
                Ordering::Equal
            }
            ("commit", _) => Ordering::Less,
            ("tree", "commit") => Ordering::Greater,
            ("tree", _) => Ordering::Less,
            ("blob", "commit") | ("blob", "tree") => Ordering::Greater,
            ("blob", _) => Ordering::Less,
            ("tag", _) => Ordering::Greater,
            (&_, _) => panic!("unknow types in ordering git nodes ")
            
        };

        if node_type_order != Ordering::Equal {
            node_type_order
        } else {
            let full_path_order = self.full_path.cmp(&other.full_path);
            if full_path_order != Ordering::Equal {
                full_path_order
            } else {
                self.size.cmp(&other.size)
            }
        }
    }
}

#[cfg(test)]
mod tests{
   
    use super::Model;
    #[test]
    fn test_nodes_sort(){
         // 示例使用
    let model1 = Model {
        id: 1,
        node_id: 1,
        git_id: "id1".to_string(),
        node_type: "blob".to_string(),
        name: Some("Name1".to_string()),
        mode: vec![],
        content_sha: None,
        size: 10,
        repo_path: "path1".to_string(),
        full_path: "fullpath1".to_string(),
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
        last_commit: "no commit".to_string(),
    };

    let model2 = Model {
        id: 2,
        node_id: 2,
        git_id: "id2".to_string(),
        node_type: "tree".to_string(),
        name: Some("Name2".to_string()),
        mode: vec![],
        content_sha: None,
        size: 20,
        repo_path: "path2".to_string(),
        full_path: "fullpath2".to_string(),
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
        last_commit: "no commit".to_string(),
    };
    let model3 = Model {
        id: 3,
        node_id: 3,
        git_id: "id2".to_string(),
        node_type: "tree".to_string(),
        name: Some("Name2".to_string()),
        mode: vec![],
        content_sha: None,
        size: 30,
        repo_path: "path2".to_string(),
        full_path: "fullpath1".to_string(),
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
        last_commit: "no commit".to_string(),
    };
    let model4 = Model {
        id: 4,
        node_id: 4,
        git_id: "id2".to_string(),
        node_type: "tree".to_string(),
        name: Some("Name2".to_string()),
        mode: vec![],
        content_sha: None,
        size: 40,
        repo_path: "path2".to_string(),
        full_path: "fullpath2".to_string(),
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
        last_commit: "no commit".to_string(),
    };

    let mut nodes = vec![model1,model2,model3,model4];
    nodes.sort();
    print!("{:?}", nodes);
    }
}