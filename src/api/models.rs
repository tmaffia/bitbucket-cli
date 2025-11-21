use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PaginatedResponse<T> {
    pub size: Option<u32>,
    pub page: Option<u32>,
    pub pagelen: Option<u32>,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub values: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub id: u32,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub created_on: String,
    pub updated_on: String,
    pub author: User,
    pub source: Source,
    pub destination: Source,
    pub links: Links,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub display_name: String,
    pub uuid: String,
    pub nickname: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Source {
    pub branch: Branch,
    pub repository: Repository,
}

#[derive(Debug, Deserialize)]
pub struct Branch {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub name: String,
    pub full_name: String,
    pub uuid: String,
}

#[derive(Debug, Deserialize)]
pub struct Links {
    pub html: Link,
}

#[derive(Debug, Deserialize)]
pub struct Comment {
    pub id: u32,
    pub content: Content,
    pub created_on: String,
    pub user: User,
    pub inline: Option<InlineContext>,
}

#[derive(Debug, Deserialize)]
pub struct Content {
    pub raw: String,
    pub html: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InlineContext {
    pub path: String,
    pub from: Option<u32>,
    pub to: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct Link {
    pub href: String,
}
