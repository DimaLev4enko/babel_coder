use inquire::{Select, Text};
use scraper::{Html, Selector};
use std::fs;
use std::io::{self, Write};
use uuid::Uuid;

const SEARCH_URL: &str = "https://libraryofbabel.info/search.cgi";
const BOOK_URL: &str = "https://libraryofbabel.info/book.cgi";
const MAX_PAGE_SIZE: usize = 3000;

fn main() {
    loop {
        println!("\n=== BABEL ENGINE v3.1 (True Babel Storage) ===");
        let options = vec![
            "📦 ТРАНСПОРТ (Открытый канал)",
            "🔐 КРИПТОГРАФИЯ (Шифрование)",
            "🚪 Выход",
        ];

        match Select::new("Главное меню:", options).prompt() {
            Ok("📦 ТРАНСПОРТ (Открытый канал)") => transport_flow(),
            Ok("🔐 КРИПТОГРАФИЯ (Шифрование)") => crypto_flow(),
            Ok("🚪 Выход") | Err(_) => break,
            _ => break,
        }
    }
}

// === УТИЛИТЫ ===
fn get_raw_data(msg: &str) -> Vec<u8> {
    let mode = Select::new(msg, vec!["Ввести текст руками", "Указать путь к файлу"])
        .prompt()
        .unwrap();
    if mode == "Ввести текст руками" {
        Text::new("Ввод:").prompt().unwrap().into_bytes()
    } else {
        let path = Text::new("Путь к файлу:").prompt().unwrap();
        fs::read(&path).unwrap_or_else(|_| {
            println!("❌ Файл не найден!");
            vec![]
        })
    }
}

fn get_babel_string(msg: &str) -> String {
    let mode = Select::new(
        msg,
        vec![
            "Вставить строку (Координаты/Колбаса)",
            "Прочитать из .babel",
        ],
    )
    .prompt()
    .unwrap();
    let raw = if mode.starts_with("Вставить") {
        Text::new("Ввод:").prompt().unwrap()
    } else {
        let path = Text::new("Путь:").prompt().unwrap();
        fs::read_to_string(&path).unwrap_or_default()
    };
    raw.lines()
        .map(|l| l.trim())
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

// === ПОТОКИ ===
fn transport_flow() {
    let opt = Select::new("Действие:", vec!["Упаковать в Вавилон", "Распаковать"])
        .prompt()
        .unwrap();
    if opt == "Упаковать в Вавилон" {
        let data = get_raw_data("Что пакуем?");
        handle_output_options(&data, None, false);
    } else {
        let input = get_babel_string("Данные для распаковки:");
        let unpacked = unpack_data(&input, None, false);
        save_result(&unpacked);
    }
}

fn crypto_flow() {
    let opt = Select::new("Действие:", vec!["🔒 Шифрование", "🔓 Расшифрование"])
        .prompt()
        .unwrap();
    if opt == "🔒 Шифрование" {
        let data = get_raw_data("Что шифруем?");
        let mode = Select::new(
            "Метод:",
            vec![
                "1. SKEY (Ссылки link[...])",
                "2. Маскировка (Ключ внутри)",
                "3. Свой пароль (Manual)",
            ],
        )
        .prompt()
        .unwrap();

        if mode.starts_with('1') {
            let (k, s) = generate_proportional_skey(data.len());
            handle_output_options(&xor_cipher(&data, &k), Some(s), false);
        } else if mode.starts_with('2') {
            handle_output_options(&data, None, true);
        } else {
            let k = get_raw_data("Пароль/Маска:");
            handle_output_options(&xor_cipher(&data, &k), None, false);
        }
    } else {
        let data_in = get_babel_string("Зашифрованные данные (Координаты):");
        let mode = Select::new(
            "Ключ:",
            vec![
                "У меня есть SKEY (Ссылки)",
                "Маскировка (Ключ внутри)",
                "Свой пароль",
            ],
        )
        .prompt()
        .unwrap();

        let decrypted = if mode.contains("SKEY") {
            let skey = parse_skey(&get_babel_string("Введи SKEY:"));
            unpack_data(&data_in, Some(&skey), false)
        } else if mode.contains("Маскировка") {
            unpack_data(&data_in, None, true)
        } else {
            let k = get_raw_data("Пароль/Маска:");
            unpack_data(&data_in, Some(&k), false)
        };
        save_result(&decrypted);
    }
}

fn handle_output_options(data: &[u8], skey: Option<String>, is_masked: bool) {
    let final_payload = if is_masked {
        let mut masked = Uuid::new_v4().as_bytes().to_vec();
        let mut encrypted = xor_cipher(data, &masked);
        masked.append(&mut encrypted);
        masked
    } else {
        data.to_vec()
    };

    if let Some(s) = skey {
        println!("\n🔑 SKEY (Ссылки):\n{}", s);
    }

    let opt = Select::new(
        "\nВывод ДАННЫХ:",
        vec![
            "1. .babel файл (Полные координаты: стена:полка:том:стр:ОГРОМНЫЙ_ХЕКС)",
            "2. Терминал (Старая колбаса key...val...q)",
        ],
    )
    .prompt()
    .unwrap();

    println!("🌐 Заливаю данные в Вавилон...");
    let (sausages, coords) = pack_to_babel(&final_payload);

    if opt.starts_with('1') {
        let path = Text::new("Имя:")
            .with_default("out.babel")
            .prompt()
            .unwrap();
        let content = coords.join("\n");
        // СОХРАНЯЕМ ИСТИННЫЕ КООРДИНАТЫ ВМЕСТО КОМПЬЮТЕРНОГО ХЕКСА
        fs::write(
            &path,
            format!("# Babel Data Storage Coordinates\n{}", content),
        )
        .unwrap();
        println!("✅ Истинные координаты сохранены в {}", path);
    } else {
        println!("\n📦 РЕЗУЛЬТАТ (Колбаса):\n{}", sausages.join("_"));
    }
}

fn save_result(data: &[u8]) {
    let mode = Select::new("Результат:", vec!["В терминал", "В файл"])
        .prompt()
        .unwrap();
    if mode == "В терминал" {
        println!("\n🔓:\n{}", String::from_utf8_lossy(data));
    } else {
        fs::write(Text::new("Имя:").prompt().unwrap(), data).unwrap();
    }
}

// === ЯДРО ===
fn xor_cipher(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % key.len()])
        .collect()
}

