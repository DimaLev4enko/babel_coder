use inquire::{CustomType, Select, Text};
use scraper::{Html, Selector};
use std::fs;
use std::io::{self, Write};
use uuid::Uuid;

const BASE_URL: &str = "https://libraryofbabel.info";
const SEARCH_URL: &str = "https://libraryofbabel.info/search.cgi";
const BOOK_URL: &str = "https://libraryofbabel.info/book.cgi";
const MAX_PAGE_SIZE: usize = 3100;

fn main() {
    loop {
        println!("\n=== BABEL ENGINE v2.0 (The Iron Logic) ===");
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

// === УТИЛИТЫ ВВОДА / ВЫВОДА ===
fn get_data(msg: &str) -> Vec<u8> {
    let mode = Select::new(msg, vec!["Ввести текст руками", "Указать путь к файлу"])
        .prompt()
        .unwrap();
    if mode == "Ввести текст руками" {
        Text::new("Ввод:").prompt().unwrap().into_bytes()
    } else {
        let path = Text::new("Путь к файлу:").prompt().unwrap();
        fs::read(&path).unwrap_or_else(|_| {
            println!("❌ Ошибка: файл не найден. Беру пустую строку.");
            vec![]
        })
    }
}

fn save_data(data: &[u8]) {
    let mode = Select::new(
        "Как вывести результат?",
        vec!["В терминал", "Сохранить в файл"],
    )
    .prompt()
    .unwrap();
    if mode == "В терминал" {
        println!("\n🔓 РЕЗУЛЬТАТ:\n{}", String::from_utf8_lossy(data));
    } else {
        let path = Text::new("Имя файла (напр. output.txt):").prompt().unwrap();
        fs::write(&path, data).expect("❌ Ошибка сохранения");
        println!("✅ Сохранено в {}", path);
    }
}

// === ПОТОКИ (FLOWS) ===
fn transport_flow() {
    let opt = Select::new("Действие:", vec!["Упаковать в Вавилон", "Распаковать"])
        .prompt()
        .unwrap();
    if opt == "Упаковать в Вавилон" {
        let data = get_data("Что пакуем?");
        let key_str = pack_to_babel(data);
        println!("\n✅ УСПЕШНО!\n🔑 ТВОЙ КЛЮЧ:\n{}", key_str);
    } else {
        let k = Text::new("Вставь строку (key..._key...):")
            .prompt()
            .unwrap();
        let unpacked = unpack_from_babel(&k, None);
        save_data(&unpacked);
    }
}

fn crypto_flow() {
    let opt = Select::new("Действие:", vec!["🔒 Зашифровать", "🔓 Расшифровать"])
        .prompt()
        .unwrap();
    if opt == "🔒 Зашифровать" {
        let data = get_data("Что шифруем?");
        let key_mode = Select::new(
            "Метод ключа:",
            vec![
                "Свой пароль/файл (Manual)",
                "Случайный из Вавилона (SKEY - Вернам)",
            ],
        )
        .prompt()
        .unwrap();

        let (encrypted, skey_str) = if key_mode == "Свой пароль/файл (Manual)" {
            let k = get_data("Выбери ключ-маску:");
            (xor_cipher(&data, &k), String::new())
        } else {
            let (k, s) = generate_proportional_skey(data.len());
            (xor_cipher(&data, &k), s)
        };

        let result_key = pack_to_babel(encrypted);
        println!("\n✅ ЗАШИФРОВАНО И УПАКОВАНО!");

        // РАЗДЕЛЯЕМ ВЫВОД КЛЮЧЕЙ
        if skey_str.is_empty() {
            println!("📦 ПУБЛИЧНЫЕ ДАННЫЕ (KEY):\n{}", result_key);
        } else {
            println!(
                "🛡️ СЕКРЕТНЫЙ КЛЮЧ (SKEY - Храни у себя или передай лично!):\n{}",
                skey_str
            );
            println!(
                "\n📦 ПУБЛИЧНЫЕ ДАННЫЕ (KEY - Можно кидать в открытую сеть):\n{}",
                result_key
            );
        }
    } else {
        // РАЗДЕЛЯЕМ ВВОД ПРИ РАСШИФРОВКЕ
        let data_chain = Text::new("Вставь публичные данные (начинается с key...):")
            .prompt()
            .unwrap();

        let key_mode = Select::new(
            "Где ключ для расшифровки?",
            vec!["У меня есть SKEY", "Свой пароль/файл (Manual)"],
        )
        .prompt()
        .unwrap();

        let manual_key = if key_mode == "У меня есть SKEY" {
            let skey_input = Text::new("Вставь секретный SKEY:").prompt().unwrap();
            // Очищаем от тегов skey[] для парсинга координат
            let coords_str = skey_input.replace("skey[", "").replace("]", "");
            println!("📥 Тяну SKEY (шум) из Вавилона по координатам...");
            Some(fetch_skey_noise(&coords_str))
        } else {
            Some(get_data("Выбери свой ключ-маску (Manual):"))
        };

        let encrypted_data = unpack_from_babel(&data_chain, manual_key.as_deref());
        save_data(&encrypted_data);
    }
}

// === КРИПТОГРАФИЯ И SKEY ===
fn xor_cipher(data: &[u8], key: &[u8]) -> Vec<u8> {
    if key.is_empty() {
        return data.to_vec();
    }
    data.iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % key.len()])
        .collect()
}

