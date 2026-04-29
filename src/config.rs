/// PreventSleep.txt の1行分のルール
#[derive(Debug, Clone)]
pub struct Rule {
    pub title_regex: String,
    pub class_regex: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub displays: String, // 例: "12345" → 1〜5画面すべてで有効
}

/// PreventSleep.txt を読み込んでルールのリストを返す。
/// ファイルが存在しない / 読み取れない場合は空リストを返す。
pub fn load_rules(path: &str) -> Vec<Rule> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut rules = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        // コメント行・空行はスキップ
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let cols: Vec<&str> = line.splitn(8, ',').collect();
        if cols.len() < 6 {
            continue;
        }

        // 最初のフィールドが "####" または "#" で始まる行はコメント扱い
        if cols[0].starts_with("####") || cols[0].starts_with('#') {
            continue;
        }

        let x = cols[2].trim().parse::<i32>().unwrap_or(0);
        let y = cols[3].trim().parse::<i32>().unwrap_or(0);
        let w = cols[4].trim().parse::<i32>().unwrap_or(100);
        let h = cols[5].trim().parse::<i32>().unwrap_or(100);
        let displays = if cols.len() > 6 {
            cols[6].trim().to_string()
        } else {
            "12345".to_string()
        };

        rules.push(Rule {
            title_regex: cols[0].trim().to_string(),
            class_regex: cols[1].trim().to_string(),
            x,
            y,
            w,
            h,
            displays,
        });
    }
    rules
}
