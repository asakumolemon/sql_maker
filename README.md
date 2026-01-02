# SQL Maker - Excel转SQL批量生成工具

一个基于Rust开发的命令行工具，用于从Excel文件中提取指定列的数据，并根据模板批量生成SQL语句。

## 功能特性

- 📊 **读取Excel文件** - 支持.xlsx格式，自动处理多工作表
- 🎯 **指定列提取** - 支持列字母（A, B, C, AA, AB等）
- 📝 **SQL模板** - 使用`{value}`作为占位符，灵活定制SQL语句
- ⚡ **批量生成** - 为每个非空单元格生成一条SQL语句
- 💾 **多种输出** - 支持控制台输出或保存到文件
- 🔒 **SQL安全** - 自动转义特殊字符，防止SQL注入
- 🚀 **高性能** - Rust编写，处理大型Excel文件速度快

## 安装

### 从源码构建

确保已安装Rust环境（1.70或更高版本）：

```bash
# 克隆项目
git clone <repository-url>
cd sql_maker

# 构建Release版本
cargo build --release

# 可执行文件位置
target/release/sql_maker.exe  # Windows
target/release/sql_maker      # Linux/macOS
```

### 添加到系统PATH（可选）

```bash
# Windows
set PATH=%PATH%;D:\path\to\sql_maker\target\release

# Linux/macOS
export PATH=$PATH:/path/to/sql_maker/target/release
```

## 使用方法

### 基本命令格式

```bash
sql_maker -f <Excel文件> -c <列名> -t <SQL模板> [选项]
```

### 参数说明

| 参数 | 短选项 | 说明 | 是否必需 |
|------|--------|------|----------|
| `--file` | `-f` | Excel文件路径 | ✓ |
| `--column` | `-c` | 列名称（如A, B, C, AA） | ✓ |
| `--template` | `-t` | SQL模板，使用`{value}`作为占位符 | ✓ |
| `--sheet` | `-s` | 工作表名称（默认第一个工作表） | ✗ |
| `--start-row` | `-r` | 起始行号，从1开始（默认2） | ✗ |
| `--output` | `-o` | 输出文件路径（默认控制台输出） | ✗ |
| `--skip-empty` | - | 跳过空单元格（默认true） | ✗ |
| `--help` | `-h` | 显示帮助信息 | ✗ |
| `--version` | `-V` | 显示版本信息 | ✗ |

### 使用示例

#### 示例1：生成INSERT语句

假设Excel文件`users.xlsx`的A列包含用户名：

```bash
sql_maker -f "users.xlsx" -c "A" -t "INSERT INTO users (name) VALUES ('{value}');"
```

输出：
```sql
INSERT INTO users (name) VALUES ('张三');
INSERT INTO users (name) VALUES ('李四');
INSERT INTO users (name) VALUES ('王五');
```

#### 示例2：生成UPDATE语句并保存到文件

```bash
sql_maker -f "data.xlsx" -c "B" -r 3 -t "UPDATE products SET status='active' WHERE id='{value}';" -o "update.sql"
```

#### 示例3：指定工作表

```bash
sql_maker -f "report.xlsx" -s "Sheet2" -c "C" -t "DELETE FROM logs WHERE user_id='{value}';"
```

#### 示例4：从第1行开始读取（包含标题）

```bash
sql_maker -f "data.xlsx" -c "A" -r 1 -t "SELECT * FROM table WHERE id={value};"
```

## SQL模板语法

### 基本占位符

使用`{value}`作为Excel单元格值的占位符：

```bash
# 字符串值（自动添加引号）
-t "INSERT INTO users (name) VALUES ('{value}')"

# 数字值
-t "UPDATE stats SET count = {value} WHERE id = 1"

# 多字段
-t "INSERT INTO products (name, code) VALUES ('{value}', 'PROD_{value}')"
```

### 特殊字符处理

程序会自动处理SQL特殊字符：

- 单引号`'` → `''` （SQL标准转义）
- 反斜杠`\` → `\\`

示例：
- `O'Reilly` → `O''Reilly`
- `C:\path` → `C:\\path`

## 实际应用场景

### 场景1：批量插入用户数据

Excel内容（A列）：
```
用户名
--------
张三
李四
王五
```

命令：
```bash
sql_maker -f "users.xlsx" -c "A" -r 2 -t "INSERT INTO users (username, created_at) VALUES ('{value}', NOW());"
```

### 场景2：批量更新状态

Excel内容（B列）：
```
订单ID
--------
ORD-001
ORD-002
ORD-003
```

命令：
```bash
sql_maker -f "orders.xlsx" -c "B" -r 2 -t "UPDATE orders SET status = 'completed' WHERE order_id = '{value}';" -o "update_orders.sql"
```

### 场景3：批量删除记录

Excel内容（C列）：
```
用户ID
--------
1001
1002
1003
```

命令：
```bash
sql_maker -f "cleanup.xlsx" -c "C" -r 2 -t "DELETE FROM user_sessions WHERE user_id = {value};"
```

## 高级用法

### 跳过标题行

默认从第2行开始（跳过第1行标题），可通过`-r`参数调整：

```bash
# 从第3行开始（跳过2行）
sql_maker -f "data.xlsx" -c "A" -r 3 -t "..."

# 从第1行开始（包含标题行）
sql_maker -f "data.xlsx" -c "A" -r 1 -t "..."
```

### 处理空单元格

默认跳过空单元格，可通过`--skip-empty false`保留：

```bash
# 保留空值（生成空字符串）
sql_maker -f "data.xlsx" -c "A" -t "INSERT INTO table (col) VALUES ('{value}');" --skip-empty false
```

### 多列字母支持

支持超过Z列的双字母列名：

```bash
# AA列
sql_maker -f "data.xlsx" -c "AA" -t "..."

# AB列
sql_maker -f "data.xlsx" -c "AB" -t "..."
```

## 错误处理

### 常见错误及解决方案

| 错误信息 | 原因 | 解决方案 |
|---------|------|----------|
| `工作表 'Sheet1' 不存在` | 指定的工作表名称错误 | 使用`-s`指定正确的工作表名，或省略使用第一个 |
| `没有从指定列中找到任何数据` | 列为空或起始行设置错误 | 检查列字母和起始行号`-r` |
| `无效的列名称: 1` | 列格式错误 | 使用字母（A, B, C）而不是数字 |

## 性能提示

- 对于大型Excel文件（10万+行），建议使用`-o`参数输出到文件，而不是控制台
- Release版本比Debug版本快3-5倍，生产环境务必使用`cargo build --release`
- 跳过空单元格（默认）可以提高处理速度

## 开发

### 运行测试

```bash
cargo test
```

### 构建Debug版本

```bash
cargo build
```

### 查看帮助

```bash
cargo run -- --help
```

## 技术栈

- **语言**: Rust
- **Excel解析**: [calamine](https://github.com/tafia/calamine)
- **命令行解析**: [clap](https://github.com/clap-rs/clap)

## 许可证

MIT License

## 贡献

欢迎提交Issue和Pull Request！

## 版本历史

### v0.1.0
- 初始版本
- 支持Excel读取和SQL生成
- 命令行参数解析
- 支持输出到文件或控制台