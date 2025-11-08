use std::collections::HashMap;

/// ファイルインデックス: パス → (mtime, size, hash, deps) のマッピング
/// depsは (パス, sha256ハッシュ) のペアのベクター
#[derive(Debug, Clone)]
pub struct FileIndex {
    inner: HashMap<String, (u64, u64, String, Vec<(String, String)>)>,
}

impl FileIndex {
    /// 空のFileIndexを作成する
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// 要素数を返す
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// 指定されたパスのメタデータを取得する
    pub fn get(&self, path: &str) -> Option<&(u64, u64, String, Vec<(String, String)>)> {
        self.inner.get(path)
    }

    /// パスとメタデータを挿入する
    pub fn insert(&mut self, path: String, metadata: (u64, u64, String, Vec<(String, String)>)) -> Option<(u64, u64, String, Vec<(String, String)>)> {
        self.inner.insert(path, metadata)
    }

    /// イテレータを返す
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, (u64, u64, String, Vec<(String, String)>)> {
        self.inner.iter()
    }

    /// 内部のHashMapを取得する（storage.rsで使用）
    pub fn into_inner(self) -> HashMap<String, (u64, u64, String, Vec<(String, String)>)> {
        self.inner
    }

    /// 内部のHashMapへの参照を取得する（storage.rsで使用）
    pub fn as_inner(&self) -> &HashMap<String, (u64, u64, String, Vec<(String, String)>)> {
        &self.inner
    }

    /// HashMapからFileIndexを作成する
    pub fn from_hashmap(map: HashMap<String, (u64, u64, String, Vec<(String, String)>)>) -> Self {
        Self { inner: map }
    }
}

impl Default for FileIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for FileIndex {
    type Item = (String, (u64, u64, String, Vec<(String, String)>));
    type IntoIter = std::collections::hash_map::IntoIter<String, (u64, u64, String, Vec<(String, String)>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a FileIndex {
    type Item = (&'a String, &'a (u64, u64, String, Vec<(String, String)>));
    type IntoIter = std::collections::hash_map::Iter<'a, String, (u64, u64, String, Vec<(String, String)>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl FromIterator<(String, (u64, u64, String, Vec<(String, String)>))> for FileIndex {
    fn from_iter<T: IntoIterator<Item = (String, (u64, u64, String, Vec<(String, String)>))>>(iter: T) -> Self {
        Self {
            inner: HashMap::from_iter(iter),
        }
    }
}

