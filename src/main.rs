use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: syncStrings <original.strings> <lproj_folder>");
        std::process::exit(1);
    }

    let original_path = &args[1];
    let lproj_folder = &args[2];

    let original_content = fs::read_to_string(original_path)?;
    let original_entries = parse_strings_with_order(&original_content);

    // Исправлено: клонируем строки → HashSet<String>
    let original_keys: HashSet<String> = original_entries.iter().map(|e| e.key.clone()).collect();

    println!("Scanning folder: {}", lproj_folder);

    let strings_files = find_strings_files(Path::new(lproj_folder));

    if strings_files.is_empty() {
        println!("No .strings files found in {}", lproj_folder);
        return Ok(());
    }

    for file_path in strings_files {
        println!("Syncing: {}", file_path.display());
        sync_strings_file(&file_path, &original_keys, &original_entries)?;
    }

    println!("All .strings files synchronized successfully!");
    Ok(())
}

fn find_strings_files(folder: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if folder.is_dir() {
        for entry in fs::read_dir(folder).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                result.extend(find_strings_files(&path));
            } else if path.extension().and_then(|s| s.to_str()) == Some("strings") {
                result.push(path);
            }
        }
    }
    result
}

// ──────────────────────────────────────────────────────────────
//  ПАРСИНГ
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct StringEntry {
    key: String,
    raw_lines: Vec<String>,
}

fn parse_strings_with_order(content: &str) -> Vec<StringEntry> {
    let mut entries = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let (entry, new_i) = parse_multiline_entry(&lines, i);
        if let Some(ent) = entry {
            if !ent.key.is_empty() {
                entries.push(ent);
            }
        }
        i = new_i;
    }
    entries
}

fn sync_strings_file(
    path: &Path,
    original_keys: &HashSet<String>,
    original_entries: &Vec<StringEntry>,
) -> io::Result<()> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    let mut preserved_comments = Vec::new();
    let mut existing_keys = HashSet::new();

    // 1. Сбор комментариев и существующих ключей
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            i += 1;
            continue;
        }
        if trimmed.starts_with("/*") || trimmed.starts_with("//") {
            preserved_comments.push(line.to_string());
            i += 1;
            continue;
        }

        let (entry, new_i) = parse_multiline_entry(&lines, i);
        if let Some(ent) = entry {
            if !ent.key.is_empty() {
                existing_keys.insert(ent.key.clone());
            }
            i = new_i;
        } else {
            i += 1;
        }
    }

    // 2. Формируем результат
    let mut output_lines = Vec::new();

    // Комментарии в начале
    for comment in &preserved_comments {
        if !comment.trim().is_empty() {
            output_lines.push(comment.clone());
        }
    }
    if !preserved_comments.is_empty() {
        output_lines.push(String::new());
    }

    // Проходим по оригинальному порядку
    for orig_entry in original_entries {
        let key = &orig_entry.key;

        if existing_keys.contains(key) {
            // Ключ есть — берём его запись из целевого файла
            let target_content = fs::read_to_string(path)?;
            let target_lines: Vec<&str> = target_content.lines().collect();
            let mut j = 0;
            let mut found = false;
            while j < target_lines.len() && !found {
                let (entry, new_j) = parse_multiline_entry(&target_lines, j);
                if let Some(ent) = entry {
                    if ent.key == *key {
                        for line in &ent.raw_lines {
                            output_lines.push(line.clone());
                        }
                        found = true;
                    }
                    j = new_j;
                } else {
                    j += 1;
                }
            }
        } else {
            // Ключ отсутствует — копируем из original
            println!("   Adding missing key: {}", key);
            for line in &orig_entry.raw_lines {
                output_lines.push(line.clone());
            }
        }
    }

    // 3. Убираем лишние пустые строки
    let mut final_lines = Vec::new();
    let mut last_empty = false;
    for line in output_lines {
        if line.trim().is_empty() {
            if !last_empty {
                final_lines.push(String::new());
                last_empty = true;
            }
        } else {
            final_lines.push(line);
            last_empty = false;
        }
    }

    // Убираем пустые в начале и конце
    let mut final_lines: Vec<String> = final_lines
        .into_iter()
        .skip_while(|s| s.trim().is_empty())
        .collect();
    while final_lines.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
        final_lines.pop();
    }

    // 4. Запись
    let mut file = fs::File::create(path)?;
    if !final_lines.is_empty() {
        writeln!(file, "{}", final_lines.join("\n"))?;
    }
    Ok(())
}

// ──────────────────────────────────────────────────────────────
//  МНОГОСТРОЧНЫЙ ПАРСЕР
// ──────────────────────────────────────────────────────────────

fn parse_multiline_entry(lines: &[&str], start: usize) -> (Option<StringEntry>, usize) {
    let mut i = start;
    let mut raw_lines = Vec::new();
    let mut full_text = String::new();
    let mut key = None;

    while i < lines.len() {
        let line = lines[i];
        raw_lines.push(line.to_string());
        full_text.push_str(line);
        full_text.push('\n');

        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("/*") || trimmed.starts_with("//") {
            i += 1;
            continue;
        }

        if key.is_none() {
            if let Some(eq_pos) = full_text.find('=') {
                let before = &full_text[..eq_pos];
                if let Some(k) = extract_key_from_text(before) {
                    key = Some(k);
                }
            }
        }

        if line.trim_end().ends_with('\\') {
            i += 1;
            continue;
        }

        if key.is_some() {
            return (
                Some(StringEntry {
                    key: key.unwrap(),
                    raw_lines,
                }),
                i + 1,
            );
        }

        i += 1;
    }

    (None, i)
}

fn extract_key_from_text(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if !trimmed.starts_with('"') {
        return None;
    }

    let mut chars = trimmed[1..].chars();
    let mut key = String::new();
    let mut escape = false;

    while let Some(c) = chars.next() {
        if escape {
            key.push(c);
            escape = false;
            continue;
        }
        if c == '\\' {
            escape = true;
            continue;
        }
        if c == '"' {
            return Some(key);
        }
        key.push(c);
    }
    None
}
