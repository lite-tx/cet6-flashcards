use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// 单词、翻译、短语的结构保持不变
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Word {
    pub word: String,
    #[serde(default)]
    pub translations: Vec<Translation>,
    #[serde(default)]
    pub phrases: Vec<Phrase>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Translation {
    pub translation: String,
    #[serde(rename = "type", default)]
    pub word_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Phrase {
    pub phrase: String,
    pub translation: String,
}

// 单词的统计和状态信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WordStats {
    // 核心统计数据
    pub consecutive_correct_answers: u32,   // 连续答对次数
    pub last_review_timestamp: i64,         // 上次复习时间 (Unix timestamp, milliseconds)

    // 历史统计（不衰减）
    pub total_reviews: u32,                 // 总复习次数
    pub total_correct: u32,                 // 总答对次数

    // 周期内的临时状态 (每周期重置)
    pub cycle_reviewed: bool,               // 本周期是否已复习
    pub cycle_first_answer_correct: Option<bool>, // 本周期首次回答是否正确
    pub cycle_attempts: u32,                // 本周期尝试次数
}

impl Default for WordStats {
    fn default() -> Self {
        Self {
            consecutive_correct_answers: 0,
            last_review_timestamp: 0,
            total_reviews: 0,
            total_correct: 0,
            cycle_reviewed: false,
            cycle_first_answer_correct: None,
            cycle_attempts: 0,
        }
    }
}

// 整个应用的统一状态管理
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppState {
    // --- 核心数据 ---
    #[serde(skip)] // 从JSON加载，不存入LocalStorage
    pub all_words: Vec<Word>,                       // 全局词库

    // --- 静态词库 (持久化) ---
    pub mastered_words: Vec<String>,                // 已掌握库 (存储单词的String)
    pub difficult_words: Vec<String>,               // 难词库 (存储单词的String)
    pub word_stats: HashMap<String, WordStats>,     // 单词统计信息 (Key是单词的String)

    // --- 动态词库与状态 (部分持久化) ---
    pub dynamic_review_pool: Vec<String>,           // 动态复习库（当前正在复习的单词）
    pub cache_pool: Vec<String>,                    // 缓存库 (上一周期的只读快照)
    pub review_pool_target_size: usize,             // 复习库目标大小 (用于周期结算调整)

    // --- 导航索引 (持久化) ---
    pub new_words_index: usize,                     // 生词库当前索引
    pub dynamic_review_index: usize,                // 动态复习库当前索引

    // --- 系统状态 (持久化) ---
    pub is_locked: bool,                            // 状态锁 (用于结算等关键操作)
    pub last_cycle_timestamp: i64,                  // 上次周期结算时间
    pub cycle_stats: CycleStats,                    // 当前周期统计
}

// 周期统计信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CycleStats {
    pub total_words: usize,          // 本周期总单词数
    pub reviewed_words: usize,       // 已复习单词数
    pub correct_count: usize,        // 首次答对数
    pub accuracy_rate: f32,          // 正确率
}

impl Default for CycleStats {
    fn default() -> Self {
        Self {
            total_words: 0,
            reviewed_words: 0,
            correct_count: 0,
            accuracy_rate: 0.0,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            all_words: Vec::new(),
            mastered_words: Vec::new(),
            difficult_words: Vec::new(),
            word_stats: HashMap::new(),
            dynamic_review_pool: Vec::new(),
            cache_pool: Vec::new(),
            review_pool_target_size: 20, // 默认复习池大小
            new_words_index: 0,
            dynamic_review_index: 0,
            is_locked: false,
            last_cycle_timestamp: 0,
            cycle_stats: CycleStats::default(),
        }
    }
}

// L2D 消息类型
#[derive(Debug, Clone, PartialEq)]
pub enum L2DMessage {
    Success(String),        // 答对消息
    Error(String),         // 答错消息
    Flow(String),          // 流动通知
    Quiz(String),          // 随机问答
    System(String),        // 系统消息
}

impl L2DMessage {
    pub fn priority(&self) -> u32 {
        match self {
            L2DMessage::Success(_) => 3,
            L2DMessage::Error(_) => 3,
            L2DMessage::Flow(_) => 2,
            L2DMessage::System(_) => 2,
            L2DMessage::Quiz(_) => 1,
        }
    }
}