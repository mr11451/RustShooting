use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub const RANKING_COUNT: usize = 10;
pub const RANKING_NAME_LEN: usize = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RankingEntry {
    pub name: [u8; RANKING_NAME_LEN],
    pub score: u32,
}

impl RankingEntry {
    pub const fn new(name: [u8; RANKING_NAME_LEN], score: u32) -> Self {
        Self { name, score }
    }

    pub fn name_str(&self) -> &str {
        std::str::from_utf8(&self.name).unwrap_or("???")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RankingTable {
    pub entries: [RankingEntry; RANKING_COUNT],
}

impl Default for RankingTable {
    fn default() -> Self {
        Self {
            entries: [
                RankingEntry::new(*b"AAA", 10_000),
                RankingEntry::new(*b"BBB", 9_000),
                RankingEntry::new(*b"CCC", 8_000),
                RankingEntry::new(*b"DDD", 7_000),
                RankingEntry::new(*b"EEE", 6_000),
                RankingEntry::new(*b"FFF", 5_000),
                RankingEntry::new(*b"GGG", 4_000),
                RankingEntry::new(*b"HHH", 3_000),
                RankingEntry::new(*b"III", 2_000),
                RankingEntry::new(*b"JJJ", 1_000),
            ],
        }
    }
}

impl RankingTable {
    pub fn is_high_score(&self, score: u32) -> bool {
        if score == 0 {
            return false;
        }
        self.entries
            .last()
            .is_none_or(|lowest| score > lowest.score)
    }

    #[allow(dead_code)]
    pub fn qualifies_for_ranking(&self, score: u32) -> bool {
        self.is_high_score(score)
    }

    pub fn insert(&mut self, name: [u8; RANKING_NAME_LEN], score: u32) -> Option<usize> {
        if !self.is_high_score(score) {
            return None;
        }

        let mut insert_pos = RANKING_COUNT;
        for (i, entry) in self.entries.iter().enumerate() {
            if score > entry.score {
                insert_pos = i;
                break;
            }
        }

        if insert_pos < RANKING_COUNT {
            for i in (insert_pos + 1..RANKING_COUNT).rev() {
                self.entries[i] = self.entries[i - 1];
            }
            self.entries[insert_pos] = RankingEntry::new(name, score);
            Some(insert_pos)
        } else {
            None
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + 1 + RANKING_COUNT * (RANKING_NAME_LEN + 4));
        bytes.extend_from_slice(b"RSHK");
        bytes.push(1); // format version
        for entry in &self.entries {
            bytes.extend_from_slice(&entry.name);
            bytes.extend_from_slice(&entry.score.to_le_bytes());
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 5 || &bytes[0..4] != b"RSHK" || bytes[4] != 1 {
            return None;
        }
        let expected_len = 5 + RANKING_COUNT * (RANKING_NAME_LEN + 4);
        if bytes.len() < expected_len {
            return None;
        }

        let mut entries = [RankingEntry::new(*b"   ", 0); RANKING_COUNT];
        let mut offset = 5;
        for entry in &mut entries {
            let mut name = [b' '; RANKING_NAME_LEN];
            name.copy_from_slice(&bytes[offset..offset + RANKING_NAME_LEN]);
            offset += RANKING_NAME_LEN;
            let score = u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?);
            offset += 4;
            *entry = RankingEntry::new(name, score);
        }
        Some(Self { entries })
    }

    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, self.to_bytes())
    }

    pub fn load_from_file(path: &Path) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        Self::from_bytes(&bytes).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "invalid ranking file format")
        })
    }

    pub fn load_or_default(path: &Path) -> Self {
        Self::load_from_file(path).unwrap_or_default()
    }

    pub fn save_default(&self) -> io::Result<()> {
        let path = default_save_path();
        self.save_to_file(&path)
    }

    pub fn load_default() -> Self {
        let path = default_save_path();
        Self::load_or_default(&path)
    }
}

#[cfg(target_os = "windows")]
pub fn default_save_path() -> PathBuf {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        PathBuf::from(appdata)
            .join("RustShooting")
            .join("ranking.bin")
    } else {
        PathBuf::from("data/ranking.bin")
    }
}

#[cfg(not(target_os = "windows"))]
pub fn default_save_path() -> PathBuf {
    if let Some(data_home) = std::env::var_os("XDG_DATA_HOME") {
        PathBuf::from(data_home)
            .join("rust_shooting")
            .join("ranking.bin")
    } else if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("rust_shooting")
            .join("ranking.bin")
    } else {
        PathBuf::from("data/ranking.bin")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ranking_is_ordered_and_has_ten_entries() {
        let table = RankingTable::default();
        assert_eq!(table.entries.len(), 10);
        for i in 0..9 {
            assert!(table.entries[i].score > table.entries[i + 1].score);
        }
    }

    #[test]
    fn high_score_qualification() {
        let table = RankingTable::default();
        assert!(table.is_high_score(15_000));
        assert!(table.is_high_score(1_500));
        assert!(!table.is_high_score(1_000));
        assert!(!table.is_high_score(500));
        assert!(!table.is_high_score(0));
    }

    #[test]
    fn insert_high_score_shifts_entries() {
        let mut table = RankingTable::default();
        let rank = table.insert(*b"NEW", 9_500);
        assert_eq!(rank, Some(1));
        assert_eq!(table.entries[1].name, *b"NEW");
        assert_eq!(table.entries[1].score, 9_500);
        assert_eq!(table.entries[2].name, *b"BBB");
        assert_eq!(table.entries[2].score, 9_000);
        assert_eq!(table.entries[9].name, *b"III");
        assert_eq!(table.entries[9].score, 2_000);
    }

    #[test]
    fn serialization_roundtrip() {
        let mut table = RankingTable::default();
        table.insert(*b"TOP", 99_999);
        let bytes = table.to_bytes();
        let loaded = RankingTable::from_bytes(&bytes).expect("deserialize failed");
        assert_eq!(table, loaded);
    }

    #[test]
    fn file_save_and_load_roundtrip() {
        let temp_dir = std::env::temp_dir().join("rust_shooting_test");
        let path = temp_dir.join("test_ranking.bin");
        let mut table = RankingTable::default();
        table.insert(*b"SAV", 88_888);
        table.save_to_file(&path).expect("save should succeed");
        let loaded = RankingTable::load_from_file(&path).expect("load should succeed");
        assert_eq!(table, loaded);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&temp_dir);
    }
}
