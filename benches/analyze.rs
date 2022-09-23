use std::collections::HashMap;
use std::env;
use std::fs;
use std::ops::Not;

use anyhow::Context;
use anyhow::Result;
use ordered_float::OrderedFloat;
use serde_json::Value;
use simd_benches::map_collect;
use tabled::Style;
use tabled::Table;

fn default<T: Default>() -> T {
    T::default()
}

#[derive(Debug)]
struct Data {
    function: String,
    crate_: String,
    variant: String,
    case: String,
    time: f64,
}

fn read_messages(path: &str) -> Result<Vec<Value>> {
    let content = fs::read_to_string(path).with_context(|| format!("path = {path}"))?;
    let mut ans = Vec::new();
    for line in content.lines() {
        let msg: Value = serde_json::from_str(line)?;
        ans.push(msg);
    }
    Ok(ans)
}

fn extract_data(messages: &[Value]) -> Result<Vec<Data>> {
    let mut ans = Vec::new();
    for msg in messages {
        let reason = msg["reason"].as_str().unwrap();
        if reason != "benchmark-complete" {
            continue;
        }

        let id = msg["id"].as_str().unwrap();
        let parts: Vec<&str> = id.split('/').collect();
        let function = parts[0].to_owned();
        let crate_ = parts[1].to_owned();
        let variant = parts[2].to_owned();
        let case = parts[3].to_owned();

        let time = msg["typical"]["estimate"].as_f64().unwrap();

        let data = Data {
            function,
            crate_,
            variant,
            case,
            time,
        };

        ans.push(data);
    }
    Ok(ans)
}

struct TableData {
    title: String,
    rows: Vec<String>,
    cols: Vec<String>,
    content: Vec<Vec<OrderedFloat<f64>>>,
    is_throughput: bool,
}

fn gather_table_data(data: &[Data]) -> Result<Vec<TableData>> {
    let functions: &[&str] = &[
        "base64-check",
        "base64-decode",
        "base64-encode",
        "hex-check",
        "hex-decode",
        "hex-encode",
        "base32-check",
        "base32-decode",
        "base32-encode",
        "uuid-format",
        "uuid-parse",
    ];

    let mut tables: HashMap<String, TableData> = default();

    for d in data {
        let t = tables.entry(d.function.to_owned()).or_insert_with(|| TableData {
            title: d.function.to_owned(),
            rows: default(),
            cols: default(),
            content: default(),
            is_throughput: true,
        });

        let row = format!("{}/{}", d.crate_, d.variant);
        let col = d.case.clone();

        let content = match d.case.parse::<u64>() {
            Ok(count) => {
                let gi = 1024.0 * 1024.0 * 1024.0;
                count as f64 / d.time * 1e9 / gi // GiB/s
            }
            Err(_) => {
                t.is_throughput = false;
                d.time // ns
            }
        };

        let i = match t.rows.iter().position(|r| r == &row) {
            Some(i) => i,
            None => {
                t.rows.push(row);
                t.content.push(default());
                t.rows.len() - 1
            }
        };

        if t.cols.contains(&col).not() {
            t.cols.push(col.clone());
        }

        t.content[i].push(OrderedFloat(content));
    }

    Ok(functions.iter().filter_map(|f| tables.remove(*f)).collect())
}

fn render_table(table_data: &[TableData]) -> Result<Vec<(String, Table)>> {
    let mut ans: Vec<(String, Table)> = default();

    for t in table_data {
        let title = if t.is_throughput {
            format!("{} (GiB/s)", t.title)
        } else {
            format!("{} (ns)", t.title)
        };

        let mut b: tabled::builder::Builder = default();

        {
            let mut record: Vec<String> = vec![default()];
            record.extend(t.cols.iter().cloned());
            b.add_record(record);
        }

        let bold_row: Vec<usize> = map_collect(0..t.cols.len(), |j| {
            let iter = t.content.iter().map(|r| r[j]).enumerate();
            if t.is_throughput {
                iter.max_by_key(|&(_, f)| f).unwrap().0
            } else {
                iter.min_by_key(|&(_, f)| f).unwrap().0
            }
        });

        for (i, row) in t.rows.iter().enumerate() {
            let mut record: Vec<String> = vec![row.clone()];

            #[allow(clippy::needless_range_loop)]
            for j in 0..t.cols.len() {
                let f = t.content[i][j];
                let needs_bold = i == bold_row[j];
                if needs_bold {
                    record.push(format!("**{:.3}**", f));
                } else {
                    record.push(format!("{:.3}", f));
                }
            }

            b.add_record(record);
        }

        let table = b.build().with(Style::markdown());
        ans.push((title, table));
    }

    Ok(ans)
}

fn main() -> anyhow::Result<()> {
    let path = env::args().nth(1).unwrap();
    let messages = read_messages(&path)?;
    let data = extract_data(&messages)?;
    let table_data = gather_table_data(&data)?;
    for (title, table) in render_table(&table_data)? {
        println!("#### {}", title);
        println!();
        println!("{}", table);
        println!();
    }
    Ok(())
}
