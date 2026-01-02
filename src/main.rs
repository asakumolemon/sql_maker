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

    /// 列名称，可指定多个（如：-c A -c B -c C）
    #[arg(short, long, required = true)]
    column: Vec<String>,

    /// 起始行号（从1开始）
    #[arg(short = 'r', long, default_value = "2")]
    start_row: usize,

    /// SQL模板，使用{1},{2}...作为占位符，或使用{列名}如{A},{B}
    #[arg(short, long)]
    template: String,

    /// 输出文件路径（如果不指定则输出到控制台）
    #[arg(short, long)]
    output: Option<String>,

    /// 每个输出文件包含的SQL条数（分片功能）
    #[arg(short = 'b', long)]
    batch_size: Option<usize>,

    /// 每条SQL语句使用的行数（批量模式）
    #[arg(short = 'n', long)]
    rows_per_sql: Option<usize>,

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

    // 转换多个列字母为索引
    let col_indices: Vec<usize> = args.column
        .iter()
        .map(|col| column_letter_to_index(col))
        .collect::<Result<Vec<_>, _>>()?;

    // 收集数据 - 每行是一个Vec<String>
    let mut rows_data = Vec::new();
    let start_row = args.start_row.saturating_sub(1); // 转换为0-based索引

    for (row_idx, row) in range.rows().enumerate() {
        if row_idx < start_row {
            continue;
        }

        // 从当前行提取所有指定列的值
        let mut row_values = Vec::new();
        let mut has_non_empty = false;

        for &col_idx in &col_indices {
            if let Some(cell) = row.get(col_idx) {
                let value = cell.to_string().trim().to_string();
                if !value.is_empty() {
                    has_non_empty = true;
                }
                row_values.push(value);
            } else {
                row_values.push(String::new());
            }
        }

        // 如果至少有一个非空值，或者设置了不跳过空行
        if has_non_empty || !args.skip_empty {
            rows_data.push(row_values);
        }
    }

    if rows_data.is_empty() {
        return Err("没有从指定列中找到任何数据".into());
    }

    println!("📊 从Excel中读取到 {} 行数据", rows_data.len());
    println!("📋 每行包含 {} 个列值", args.column.len());

    // 生成SQL语句
    let sql_statements: Vec<String> = if let Some(rows_per_sql) = args.rows_per_sql {
        // 批量模式：每条SQL使用多行数据
        if rows_per_sql == 0 {
            return Err("每条SQL的行数不能为0".into());
        }
        generate_batch_sql(&args.template, &rows_data, &args.column, rows_per_sql)
    } else {
        // 普通模式：每条SQL使用一行数据
        rows_data
            .iter()
            .map(|row_values| generate_sql_multi(&args.template, row_values, &args.column))
            .collect()
    };

    println!("📝 生成了 {} 条SQL语句", sql_statements.len());

    // 输出结果
    if let Some(output_path) = &args.output {
        if let Some(batch_size) = args.batch_size {
            // 分片输出到多个文件
            output_in_batches(&sql_statements, output_path, batch_size)?;
        } else {
            // 单文件输出
            let mut file = File::create(output_path)?;
            for (i, sql) in sql_statements.iter().enumerate() {
                writeln!(file, "{}", sql)?;
                if i < sql_statements.len() - 1 {
                    writeln!(file)?;
                }
            }
            println!("💾 SQL语句已保存到: {}", output_path);
        }
    } else {
        // 控制台输出
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

/// 生成SQL语句，替换模板中的{value}占位符（单列模式）
fn generate_sql(template: &str, value: &str) -> String {
    // 对SQL字符串进行转义，防止SQL注入
    let escaped_value = escape_sql_string(value);
    template.replace("{value}", &escaped_value)
}

/// 生成SQL语句，替换模板中的{1},{2}...或{A},{B}...占位符（多列模式）
fn generate_sql_multi(template: &str, row_values: &[String], column_names: &[String]) -> String {
    let mut result = template.to_string();

    // 替换{1}, {2}, {3}...占位符（基于列的顺序）
    for (idx, value) in row_values.iter().enumerate() {
        let placeholder = format!("{{{}}}", idx + 1);
        let escaped_value = escape_sql_string(value);
        result = result.replace(&placeholder, &escaped_value);
    }

    // 替换{A}, {B}, {C}...占位符（基于列名）
    for (col_name, value) in column_names.iter().zip(row_values.iter()) {
        let placeholder = format!("{{{}}}", col_name.to_uppercase());
        let escaped_value = escape_sql_string(value);
        result = result.replace(&placeholder, &escaped_value);
    }

    // 向后兼容：如果只有一个列，也替换{value}
    if row_values.len() == 1 {
        let escaped_value = escape_sql_string(&row_values[0]);
        result = result.replace("{value}", &escaped_value);
    }

    result
}

/// 转义SQL字符串中的特殊字符
fn escape_sql_string(value: &str) -> String {
    value.replace("'", "''")
        .replace("\\", "\\\\")
}

/// 生成批量SQL语句，将多行数据合并到一条SQL中
fn generate_batch_sql(template: &str, rows_data: &[Vec<String>], column_names: &[String], rows_per_sql: usize) -> Vec<String> {
    let mut sql_statements = Vec::new();

    // 检查模板类型：IN子句模式或VALUES批量插入模式
    let is_in_clause_mode = template.contains("{values}");
    let is_values_batch_mode = template.contains("{@row}");

    if is_in_clause_mode {
        // IN子句模式：如 UPDATE ... WHERE id IN ({values})
        sql_statements.extend(generate_in_clause_sql(template, rows_data, rows_per_sql, column_names));
    } else if is_values_batch_mode {
        // VALUES批量插入模式：如 INSERT ... VALUES {@row}
        sql_statements.extend(generate_values_batch_sql(template, rows_data, rows_per_sql, column_names));
    } else {
        // 普通批量模式：将多行数据依次替换到模板中
        sql_statements.extend(generate_simple_batch_sql(template, rows_data, rows_per_sql, column_names));
    }

    sql_statements
}

/// IN子句模式：生成WHERE id IN (v1, v2, v3...)格式的SQL
fn generate_in_clause_sql(template: &str, rows_data: &[Vec<String>], rows_per_sql: usize, column_names: &[String]) -> Vec<String> {
    let mut sql_statements = Vec::new();

    // 默认使用第一列的值作为IN子句的值
    let col_idx = 0;

    for chunk in rows_data.chunks(rows_per_sql) {
        let values: Vec<String> = chunk.iter()
            .filter_map(|row| row.get(col_idx).map(|v| escape_sql_string(v)))
            .map(|v| format!("'{}'", v))
            .collect();

        if !values.is_empty() {
            let values_str = values.join(", ");
            let sql = template.replace("{values}", &values_str);
            sql_statements.push(sql);
        }
    }

    sql_statements
}

/// VALUES批量插入模式：生成INSERT ... VALUES (a1,b1), (a2,b2)...格式的SQL
fn generate_values_batch_sql(template: &str, rows_data: &[Vec<String>], rows_per_sql: usize, column_names: &[String]) -> Vec<String> {
    let mut sql_statements = Vec::new();

    // 提取{@row}模板部分
    let row_template = if let Some(start) = template.find("{@row}") {
        if let Some(end) = template[start..].find('}') {
            &template[start..start + end + 1]
        } else {
            "{@row}"
        }
    } else {
        "{@row}"
    };

    for chunk in rows_data.chunks(rows_per_sql) {
        let mut all_rows = Vec::new();

        for row_values in chunk {
            let mut row_sql = row_template.replace("{@row}", "");

            // 替换数字索引占位符 {1}, {2}...
            for (idx, value) in row_values.iter().enumerate() {
                let placeholder = format!("{{{}}}", idx + 1);
                let escaped_value = escape_sql_string(value);
                row_sql = row_sql.replace(&placeholder, &format!("'{}'", escaped_value));
            }

            // 替换列名占位符 {A}, {B}...
            for (col_name, value) in column_names.iter().zip(row_values.iter()) {
                let placeholder = format!("{{{}}}", col_name.to_uppercase());
                let escaped_value = escape_sql_string(value);
                row_sql = row_sql.replace(&placeholder, &format!("'{}'", escaped_value));
            }

            // 替换单列占位符 {value}
            if row_values.len() == 1 {
                let escaped_value = escape_sql_string(&row_values[0]);
                row_sql = row_sql.replace("{value}", &format!("'{}'", escaped_value));
            }

            // 如果没有替换任何占位符，使用默认格式
            if row_sql == row_template.replace("{@row}", "") {
                let values: Vec<String> = row_values.iter()
                    .map(|v| format!("'{}'", escape_sql_string(v)))
                    .collect();
                row_sql = format!("({})", values.join(", "));
            }

            all_rows.push(row_sql);
        }

        if !all_rows.is_empty() {
            let rows_str = all_rows.join(", ");
            let sql = template.replace(row_template, &rows_str);
            sql_statements.push(sql);
        }
    }

    sql_statements
}

/// 简单批量模式：将多行数据依次替换到模板中
fn generate_simple_batch_sql(template: &str, rows_data: &[Vec<String>], rows_per_sql: usize, column_names: &[String]) -> Vec<String> {
    let mut sql_statements = Vec::new();

    for chunk in rows_data.chunks(rows_per_sql) {
        let mut sql = template.to_string();

        // 替换数字索引的多行占位符 {#1}, {#2}...
        for (idx, _) in column_names.iter().enumerate() {
            let placeholder = format!("{{#{}}}", idx + 1);
            if sql.contains(&placeholder) {
                let values: Vec<String> = chunk.iter()
                    .filter_map(|row| row.get(idx).map(|v| format!("'{}'", escape_sql_string(v))))
                    .collect();
                let values_str = values.join(", ");
                sql = sql.replace(&placeholder, &values_str);
            }
        }

        // 替换列名的多行占位符 {#A}, {#B}...
        for (idx, col_name) in column_names.iter().enumerate() {
            let placeholder = format!("{{#{}}}", col_name.to_uppercase());
            if sql.contains(&placeholder) {
                let values: Vec<String> = chunk.iter()
                    .filter_map(|row| row.get(idx).map(|v| format!("'{}'", escape_sql_string(v))))
                    .collect();
                let values_str = values.join(", ");
                sql = sql.replace(&placeholder, &values_str);
            }
        }

        sql_statements.push(sql);
    }

    sql_statements
}

/// 将SQL语句分片输出到多个文件
fn output_in_batches(sql_statements: &[String], output_path: &str, batch_size: usize) -> Result<(), Box<dyn std::error::Error>> {
    if batch_size == 0 {
        return Err("分片大小不能为0".into());
    }

    // 解析输出路径，分离目录、文件名和扩展名
    let path = std::path::Path::new(output_path);
    let directory = path.parent().unwrap_or(std::path::Path::new(""));
    let file_name = path.file_name()
        .ok_or("无效的文件路径")?
        .to_str()
        .ok_or("无效的文件名")?;

    // 分离文件名和扩展名
    let (name_without_ext, ext) = if let Some(pos) = file_name.rfind('.') {
        (&file_name[..pos], &file_name[pos..])
    } else {
        (file_name, ".sql")
    };

    // 计算需要多少个文件
    let total_sql = sql_statements.len();
    let file_count = (total_sql + batch_size - 1) / batch_size;

    println!("💾 正在将 {} 条SQL语句分片保存到 {} 个文件中（每文件{}条）", total_sql, file_count, batch_size);

    // 分片写入文件
    for file_idx in 0..file_count {
        let start_idx = file_idx * batch_size;
        let end_idx = std::cmp::min(start_idx + batch_size, total_sql);
        let batch = &sql_statements[start_idx..end_idx];

        // 生成文件名，如：output_1.sql, output_2.sql
        let file_name = format!("{}_{}{}", name_without_ext, file_idx + 1, ext);
        let output_file_path = if directory.as_os_str().is_empty() {
            file_name.clone()
        } else {
            format!("{}\\{}", directory.display(), file_name)
        };

        let mut file = File::create(&output_file_path)?;

        for (i, sql) in batch.iter().enumerate() {
            writeln!(file, "{}", sql)?;
            if i < batch.len() - 1 {
                writeln!(file)?;
            }
        }

        println!("  ✓ 文件 {}: {} 条SQL语句", file_name, batch.len());
    }

    Ok(())
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
    fn test_generate_sql_multi_numeric() {
        let template = "INSERT INTO users (name, age) VALUES ('{1}', {2});";
        let row_values = vec!["张三".to_string(), "25".to_string()];
        let column_names = vec!["A".to_string(), "B".to_string()];
        assert_eq!(
            generate_sql_multi(template, &row_values, &column_names),
            "INSERT INTO users (name, age) VALUES ('张三', 25);"
        );
    }

    #[test]
    fn test_generate_sql_multi_column_names() {
        let template = "INSERT INTO users (name, age) VALUES ('{A}', {B});";
        let row_values = vec!["李四".to_string(), "30".to_string()];
        let column_names = vec!["A".to_string(), "B".to_string()];
        assert_eq!(
            generate_sql_multi(template, &row_values, &column_names),
            "INSERT INTO users (name, age) VALUES ('李四', 30);"
        );
    }

    #[test]
    fn test_generate_sql_multi_backward_compatible() {
        let template = "INSERT INTO users (name) VALUES ('{value}');";
        let row_values = vec!["王五".to_string()];
        let column_names = vec!["A".to_string()];
        assert_eq!(
            generate_sql_multi(template, &row_values, &column_names),
            "INSERT INTO users (name) VALUES ('王五');"
        );
    }

    #[test]
    fn test_escape_sql_string() {
        assert_eq!(escape_sql_string("O'Reilly"), "O''Reilly");
        assert_eq!(escape_sql_string("C:\\path"), "C:\\\\path");
    }

    #[test]
    fn test_generate_batch_sql_in_clause() {
        let template = "UPDATE users SET status='active' WHERE id IN ({values});";
        let rows_data = vec![
            vec!["1".to_string(), "Alice".to_string()],
            vec!["2".to_string(), "Bob".to_string()],
            vec!["3".to_string(), "Charlie".to_string()],
        ];
        let column_names = vec!["A".to_string(), "B".to_string()];
        let rows_per_sql = 2;

        let sql_statements = generate_batch_sql(template, &rows_data, &column_names, rows_per_sql);

        assert_eq!(sql_statements.len(), 2);
        assert_eq!(
            sql_statements[0],
            "UPDATE users SET status='active' WHERE id IN ('1', '2');"
        );
        assert_eq!(
            sql_statements[1],
            "UPDATE users SET status='active' WHERE id IN ('3');"
        );
    }

    #[test]
    fn test_generate_batch_sql_values_batch() {
        let template = "INSERT INTO users (id, name) VALUES {@row};";
        let rows_data = vec![
            vec!["1".to_string(), "Alice".to_string()],
            vec!["2".to_string(), "Bob".to_string()],
            vec!["3".to_string(), "Charlie".to_string()],
        ];
        let column_names = vec!["A".to_string(), "B".to_string()];
        let rows_per_sql = 2;

        let sql_statements = generate_batch_sql(template, &rows_data, &column_names, rows_per_sql);

        assert_eq!(sql_statements.len(), 2);
        assert_eq!(
            sql_statements[0],
            "INSERT INTO users (id, name) VALUES ('1', 'Alice'), ('2', 'Bob');"
        );
        assert_eq!(
            sql_statements[1],
            "INSERT INTO users (id, name) VALUES ('3', 'Charlie');"
        );
    }

    #[test]
    fn test_generate_batch_sql_simple_batch() {
        let template = "UPDATE users SET status='active' WHERE id IN ({#A});";
        let rows_data = vec![
            vec!["1".to_string()],
            vec!["2".to_string()],
            vec!["3".to_string()],
            vec!["4".to_string()],
        ];
        let column_names = vec!["A".to_string()];
        let rows_per_sql = 3;

        let sql_statements = generate_batch_sql(template, &rows_data, &column_names, rows_per_sql);

        assert_eq!(sql_statements.len(), 2);
        assert_eq!(
            sql_statements[0],
            "UPDATE users SET status='active' WHERE id IN ('1', '2', '3');"
        );
        assert_eq!(
            sql_statements[1],
            "UPDATE users SET status='active' WHERE id IN ('4');"
        );
    }
}