// ИСТИННАЯ ЗАЛИВКА В ВАВИЛОН (ВОЗВРАЩАЕТ И КОЛБАСУ И КООРДИНАТЫ)
fn pack_to_babel(data: &[u8]) -> (Vec<String>, Vec<String>) {
    let hex_data = hex_to_babel(&hex::encode(data));
    let client = reqwest::blocking::Client::new();
    let mut sausages = Vec::new();
    let mut coordinates = Vec::new();

    for (i, chunk) in hex_data
        .chars()
        .collect::<Vec<_>>()
        .chunks(MAX_PAGE_SIZE - 50)
        .enumerate()
    {
        let chunk_str: String = chunk.iter().collect();
        loop {
            let id = &hex_to_babel(&Uuid::new_v4().simple().to_string())[..20];
            let q = format!("key{}val{}q", id, chunk_str);
            print!("🔄 Чанк {}... ", i + 1);
            io::stdout().flush().unwrap();

            let res = client
                .post(SEARCH_URL)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(format!("find={}", q))
                .send()
                .and_then(|r| r.text())
                .unwrap_or_default();
            if let Some(coords) = parse_oc(&res) {
                if fetch_page(&coords).contains(&q) {
                    println!("✅");
                    sausages.push(q);
                    coordinates.push(coords); // Сохраняем стену:полку:том:страницу:огромный_хекс
                    break;
                }
            }
            println!("⚠️");
        }
    }
    (sausages, coordinates)
}

