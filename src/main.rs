use redis::{Client, Connection};
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};

const DEFAULT_KEY_FILE: &str = "redis_keys.txt";
const DIFF_FILE: &str = "key_differences.txt";

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    // Redis bağlantısı oluştur
    let client = Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;

    // Argüman kontrolü
    if args.len() > 1 && args[1] == "--diff" {
        compare_keys(&mut con)?;
    } else {
        dump_keys(&mut con)?;
    }

    Ok(())
}

/// SCAN kullanarak tüm anahtarları dosyaya yazar
fn dump_keys(con: &mut Connection) -> Result<(), Box<dyn Error>> {
    let mut keys: Vec<String> = Vec::new();
    let mut cursor: u64 = 0;
    const BATCH_SIZE: usize = 1000;

    loop {
        let result: (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("COUNT")
            .arg(BATCH_SIZE)
            .query(con)?;
        
        cursor = result.0;
        keys.extend(result.1);
        
        if cursor == 0 {
            break;
        }
    }

    let file = File::create(DEFAULT_KEY_FILE)?;
    let mut writer = BufWriter::new(file);

    for key in &keys {
        writeln!(writer, "{}", key)?;
    }

    println!(
        "Toplam {} anahtar '{}' dosyasına yazıldı.",
        keys.len(),
        DEFAULT_KEY_FILE
    );
    Ok(())
}

/// Mevcut anahtarları dosyayla karşılaştırır
fn compare_keys(con: &mut Connection) -> Result<(), Box<dyn Error>> {
    // Redis'ten güncel anahtarları al
    let mut current_keys: Vec<String> = Vec::new();
    let mut cursor: u64 = 0;
    const BATCH_SIZE: usize = 1000;

    loop {
        let result: (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("COUNT")
            .arg(BATCH_SIZE)
            .query(con)?;
        
        cursor = result.0;
        current_keys.extend(result.1);
        
        if cursor == 0 {
            break;
        }
    }

    let current_set: HashSet<_> = current_keys.iter().collect();

    // Eski anahtarları dosyadan oku
    let old_keys = read_keys_from_file(DEFAULT_KEY_FILE)?;
    let old_set: HashSet<_> = old_keys.iter().collect();

    // Farkları hesapla
    let added: Vec<_> = current_set.difference(&old_set).collect();
    let removed: Vec<_> = old_set.difference(&current_set).collect();

    // Farkları dosyaya yaz
    let diff_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(DIFF_FILE)?;
    let mut writer = BufWriter::new(diff_file);

    writeln!(writer, "Eklenen Anahtarlar ({}):", added.len())?;
    for key in &added {
        writeln!(writer, "{}", key)?;
    }

    writeln!(writer, "\nKaldırılan Anahtarlar ({}):", removed.len())?;
    for key in &removed {
        writeln!(writer, "{}", key)?;
    }

    println!(
        "Farklar '{}' dosyasına yazıldı:\n- Eklenen: {}\n- Kaldırılan: {}",
        DIFF_FILE,
        added.len(),
        removed.len()
    );

    Ok(())
}

/// Dosyadan anahtarları okur
fn read_keys_from_file(filename: &str) -> io::Result<Vec<String>> {
    let file = match File::open(filename) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };

    let reader = BufReader::new(file);
    reader.lines().collect()
}