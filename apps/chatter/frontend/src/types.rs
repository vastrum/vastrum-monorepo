use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub struct GroupMember {
    pub id: usize,
    pub name: String,
    pub avatar: String,
    pub role: String,
    pub status: String,
}

pub type GroupMemberMap = HashMap<usize, Vec<GroupMember>>;
