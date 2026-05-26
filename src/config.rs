/// PreventSleep.txt の1行分のルール
#[derive(Debug, Clone)]
pub enum CoordSpec {
    Cascade, // "*" → 対象モニタで階段配置
    Pixels(i32),
    Percent(f32), // "10%" など
}

#[derive(Debug, Clone)]
pub enum SizeSpec {
    Pixels(i32),
    Fill, // "*"
    Percent(f32), // "70%" など
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub title_regex: String,
    pub class_regex: String,
    pub x: CoordSpec,
    pub y: CoordSpec,
    pub w: SizeSpec,
    pub h: SizeSpec,
    pub displays: String, // 例: "12345" → 1〜5画面すべてで有効
}

fn parse_coord_spec(s: &str, default_px: i32) -> CoordSpec {
    let trimmed = s.trim();
    if trimmed == "*" {
        return CoordSpec::Cascade;
    }
    if let Some(rest) = trimmed.strip_suffix('%') {
        if let Ok(p) = rest.trim().parse::<f32>() {
            return CoordSpec::Percent((p / 100.0).max(0.0));
        }
    }
    CoordSpec::Pixels(trimmed.parse::<i32>().unwrap_or(default_px))
}

fn parse_size_spec(s: &str, default_px: i32) -> SizeSpec {
    let trimmed = s.trim();
    if trimmed == "*" {
        return SizeSpec::Fill;
    }
    if let Some(rest) = trimmed.strip_suffix('%') {
        if let Ok(p) = rest.trim().parse::<f32>() {
            return SizeSpec::Percent((p / 100.0).max(0.0));
        }
    }
    SizeSpec::Pixels(trimmed.parse::<i32>().unwrap_or(default_px))
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
        // 区切り行（### / #### など）以降は読み込み終了
        if line.starts_with("###") {
            break;
        }
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

        // "#" で始まる行はコメント扱い
        if title_regex.starts_with('#') {
            continue;
        }

        let x = parse_coord_spec(rec.get(2).unwrap_or("0"), 0);
        let y = parse_coord_spec(rec.get(3).unwrap_or("0"), 0);
        let w = parse_size_spec(rec.get(4).unwrap_or("100"), 100);
        let h = parse_size_spec(rec.get(5).unwrap_or("100"), 100);
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