fn generate_proportional_skey(needed_bytes: usize) -> (Vec<u8>, String) {
    let pages_needed = (needed_bytes as f64 / MAX_PAGE_SIZE as f64).ceil() as usize;
    println!(
        "🎲 Генерация SKEY: нужно {} байт ({} страниц Вавилона)",
        needed_bytes, pages_needed
    );

    let mut all_noise = Vec::new();
    let mut coords_list = Vec::new();

    for i in 0..pages_needed {
        let rand_uuid = Uuid::new_v4();
        let b = rand_uuid.as_bytes();

        let wall: u8 = (b[0] % 4) + 1;
        let shelf: u8 = (b[1] % 5) + 1;
        let vol: u8 = (b[2] % 32) + 1;

        let page_raw = u16::from_le_bytes([b[3], b[4]]);
        let page: u16 = (page_raw % 410) + 1;

        let hex_room = rand_uuid.simple().to_string()[..15].to_string();

        let coord = format!("{}:{}:{}:{}:{}", wall, shelf, vol, page, hex_room);
        println!("📍 SKEY страница {}/{}: {}", i + 1, pages_needed, coord);

        let noise = fetch_page_by_coords(&coord);
        all_noise.extend_from_slice(noise.as_bytes());
        coords_list.push(coord);
    }

    all_noise.truncate(needed_bytes);
    (all_noise, format!("skey[{}]", coords_list.join("_")))
}

fn fetch_skey_noise(coords_str: &str) -> Vec<u8> {
    let coords: Vec<&str> = coords_str.split('_').collect();
    let mut full_noise = Vec::new();
    for (i, c) in coords.iter().enumerate() {
        print!("🌍 Выкачиваю SKEY часть {}/{}... ", i + 1, coords.len());
        io::stdout().flush().unwrap();
        let noise = fetch_page_by_coords(c);
        full_noise.extend_from_slice(noise.as_bytes());
        println!("OK.");
    }
    full_noise
}

// === ЯДРО ВАВИЛОНА (THE IRON LOGIC) ===
fn pack_to_babel(data: Vec<u8>) -> String {
    let uuid_len =
        CustomType::<usize>::new("Длина UUID (от 10 до 1000, больше = надежнее, но дольше):")
            .with_default(20)
            .with_error_message("Введите число.")
            .prompt()
            .unwrap();

    let hex_data = hex::encode(data);
    let babel_data = hex_to_babel(&hex_data);

    let chunk_capacity = MAX_PAGE_SIZE.saturating_sub(10 + uuid_len);
    let chars: Vec<char> = babel_data.chars().collect();
    let chunks: Vec<&[char]> = chars.chunks(chunk_capacity).collect();

    let mut final_keys = Vec::new();
    let client = reqwest::blocking::Client::new();

    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_str: String = chunk.iter().collect();
        let mut attempts = 0;

        loop {
            attempts += 1;
            let raw_uuid = Uuid::new_v4().simple().to_string().repeat(30);
            let safe_uuid = &hex_to_babel(&raw_uuid)[..uuid_len];

            let query = format!("key{}val{}q", safe_uuid, chunk_str);
            print!(
                "🔄 [Чанк {}/{}] Проверка в Вавилоне (попытка {})... ",
                i + 1,
                chunks.len(),
                attempts
            );
            io::stdout().flush().unwrap();

            if verify_and_upload(&client, &query) {
                println!("✅ Зафиксировано!");
                final_keys.push(query);
                break;
            } else {
                println!("⚠️ Фантом. Меняю UUID...");
            }
        }
    }
    final_keys.join("_")
}

