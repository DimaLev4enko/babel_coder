use copypasta::{ClipboardContext, ClipboardProvider};
use inquire::{Select, Text};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::io::{self, Write};
use uuid::Uuid;

// === КОНСТАНТЫ ===
const BASE_URL: &str = "https://libraryofbabel.info";
const SEARCH_URL: &str = "https://libraryofbabel.info/search.cgi";
const BOOK_URL: &str = "https://libraryofbabel.info/book.cgi";

fn main() {
    loop {
        println!("\n=== BABEL AUTO-HACKER v3.0 (Onclick Bypass) ===");
        let options = vec![
            "📦 ЗАШИФРОВАТЬ (Pack)",
            "🔮 АВТО-РАСШИФРОВКА (Auto-Fetch)",
            "🚪 Выход",
        ];

        let ans = Select::new("Выбери действие:", options).prompt();

        match ans {
            Ok("📦 ЗАШИФРОВАТЬ (Pack)") => do_pack(),
            Ok("🔮 АВТО-РАСШИФРОВКА (Auto-Fetch)") => do_auto_unpack(),
            Ok("🚪 Выход") => break,
            _ => break,
        }
    }
}

fn hex_to_babel(hex_str: &str) -> String {
    let mut result = String::new();
    for c in hex_str.chars() {
        let mapped = match c {
            '0'..='9' => (c as u8 - b'0' + b'a') as char,
            'a'..='f' => (c as u8 - b'a' + b'k') as char,
            _ => continue,
        };
        result.push(mapped);
    }
    result
}

fn babel_to_hex(babel_str: &str) -> String {
    let mut result = String::new();
    for c in babel_str.chars() {
        let original = match c {
            'a'..='j' => (c as u8 - b'a' + b'0') as char,
            'k'..='p' => (c as u8 - b'k' + b'a') as char,
            _ => continue,
        };
        result.push(original);
    }
    result
}

fn do_pack() {
    let input = match Text::new("Введи текст:").prompt() {
        Ok(t) => t,
        Err(_) => return,
    };

    let raw_uuid = Uuid::new_v4().to_string();
    let safe_id = hex_to_babel(&raw_uuid)[..10].to_string();

    let hex_payload = hex::encode(input);
    let safe_payload = hex_to_babel(&hex_payload);

    let search_query = format!("key{}val{}q", safe_id, safe_payload);

    println!("\n✅ ЗАШИФРОВАНО!");
    println!("🔑 ТВОЙ КЛЮЧ (дай другу): {}", search_query);

    if let Ok(mut ctx) = ClipboardContext::new() {
        let _ = ctx.set_contents(search_query);
        println!("(Скопировано в буфер)");
    }
}

fn do_auto_unpack() {
    let key = match Text::new("Вставь ключ (начинается с key...):").prompt() {
        Ok(t) => t.trim().to_string(), // Убираем случайные пробелы и \n
        Err(_) => return,
    };

    print!("🌍 Подключаюсь к Библиотеке Вавилона... ");
    io::stdout().flush().unwrap();

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:120.0) Gecko/20100101 Firefox/120.0")
        .build()
        .unwrap();

    let mut params = HashMap::new();
    params.insert("find", key.as_str());

    let res = match client.post(SEARCH_URL).form(&params).send() {
        Ok(r) => r,
        Err(e) => {
            println!("\n❌ Ошибка сети: {}", e);
            return;
        }
    };

    let html_content = res.text().unwrap_or_default();
    println!("OK.");

    print!("🕵️ Парсим координаты книги... ");
    io::stdout().flush().unwrap();

    let document = Html::parse_document(&html_content);
    let link_selector = Selector::parse("a.intext").unwrap();

    let mut book_params = HashMap::new();

    // [ГЛАВНЫЙ ХАК]: Ищем атрибут onclick и парсим из него координаты
    for element in document.select(&link_selector) {
        if let Some(onclick) = element.value().attr("onclick") {
            if onclick.starts_with("postform(") {
                // Вырезаем мусор: postform('hex','1','2','3','4') -> hex,1,2,3,4
                let clean = onclick
                    .replace("postform(", "")
                    .replace(")", "")
                    .replace("'", "")
                    .replace(" ", "");

                let parts: Vec<&str> = clean.split(',').collect();
                if parts.len() >= 5 {
                    book_params.insert("hex", parts[0].to_string());
                    book_params.insert("wall", parts[1].to_string());
                    book_params.insert("shelf", parts[2].to_string());
                    book_params.insert("volume", parts[3].to_string());
                    book_params.insert("page", parts[4].to_string());
                    break;
                }
            }
        }
    }

    if book_params.is_empty() {
        println!("\n❌ Ничего не найдено. Ключ неверный или не удалось распарсить onclick.");
        return;
    }

    println!(
        "Нашел! (Wall: {}, Shelf: {}, Vol: {}, Page: {})",
        book_params["wall"], book_params["shelf"], book_params["volume"], book_params["page"]
    );

    print!("📥 Скачиваю содержимое книги... ");
    io::stdout().flush().unwrap();

    // Теперь мы делаем POST запрос напрямую к book.cgi с координатами
    let book_res = match client.post(BOOK_URL).form(&book_params).send() {
        Ok(r) => r,
        Err(_) => {
            println!("\n❌ Не удалось скачать книгу.");
            return;
        }
    };

    let book_html = book_res.text().unwrap_or_default();
    println!("OK.");

    let book_doc = Html::parse_document(&book_html);
    let text_selector = Selector::parse("#textblock").unwrap();

    let full_text = if let Some(text_div) = book_doc.select(&text_selector).next() {
        text_div.text().collect::<String>()
    } else {
        book_html
    };

    if let Some(start_val) = full_text.find("val") {
        let payload_part = &full_text[start_val + 3..];

        let mut clean_payload = String::new();

        for c in payload_part.chars() {
            if c == 'q' {
                break; // Наш терминатор конца сообщения
            } else if c >= 'a' && c <= 'p' {
                clean_payload.push(c);
            } else if c.is_whitespace() {
                continue;
            } else {
                break;
            }
        }

        let hex_str = babel_to_hex(&clean_payload);
        match hex::decode(&hex_str) {
            Ok(bytes) => {
                let result = String::from_utf8_lossy(&bytes);
                println!("\n🔓 ========================");
                println!("РАСШИФРОВАНО:\n{}", result);
                println!("==========================");
            }
            Err(_) => println!("\n❌ Ошибка декодирования (битые данные). HEX: {}", hex_str),
        }
    } else {
        println!("\n❌ Текст книги скачан, но я не вижу там метку 'val'.");
    }
}
