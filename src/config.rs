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
pub fn resolve_rules_path(path: &str) -> std::path::PathBuf {
    let path_buf = std::path::PathBuf::from(path);
    if path_buf.is_absolute() {
        return path_buf;
    }

    match std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(&path_buf)))
    {
        Some(p) => p,
        None => path_buf,
    }
}

pub fn load_rules(path: &str) -> Vec<Rule> {
    let resolved_path = resolve_rules_path(path);

    let content = match std::fs::read_to_string(&resolved_path) {
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

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_reader(line.as_bytes());

        let rec = match rdr.records().next() {
            Some(Ok(r)) => r,
            _ => continue,
        };

        if rec.len() < 6 {
            continue;
        }

        let title_regex = rec.get(0).unwrap_or("").trim().to_string();
        let class_regex = rec.get(1).unwrap_or("").trim().to_string();

        // 最初のフィールドが "####" または "#" で始まる行はコメント扱い
        if title_regex.starts_with("####") || title_regex.starts_with('#') {
            continue;
        }

        let x = rec.get(2).unwrap_or("0").trim().parse::<i32>().unwrap_or(0);
        let y = rec.get(3).unwrap_or("0").trim().parse::<i32>().unwrap_or(0);
        let w = rec.get(4).unwrap_or("100").trim().parse::<i32>().unwrap_or(100);
        let h = rec.get(5).unwrap_or("100").trim().parse::<i32>().unwrap_or(100);
        let displays = rec
            .get(6)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or("12345")
            .to_string();

        rules.push(Rule {
            title_regex,
            class_regex,
            x,
            y,
            w,
            h,
            displays,
        });
    }
    rules
}
