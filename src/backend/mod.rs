use crate::RespFrame;
use dashmap::{DashMap, DashSet};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Backend(Arc<BackendInner>);

#[derive(Debug)]
pub struct BackendInner {
    pub(crate) map: DashMap<String, RespFrame>,
    pub(crate) hmap: DashMap<String, DashMap<String, RespFrame>>,
    pub(crate) set: DashMap<String, DashSet<String>>, //  DashSet 中的元素要求实现 Eq, RespFrame 不能实现 Eq, 因此这里使用 String
}

impl Backend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<RespFrame> {
        self.map.get(key).map(|v| v.value().clone())
    }

    pub fn set(&self, key: String, value: RespFrame) {
        self.map.insert(key, value);
    }

    pub fn hget(&self, key: &str, field: &str) -> Option<RespFrame> {
        self.hmap
            .get(key)
            .and_then(|v| v.get(field).map(|v| v.value().clone()))
    }

    pub fn hset(&self, key: String, field: String, value: RespFrame) {
        let hmap = self.hmap.entry(key).or_default();
        hmap.insert(field, value);
    }

    pub fn hgetall(&self, key: &str) -> Option<DashMap<String, RespFrame>> {
        self.hmap.get(key).map(|v| v.clone())
    }

    pub fn sadd(&self, key: String, members: impl Into<Vec<String>>) -> i64 {
        let set = self.set.entry(key).or_default();
        let mut cnt = 0;
        for member in members.into() {
            if set.insert(member) {
                cnt += 1;
            }
        }
        cnt
    }

    pub fn sismember(&self, key: &str, value: &str) -> bool {
        self.set
            .get(key)
            .and_then(|v| v.get(value).map(|_| true))
            .unwrap_or(false)
    }
    pub fn insert_set(&self, key: String, values: Vec<String>) {
        let set = self.set.get_mut(&key);
        match set {
            Some(set) => {
                for value in values {
                    (*set).insert(value);
                }
            }
            None => {
                let new_set = DashSet::new();
                for value in values {
                    new_set.insert(value);
                }
                self.set.insert(key.to_string(), new_set);
            }
        }
    }
}

impl Deref for Backend {
    type Target = BackendInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for BackendInner {
    fn default() -> Self {
        Self {
            map: DashMap::new(),
            hmap: DashMap::new(),
            set: DashMap::new(),
        }
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self(Arc::new(BackendInner::default()))
    }
}
