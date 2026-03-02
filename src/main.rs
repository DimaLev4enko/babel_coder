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
        println!("\n=== BABEL ENGINE v3.0 ===");
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
    let mode = Select::new(msg, vec!["Вставить строку", "Прочитать из .babel"])
        .prompt()
        .unwrap();
    let raw = if mode == "Вставить строку" {
        Text::new("Ввод:").prompt().unwrap()
    } else {
        let path = Text::new("Путь:").prompt().unwrap();
        fs::read_to_string(&path).unwrap_or_default()
    };
    raw.lines()
        .map(|l| l.trim())
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .collect()
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
                "1. SKEY (Надежно, ключ отдельно)",
                "2. Маскировка (Ненадежно, ключ внутри - 'Всё в одном')",
                "3. Свой пароль (Manual)",
            ],
        )
        .prompt()
        .unwrap();

        if mode.starts_with('1') {
            let (k, s) = generate_proportional_skey(data.len());
            handle_output_options(&xor_cipher(&data, &k), Some(s), false);
        } else if mode.starts_with('2') {
            handle_output_options(&data, None, true); // Включаем режим маскировки
        } else {
            let k = get_raw_data("Пароль/Маска:");
            handle_output_options(&xor_cipher(&data, &k), None, false);
        }
    } else {
        let data_in = get_babel_string("Зашифрованные данные:");
        let mode = Select::new(
            "Ключ:",
            vec![
                "У меня есть SKEY",
                "Маскировка (Ключ внутри)",
                "Свой пароль (Manual)",
            ],
        )
        .prompt()
        .unwrap();

        let decrypted = if mode.contains("SKEY") {
            let skey = parse_skey(&get_babel_string("Введи SKEY:"));
            unpack_data(&data_in, Some(&skey), false)
        } else if mode.contains("Маскировка") {
            unpack_data(&data_in, None, true) // Режим маскировки сам найдет ключ
        } else {
            let k = get_raw_data("Пароль/Маска:");
            unpack_data(&data_in, Some(&k), false)
        };
        save_result(&decrypted);
    }
}

fn handle_output_options(data: &[u8], skey: Option<String>, is_masked: bool) {
    // В режиме маскировки приклеиваем ключ к данным ПЕРЕД упаковкой
    let final_payload = if is_masked {
        let mut masked = Uuid::new_v4().as_bytes().to_vec(); // 16 байт ключа
        let mut encrypted = xor_cipher(data, &masked);
        masked.append(&mut encrypted);
        masked
    } else {
        data.to_vec()
    };

    if let Some(s) = skey {
        println!("\n🔑 SKEY:\n{}", s);
    }

    let opt = Select::new(
        "\nВывод ДАННЫХ:",
        vec![
            "1. .babel файл",
            "2. Колбаса (Терминал)",
            "3. Локальный HEX",
        ],
    )
    .prompt()
    .unwrap();

    if opt.starts_with('1') {
        let path = Text::new("Имя:")
            .with_default("out.babel")
            .prompt()
            .unwrap();
        let content = if opt.contains("HEX") {
            hex::encode(&final_payload)
        } else {
            pack_chunks_to_babel(&final_payload).join("_")
        };
        fs::write(path, format!("# Babel v3.0 Data\n{}", content)).unwrap();
    } else if opt.starts_with('2') {
        println!(
            "\n📦 РЕЗУЛЬТАТ:\n{}",
            pack_chunks_to_babel(&final_payload).join("_")
        );
    } else {
        println!("\n📦 HEX:\n{}", hex::encode(&final_payload));
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

fn pack_chunks_to_babel(data: &[u8]) -> Vec<String> {
    let hex_data = hex_to_babel(&hex::encode(data));
    let client = reqwest::blocking::Client::new();
    hex_data
        .chars()
        .collect::<Vec<_>>()
        .chunks(MAX_PAGE_SIZE - 50)
        .enumerate()
        .map(|(i, chunk)| {
            let chunk_str: String = chunk.iter().collect();
            loop {
                let id = &hex_to_babel(&Uuid::new_v4().simple().to_string())[..20];
                let q = format!("key{}val{}q", id, chunk_str);
                print!("🔄 Чанк {}... ", i + 1);
                io::stdout().flush().unwrap();
                if verify(&client, &q) {
                    println!("✅");
                    return q;
                }
                println!("⚠️");
            }
        })
        .collect()
}

fn unpack_data(input: &str, external_key: Option<&[u8]>, is_masked: bool) -> Vec<u8> {
    let mut full_hex = String::new();
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

fn verify(client: &reqwest::blocking::Client, q: &str) -> bool {
    let res = client
        .post(SEARCH_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!("find={}", q))
        .send()
        .and_then(|r| r.text())
        .unwrap_or_default();
    if let Some(c) = parse_oc(&res) {
        fetch_page(&c).contains(q)
    } else {
        false
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
            Some(format!("{}:{}:{}:{}:{}", p[1], p[2], p[3], p[4], p[0]))
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