fn unpack_from_babel(input_chain: &str, crypto_key: Option<&[u8]>) -> Vec<u8> {
    let parts: Vec<&str> = input_chain.split('_').collect();
    let client = reqwest::blocking::Client::new();
    let mut full_babel_payload = String::new();

    for (i, query) in parts.iter().enumerate() {
        print!("🌍 [Чанк {}/{}] Тяну данные... ", i + 1, parts.len());
        io::stdout().flush().unwrap();

        let mut success = false;

        let body = format!("find={}", *query);
        if let Ok(res) = client
            .post(SEARCH_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
        {
            if let Some(coords) = parse_onclick(&res.text().unwrap_or_default()) {
                let page_text = fetch_page_by_coords(&coords);
                if let Some(val_idx) = page_text.find("val") {
                    let sub = &page_text[val_idx + 3..];
                    if let Some(q_idx) = sub.find('q') {
                        full_babel_payload.push_str(&sub[..q_idx]);
                        println!("OK (Через Библиотеку).");
                        success = true;
                    }
                }
            }
        }

        if !success {
            if let Some(val_idx) = query.find("val") {
                let sub = &query[val_idx + 3..];
                if let Some(q_idx) = sub.find('q') {
                    full_babel_payload.push_str(&sub[..q_idx]);
                    println!("OK (Failsafe: извлечено локально).");
                }
            } else {
                println!("❌ БИТЫЙ КЛЮЧ!");
            }
        }
    }

    let hex_str = babel_to_hex(&full_babel_payload);
    let encrypted_bytes = hex::decode(&hex_str).unwrap_or_default();

    if let Some(k) = crypto_key {
        let exact_key = if k.len() > encrypted_bytes.len() {
            &k[..encrypted_bytes.len()]
        } else {
            k
        };
        xor_cipher(&encrypted_bytes, exact_key)
    } else {
        encrypted_bytes
    }
}

// === СЕТЕВЫЕ ВОКЕРЫ (WORKERS) ===
fn verify_and_upload(client: &reqwest::blocking::Client, query: &str) -> bool {
    let body = format!("find={}", query);

    let res = match client
        .post(SEARCH_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
    {
        Ok(r) => r.text().unwrap_or_default(),
        Err(_) => return false,
    };

    if let Some(coords) = parse_onclick(&res) {
        let page_text = fetch_page_by_coords(&coords);
        page_text.contains(query)
    } else {
        false
    }
}

fn fetch_page_by_coords(coords: &str) -> String {
    let parts: Vec<&str> = coords.split(':').collect();
    if parts.len() < 5 {
        return String::new();
    }

    let body = format!(
        "wall={}&shelf={}&volume={}&page={}&hex={}",
        parts[0], parts[1], parts[2], parts[3], parts[4]
    );

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(BOOK_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .unwrap()
        .text()
        .unwrap();

    let doc = Html::parse_document(&res);
    if let Some(text_div) = doc.select(&Selector::parse("#textblock").unwrap()).next() {
        text_div.text().collect::<String>()
    } else {
        String::new()
    }
}

fn parse_onclick(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    for el in doc.select(&Selector::parse("a.intext").unwrap()) {
        if let Some(onclick) = el.value().attr("onclick") {
            if onclick.starts_with("postform(") {
                let clean = onclick
                    .replace("postform(", "")
                    .replace(')', "")
                    .replace('\'', "")
                    .replace(' ', "");
                let p: Vec<&str> = clean.split(',').collect();
                if p.len() >= 5 {
                    return Some(format!("{}:{}:{}:{}:{}", p[1], p[2], p[3], p[4], p[0]));
                }
            }
        }
    }
    None
}

// === ШЕСТНАДЦАТЕРИЧНАЯ МАГИЯ ===
fn hex_to_babel(hex: &str) -> String {
    hex.chars()
        .map(|c| match c {
            '0'..='9' => (c as u8 - b'0' + b'a') as char,
            'a'..='f' => (c as u8 - b'a' + b'k') as char,
            _ => c,
        })
        .collect()
}

fn babel_to_hex(babel: &str) -> String {
    babel
        .chars()
        .map(|c| match c {
            'a'..='j' => (c as u8 - b'a' + b'0') as char,
            'k'..='p' => (c as u8 - b'k' + b'a') as char,
            _ => c,
        })
        .collect()
}
