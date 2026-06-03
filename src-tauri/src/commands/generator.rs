use rand::Rng;
use serde::Deserialize;
use tauri::State;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct PasswordOptions {
    pub length: usize,
    pub uppercase: bool,
    pub lowercase: bool,
    pub numbers: bool,
    pub symbols: bool,
    #[serde(rename = "excludeAmbiguous")]
    pub exclude_ambiguous: bool,
    pub mode: Option<String>,
}

const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const NUMBERS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";
const AMBIGUOUS_UPPER: &str = "IO";
const AMBIGUOUS_LOWER: &str = "l";
const AMBIGUOUS_NUMS: &str = "01";

/// Diceware 词表（简化版，32 个常用词）
const DICEWARE_WORDS: &[&str] = &[
    "correct", "horse", "battery", "staple", "alpha", "bravo", "charlie", "delta",
    "echo", "foxtrot", "golf", "hotel", "india", "juliet", "kilo", "lima",
    "mike", "november", "oscar", "papa", "quebec", "romeo", "sierra", "tango",
    "uniform", "victor", "whiskey", "xray", "yankee", "zulu", "ocean", "mountain",
];

#[tauri::command]
pub async fn generate_password(
    session_token: String,
    options: PasswordOptions,
    state: State<'_, AppState>,
) -> Result<String, String> {
    if !state.sessions.validate(&session_token).await {
        return Err("会话已过期".to_string());
    }

    if options.mode.as_deref() == Some("diceware") {
        return generate_diceware(options.length.max(4));
    }

    let mut charset = String::new();
    let mut required_sets: Vec<String> = Vec::new();

    if options.uppercase {
        let mut chars = UPPERCASE.to_string();
        if options.exclude_ambiguous {
            chars = chars.replace(AMBIGUOUS_UPPER, "");
        }
        charset.push_str(&chars);
        required_sets.push(chars);
    }
    if options.lowercase {
        let mut chars = LOWERCASE.to_string();
        if options.exclude_ambiguous {
            chars = chars.replace(AMBIGUOUS_LOWER, "");
        }
        charset.push_str(&chars);
        required_sets.push(chars);
    }
    if options.numbers {
        let mut chars = NUMBERS.to_string();
        if options.exclude_ambiguous {
            chars = chars.replace(AMBIGUOUS_NUMS, "");
        }
        charset.push_str(&chars);
        required_sets.push(chars);
    }
    if options.symbols {
        charset.push_str(SYMBOLS);
        required_sets.push(SYMBOLS.to_string());
    }

    if charset.is_empty() {
        charset = LOWERCASE.to_string();
        required_sets.push(LOWERCASE.to_string());
    }

    let len = options.length.max(required_sets.len());
    let mut rng = rand::thread_rng();
    let mut password = String::with_capacity(len);

    // 确保每种选中的字符类型至少出现一次
    for set in &required_sets {
        if password.len() < len {
            let chars: Vec<char> = set.chars().collect();
            password.push(chars[rng.gen_range(0..chars.len())]);
        }
    }

    // 填充剩余长度
    let chars: Vec<char> = charset.chars().collect();
    while password.len() < len {
        password.push(chars[rng.gen_range(0..chars.len())]);
    }

    // Fisher-Yates 洗牌
    let mut bytes: Vec<char> = password.chars().collect();
    for i in (1..bytes.len()).rev() {
        let j = rng.gen_range(0..=i);
        bytes.swap(i, j);
    }

    Ok(bytes.into_iter().collect())
}

fn generate_diceware(word_count: usize) -> Result<String, String> {
    let mut rng = rand::thread_rng();
    let words: Vec<&str> = (0..word_count)
        .map(|_| DICEWARE_WORDS[rng.gen_range(0..DICEWARE_WORDS.len())])
        .collect();
    Ok(words.join("-"))
}
