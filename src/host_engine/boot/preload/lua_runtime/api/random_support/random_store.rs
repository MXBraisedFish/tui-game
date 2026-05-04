//! 随机数生成器仓库

use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub const DEFAULT_RANDOM_MAX: i64 = 2_147_483_647;
pub const MAX_RANDOMS: usize = 64;

/// 随机数生成器仓库。
#[derive(Debug, Default)]
pub struct RandomStore {
    next_id: u64,
    randoms: BTreeMap<String, RandomEntry>,
}

/// 随机数生成器条目。
#[derive(Debug)]
pub struct RandomEntry {
    pub id: String,
    pub note: String,
    pub seed: String,
    pub step: u64,
    pub kind: RandomKind,
    rng: StdRng,
}

/// 随机数生成器类型。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RandomKind {
    Int,
    Float,
}

impl RandomKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Float => "float",
        }
    }
}

impl RandomStore {
    /// 使用宿主默认随机源生成整数。
    pub fn default_random_int(min: i64, max: i64) -> i64 {
        let mut rng = rand::thread_rng();
        rng.gen_range(min..=max)
    }

    /// 使用宿主默认随机源生成浮点数。
    pub fn default_random_float() -> f64 {
        let mut rng = rand::thread_rng();
        rng.gen_range(0.0..1.0)
    }

    /// 创建生成器。
    pub fn create_random(
        &mut self,
        seed: String,
        note: String,
        kind: RandomKind,
    ) -> mlua::Result<String> {
        if seed.trim().is_empty() {
            return Err(mlua::Error::external("random seed is empty"));
        }
        if self.randoms.len() >= MAX_RANDOMS {
            return Err(mlua::Error::external("random generator limit reached"));
        }
        self.next_id += 1;
        let id = format!("random_{}", self.next_id);
        self.randoms.insert(
            id.clone(),
            RandomEntry {
                id: id.clone(),
                note,
                seed: seed.clone(),
                step: 0,
                kind,
                rng: seeded_rng(seed.as_str()),
            },
        );
        Ok(id)
    }

    /// 删除生成器。
    pub fn kill_random(&mut self, id: &str) -> mlua::Result<()> {
        self.randoms
            .remove(id)
            .map(|_| ())
            .ok_or_else(|| random_not_found(id))
    }

    /// 获取生成器。
    pub fn random(&self, id: &str) -> mlua::Result<&RandomEntry> {
        self.randoms.get(id).ok_or_else(|| random_not_found(id))
    }

    /// 获取可变生成器。
    pub fn random_mut(&mut self, id: &str) -> mlua::Result<&mut RandomEntry> {
        self.randoms.get_mut(id).ok_or_else(|| random_not_found(id))
    }

    /// 生成器列表。
    pub fn randoms(&self) -> impl Iterator<Item = &RandomEntry> {
        self.randoms.values()
    }
}

impl RandomEntry {
    /// 生成整数。
    pub fn next_int(&mut self, min: i64, max: i64) -> i64 {
        self.step += 1;
        self.rng.gen_range(min..=max)
    }

    /// 生成浮点数。
    pub fn next_float(&mut self) -> f64 {
        self.step += 1;
        self.rng.gen_range(0.0..1.0)
    }

    /// 重置步数和 RNG。
    pub fn reset_step(&mut self) {
        self.step = 0;
        self.rng = seeded_rng(self.seed.as_str());
    }

    /// 设置备注。
    pub fn set_note(&mut self, note: String) {
        self.note = note;
    }
}

fn seeded_rng(seed: &str) -> StdRng {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    StdRng::seed_from_u64(hasher.finish())
}

fn random_not_found(id: &str) -> mlua::Error {
    mlua::Error::external(format!("random generator not found: {id}"))
}