fn unpack_data(input: &str, external_key: Option<&[u8]>, is_masked: bool) -> Vec<u8> {
    let mut full_hex = String::new();

    // Если есть двоеточия, значит это полные координаты из .babel файла
    if input.contains(':') && input.lines().any(|l| l.split(':').count() >= 5) {
        println!("🌐 Тяну данные по координатам из хранилища Вавилона...");
        for line in input.lines() {
            let coords = line.trim();
            if coords.starts_with('#') || coords.is_empty() {
                continue;
            }
            let page_text = fetch_page(coords);
            if let Some(v) = page_text.find("val") {
                let sub = &page_text[v + 3..];
                if let Some(q) = sub.find('q') {
                    full_hex.push_str(&babel_to_hex(&sub[..q]));
                }
            }
        }
    } else {
        // Старая колбаса (ввод из терминала)
        for part in input.split('_') {
            if let Some(v) = part.find("val") {
                let sub = &part[v + 3..];
                if let Some(q) = sub.find('q') {
                    full_hex.push_str(&babel_to_hex(&sub[..q]));
                }
            } else {
                full_hex.push_str(part);
            }
        }
    }

    let mut bytes = hex::decode(full_hex.trim()).unwrap_or_default();

    if is_masked && bytes.len() > 16 {
        let key = bytes[0..16].to_vec();
        xor_cipher(&bytes[16..], &key)
    } else if let Some(k) = external_key {
        xor_cipher(&bytes, k)
    } else {
        bytes
    }
}

// SKEY ссылками
fn generate_proportional_skey(len: usize) -> (Vec<u8>, String) {
    let mut noise = Vec::new();
    let mut links = String::new();
    for _ in 0..(len / MAX_PAGE_SIZE + 1) {
        let u = Uuid::new_v4();
        let b = u.as_bytes();
        let c = format!(
            "{}:{}:{}:{}:{}",
            (b[0] % 4) + 1,
            (b[1] % 5) + 1,
            (b[2] % 32) + 1,
            (u16::from_le_bytes([b[3], b[4]]) % 410) + 1,
            u.simple().to_string()[..15].to_string()
        );
        links.push_str(&format!("link[{}]\n", c));
        noise.extend_from_slice(fetch_page(&c).as_bytes());
    }
    noise.truncate(len);
    (noise, links)
}

fn parse_skey(input: &str) -> Vec<u8> {
    let mut n = Vec::new();
    for line in input.lines() {
        if let Some(s) = line.find("link[") {
            if let Some(e) = line[s..].find(']') {
                n.extend_from_slice(fetch_page(&line[s + 5..s + e]).as_bytes());
            }
        }
    }
    if n.is_empty() {
        input.as_bytes().to_vec()
    } else {
        n
    }
}

fn fetch_page(coords: &str) -> String {
    let p: Vec<&str> = coords.split(':').collect();
    if p.len() < 5 {
        return String::new();
    }
    let body = format!(
        "wall={}&shelf={}&volume={}&page={}&hex={}",
        p[0], p[1], p[2], p[3], p[4]
    );
    let res = reqwest::blocking::Client::new()
        .post(BOOK_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .unwrap()
        .text()
        .unwrap();
    Html::parse_document(&res)
        .select(&Selector::parse("#textblock").unwrap())
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_default()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

fn parse_oc(h: &str) -> Option<String> {
    Html::parse_document(h)
        .select(&Selector::parse("a.intext").unwrap())
        .find_map(|el| {
            let oc = el
                .value()
                .attr("onclick")?
                .replace("postform(", "")
                .replace(")", "")
                .replace("'", "")
                .replace(" ", "");
            let p: Vec<&str> = oc.split(',').collect();
            Some(format!("{}:{}:{}:{}:{}", p[1], p[2], p[3], p[4], p[0])) // wall:shelf:vol:page:huge_hex
        })
}

fn hex_to_babel(h: &str) -> String {
    h.chars()
        .map(|c| match c {
            '0'..='9' => (c as u8 - b'0' + b'a') as char,
            'a'..='f' => (c as u8 - b'a' + b'k') as char,
            _ => c,
        })
        .collect()
}
fn babel_to_hex(b: &str) -> String {
    b.chars()
        .map(|c| match c {
            'a'..='j' => (c as u8 - b'a' + b'0') as char,
            'k'..='p' => (c as u8 - b'k' + b'a') as char,
            _ => c,
        })
        .collect()
}
