use clap::Parser;
use calamine::{Reader, Xlsx, open_workbook};
use std::fs::File;
use std::io::Write;

/// Excel to SQL批量生成工具
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Excel文件路径
    #[arg(short, long)]
    file: String,

    /// 工作表名称（默认为第一个工作表）
    #[arg(short, long, default_value = "")]
    sheet: String,

    /// 列名称（如：A, B, C）
    #[arg(short, long)]
    column: String,

    /// 起始行号（从1开始）
    #[arg(short = 'r', long, default_value = "2")]
    start_row: usize,

    /// SQL模板，使用{value}作为占位符
    #[arg(short, long)]
    template: String,

    /// 输出文件路径（如果不指定则输出到控制台）
    #[arg(short, long)]
    output: Option<String>,

    /// 跳过空单元格
    #[arg(long, default_value = "true")]
    skip_empty: bool,
}

fn main() {
    let args = Args::parse();

    match run(&args) {
        Ok(count) => {
            println!("✅ 成功生成 {} 条SQL语句", count);
        }
        Err(e) => {
            eprintln!("❌ 错误: {}", e);
            std::process::exit(1);
        }
    }
}

fn run(args: &Args) -> Result<usize, Box<dyn std::error::Error>> {
    // 打开Excel文件
    let mut workbook: Xlsx<_> = open_workbook(&args.file)?;

    // 获取工作表
    let sheet_name = if args.sheet.is_empty() {
        // 如果没有指定工作表，使用第一个
        workbook.sheet_names().first()
            .ok_or("Excel文件中没有找到工作表")?
            .to_string()
    } else {
        args.sheet.clone()
    };

    let range = workbook.worksheet_range(&sheet_name)
        .map_err(|e| format!("读取工作表失败: {}", e))?;

    // 转换列字母为索引（A=0, B=1, ...）
    let col_idx = column_letter_to_index(&args.column)?;

    // 收集数据
    let mut values = Vec::new();
    let start_row = args.start_row.saturating_sub(1); // 转换为0-based索引

    for (row_idx, row) in range.rows().enumerate() {
        if row_idx < start_row {
            continue;
        }

        if let Some(cell) = row.get(col_idx) {
            let value = cell.to_string().trim().to_string();
            if !value.is_empty() || !args.skip_empty {
                values.push(value);
            }
        }
    }

    if values.is_empty() {
        return Err("没有从指定列中找到任何数据".into());
    }

    println!("📊 从Excel中读取到 {} 条数据", values.len());

    // 生成SQL语句
    let sql_statements: Vec<String> = values
        .iter()
        .map(|value| generate_sql(&args.template, value))
        .collect();

    // 输出结果
    if let Some(output_path) = &args.output {
        let mut file = File::create(output_path)?;
        for (i, sql) in sql_statements.iter().enumerate() {
            writeln!(file, "{}", sql)?;
            if i < sql_statements.len() - 1 {
                writeln!(file)?;
            }
        }
        println!("💾 SQL语句已保存到: {}", output_path);
    } else {
        println!("\n========== 生成的SQL语句 ==========\n");
        for (i, sql) in sql_statements.iter().enumerate() {
            println!("-- SQL {}:", i + 1);
            println!("{}", sql);
            println!();
        }
    }

    Ok(sql_statements.len())
}

/// 将列字母转换为0-based索引（A=0, B=1, ..., Z=25, AA=26, ...）
fn column_letter_to_index(column: &str) -> Result<usize, String> {
    let column = column.to_uppercase();
    let mut index = 0;

    for (i, ch) in column.chars().rev().enumerate() {
        if !ch.is_ascii_alphabetic() {
            return Err(format!("无效的列名称: {}", column));
        }

        let value = (ch as usize) - ('A' as usize) + 1;
        index += value * 26_usize.pow(i as u32);
    }

    Ok(index.saturating_sub(1))
}

/// 生成SQL语句，替换模板中的{value}占位符
fn generate_sql(template: &str, value: &str) -> String {
    // 对SQL字符串进行转义，防止SQL注入
    let escaped_value = escape_sql_string(value);
    template.replace("{value}", &escaped_value)
}

/// 转义SQL字符串中的特殊字符
fn escape_sql_string(value: &str) -> String {
    value.replace("'", "''")
        .replace("\\", "\\\\")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_letter_to_index() {
        assert_eq!(column_letter_to_index("A").unwrap(), 0);
        assert_eq!(column_letter_to_index("B").unwrap(), 1);
        assert_eq!(column_letter_to_index("Z").unwrap(), 25);
        assert_eq!(column_letter_to_index("AA").unwrap(), 26);
        assert_eq!(column_letter_to_index("AB").unwrap(), 27);
    }

    #[test]
    fn test_generate_sql() {
        let template = "INSERT INTO users (name) VALUES ('{value}');";
        assert_eq!(
            generate_sql(template, "张三"),
            "INSERT INTO users (name) VALUES ('张三');"
        );
    }

    #[test]
    fn test_escape_sql_string() {
        assert_eq!(escape_sql_string("O'Reilly"), "O''Reilly");
        assert_eq!(escape_sql_string("C:\\path"), "C:\\\\path");
    }
}
