# SQL Maker - Excel转SQL批量生成工具

一个基于Rust开发的命令行工具，用于从Excel文件中提取指定列的数据，并根据模板批量生成SQL语句。

## 功能特性

- 📊 **读取Excel文件** - 支持.xlsx格式，自动处理多工作表
- 🎯 **指定列提取** - 支持单列或多列（A, B, C, AA, AB等）
- 📝 **SQL模板** - 使用`{1},{2}`或`{A},{B}`作为占位符，灵活定制SQL语句
- ⚡ **批量生成** - 为每行数据生成一条SQL语句，支持多值插入
- 🔢 **批量SQL模式** - 支持将多行数据合并到一条SQL（IN子句、批量INSERT等）
- 💾 **多种输出** - 支持控制台输出或保存到文件
- 📁 **分片功能** - 支持将大量SQL语句分片保存到多个文件
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
| `--column` | `-c` | 列名称，可指定多个（如：-c A -c B -c C） | ✓ |
| `--template` | `-t` | SQL模板，使用`{1},{2}...`或`{A},{B}...`作为占位符 | ✓ |
| `--sheet` | `-s` | 工作表名称（默认第一个工作表） | ✗ |
| `--start-row` | `-r` | 起始行号，从1开始（默认2） | ✗ |
| `--output` | `-o` | 输出文件路径（默认控制台输出） | ✗ |
| `--batch-size` | `-b` | 每个输出文件包含的SQL条数（分片功能） | ✗ |
| `--rows-per-sql` | `-n` | 每条SQL语句使用的行数（批量模式） | ✗ |
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

#### 示例5：多列数据插入（使用数字索引）

```bash
# Excel: A列=name, B列=age, C列=email
sql_maker -f "users.xlsx" -c A -c B -c C \
  -t "INSERT INTO users (name, age, email) VALUES ('{1}', {2}, '{3}')" \
  -o "insert_users.sql"
```

#### 示例6：多列数据插入（使用列名）

```bash
# Excel: A列=product_name, B列=price, D列=category
sql_maker -f "products.xlsx" -c A -c B -c D \
  -t "INSERT INTO products (name, price, category) VALUES ('{A}', {B}, '{D}')"
```

#### 示例7：混合使用列值

```bash
# Excel: A列=first_name, B列=last_name, C列=department
# 生成包含完整姓名和邮箱的INSERT语句
sql_maker -f "employees.xlsx" -c A -c B -c C -r 2 \
  -t "INSERT INTO employees (first_name, last_name, full_name, email, department) VALUES ('{A}', '{B}', '{A} {B}', lower('{A}.{B}@company.com'), '{C}')"
```

#### 示例8：分片输出大量SQL语句

```bash
# 假设Excel有2500行数据，每文件保存1000条SQL
# 将生成：output_1.sql (1000条), output_2.sql (1000条), output_3.sql (500条)
sql_maker -f "large_dataset.xlsx" -c A -c B \
  -t "INSERT INTO large_table (col1, col2) VALUES ('{1}', '{2}');" \
  -o "output.sql" \
  -b 1000
```

## SQL模板语法

### 单列模式（向后兼容）

使用`{value}`作为Excel单元格值的占位符：

```bash
# 字符串值（自动添加引号）
-t "INSERT INTO users (name) VALUES ('{value}')"

# 数字值
-t "UPDATE stats SET count = {value} WHERE id = 1"
```

### 多列模式

当指定多个列时，支持两种占位符格式：

#### 格式1：使用数字索引 `{1}`, `{2}`, `{3}`...

按照列的顺序匹配（从1开始）：

```bash
# 从A列和B列提取数据
sql_maker -f "users.xlsx" -c A -c B -t "INSERT INTO users (name, age) VALUES ('{1}', {2})"

# 从A、B、C三列提取数据
sql_maker -f "products.xlsx" -c A -c B -c C -t "INSERT INTO products (name, price, stock) VALUES ('{1}', {2}, {3})"
```

#### 格式2：使用列名 `{A}`, `{B}`, `{C}`...

直接使用列字母作为占位符：

```bash
# 从A列和B列提取数据
sql_maker -f "users.xlsx" -c A -c B -t "INSERT INTO users (name, age) VALUES ('{A}', {B})"

# 混合使用不同格式
sql_maker -f "data.xlsx" -c A -c B -c D -t "INSERT INTO table (col1, col2, col3) VALUES ('{A}', {B}, {D})"
```

#### 混合使用示例

```bash
# 在模板中组合使用列值
sql_maker -f "employees.xlsx" -c A -c B -c C \
  -t "INSERT INTO employees (id, full_name, email) VALUES ({1}, '{A} {B}', lower('{A}.{B}@company.com'))"
```

### 特殊字符处理

程序会自动处理SQL特殊字符：

- 单引号`'` → `''` （SQL标准转义）
- 反斜杠`\` → `\\`

示例：
- `O'Reilly` → `O''Reilly`
- `C:\path` → `C:\\path`

## 批量SQL模式（高级功能）

当需要生成包含多行数据的批量SQL语句时（如IN子句、批量INSERT等），可以使用`-n`参数指定每条SQL语句使用的行数。

### 模式1：IN子句模式

使用`{values}`占位符，将多行数据的第一列值合并为逗号分隔的列表：

```bash
# Excel数据（A列）：1, 2, 3, 4, 5
# 每3行生成一条SQL
sql_maker -f "ids.xlsx" -c A -r 2 -t "UPDATE users SET status='active' WHERE id IN ({values});" -n 3
```

输出：
```sql
UPDATE users SET status='active' WHERE id IN ('1', '2', '3');
UPDATE users SET status='active' WHERE id IN ('4', '5');
```

### 模式2：VALUES批量插入模式

使用`{@row}`占位符，生成INSERT语句的多个VALUES：

```bash
# Excel数据：A列=id, B列=name
# 每2行生成一条INSERT语句
sql_maker -f "users.xlsx" -c A -c B -r 2 \
  -t "INSERT INTO users (id, name) VALUES {@row};" \
  -n 2
```

输出：
```sql
INSERT INTO users (id, name) VALUES ('1', 'Alice'), ('2', 'Bob');
INSERT INTO users (id, name) VALUES ('3', 'Charlie');
```

### 模式3：多行占位符模式

使用`{#1}`, `{#2}`...或`{#A}`, `{#B}`...占位符，将多行数据的某一列合并：

```bash
# Excel数据：A列=user_id
# 每100行生成一条UPDATE语句
sql_maker -f "users.xlsx" -c A -r 2 \
  -t "UPDATE users SET last_login=NOW() WHERE user_id IN ({#A});" \
  -n 100
```

输出：
```sql
UPDATE users SET last_login=NOW() WHERE user_id IN ('1', '2', '3', ..., '100');
UPDATE users SET last_login=NOW() WHERE user_id IN ('101', '102', '103', ..., '200');
```

### 批量模式与分片功能结合使用

批量模式可以与分片功能（`-b`）结合，实现更灵活的文件管理：

```bash
# 生成批量INSERT语句，每条SQL包含500行数据
# 每10条SQL保存到一个文件
sql_maker -f "large_dataset.xlsx" -c A -c B -c C -r 2 \
  -t "INSERT INTO products (id, name, price) VALUES {@row};" \
  -n 500 \
  -o "products.sql" \
  -b 10
```

这将生成：
- 每条SQL包含500个VALUES（500行数据）
- 每10条SQL保存到一个文件（每个文件约5000行数据）
- 自动创建 products_1.sql, products_2.sql...

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

### 场景4：批量插入多列数据

Excel内容：
```
A列        | B列  | C列
---------------------------
产品名称   | 价格 | 库存
iPhone    | 6999 | 100
MacBook   | 9999 | 50
AirPods   | 1299 | 200
```

命令（使用数字索引）：
```bash
sql_maker -f "products.xlsx" -c A -c B -c C -r 2 \
  -t "INSERT INTO products (name, price, stock) VALUES ('{1}', {2}, {3});" \
  -o "insert_products.sql"
```

命令（使用列名）：
```bash
sql_maker -f "products.xlsx" -c A -c B -c C -r 2 \
  -t "INSERT INTO products (name, price, stock) VALUES ('{A}', {B}, {C});" \
  -o "insert_products.sql"
```

### 场景5：生成复杂INSERT语句

Excel内容：
```
A列(名) | B列(姓) | C列(部门)
-------------------------------
张      | 三      | 技术部
李      | 四      | 市场部
王      | 五      | 人事部
```

命令：
```bash
sql_maker -f "employees.xlsx" -c A -c B -c C -r 2 \
  -t "INSERT INTO employees (first_name, last_name, full_name, email, department) VALUES ('{A}', '{B}', '{A}{B}', lower('{A}.{B}@company.com'), '{C}');"
```

输出：
```sql
INSERT INTO employees (first_name, last_name, full_name, email, department) VALUES ('张', '三', '张三', lower('张.三@company.com'), '技术部');
INSERT INTO employees (first_name, last_name, full_name, email, department) VALUES ('李', '四', '李四', lower('李.四@company.com'), '市场部');
INSERT INTO employees (first_name, last_name, full_name, email, department) VALUES ('王', '五', '王五', lower('王.五@company.com'), '人事部');
```

### 场景6：处理大量数据并分片保存

当Excel文件包含数十万行数据时，一次性生成所有SQL语句可能导致文件过大，难以管理和执行。此时可以使用分片功能：

Excel内容（假设有25000行数据）：
```
A列(用户ID)
------------
USER_00001
USER_00002
...
USER_25000
```

命令（每1000条SQL保存到一个文件）：
```bash
sql_maker -f "large_users.xlsx" -c A -r 2 \
  -t "INSERT INTO user_migration (user_id, migrated_at) VALUES ('{value}', NOW());" \
  -o "migration.sql" \
  -b 1000
```

输出结果：
- 自动生成25个文件：`migration_1.sql`, `migration_2.sql`, ..., `migration_25.sql`
- 每个文件包含1000条SQL语句（最后一个文件可能少于1000条）
- 便于分批执行，避免单次执行过多SQL导致数据库压力过大

分片功能特别适用于：
- 数据迁移项目
- 大批量数据导入
- 需要分批处理的场景
- 避免生成过大的SQL文件

### 场景7：使用批量SQL模式优化大量IN条件

当需要为大量ID更新状态时，每条SQL一个ID效率低下。使用批量SQL模式可以将多个ID合并到一条SQL的IN子句中：

Excel内容（A列：用户ID）：
```
用户ID
--------
1001
1002
1003
1004
1005
1006
1007
1008
1009
1010
```

命令（每5个ID生成一条UPDATE语句）：
```bash
sql_maker -f "user_ids.xlsx" -c A -r 2 \
  -t "UPDATE users SET status='premium' WHERE user_id IN ({values});" \
  -n 5 \
  -o "update_users.sql"
```

输出（update_users.sql）：
```sql
UPDATE users SET status='premium' WHERE user_id IN ('1001', '1002', '1003', '1004', '1005');
UPDATE users SET status='premium' WHERE user_id IN ('1006', '1007', '1008', '1009', '1010');
```

相比生成10条单独的UPDATE语句，批量模式：
- ✅ 减少SQL语句数量，提高执行效率
- ✅ 降低数据库连接开销
- ✅ 便于事务管理
- ✅ 适合批量处理场景

### 场景8：批量INSERT优化

将多行数据合并到一条INSERT语句中，大幅提高插入性能：

Excel内容：
```
A列(id) | B列(name) | C列(email)
----------------------------------
1       | Alice     | alice@example.com
2       | Bob       | bob@example.com
3       | Charlie   | charlie@example.com
4       | David     | david@example.com
5       | Eve       | eve@example.com
```

命令（每3行生成一条INSERT语句）：
```bash
sql_maker -f "users.xlsx" -c A -c B -c C -r 2 \
  -t "INSERT INTO users (id, name, email) VALUES {@row};" \
  -n 3 \
  -o "batch_insert.sql"
```

输出（batch_insert.sql）：
```sql
INSERT INTO users (id, name, email) VALUES ('1', 'Alice', 'alice@example.com'), ('2', 'Bob', 'bob@example.com'), ('3', 'Charlie', 'charlie@example.com');
INSERT INTO users (id, name, email) VALUES ('4', 'David', 'david@example.com'), ('5', 'Eve', 'eve@example.com');
```

批量INSERT的优势：
- ⚡ 插入速度提升10-100倍（取决于数据库）
- 📉 减少网络往返次数
- 🔒 减少事务日志写入
- 💾 降低数据库负载

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

### 分片输出（批量保存到多个文件）

当生成的SQL语句数量很大时，可以使用分片功能将其保存到多个文件中：

```bash
# 每1000条SQL保存到一个文件，自动生成 output_1.sql, output_2.sql...
sql_maker -f "large_data.xlsx" -c A -c B -t "INSERT INTO table VALUES ('{1}', '{2}');" -o "output.sql" -b 1000

# 每500条SQL保存到一个文件
sql_maker -f "huge_dataset.xlsx" -c A -t "INSERT INTO logs (data) VALUES ('{value}');" -o "logs.sql" -b 500
```

分片功能会自动：
- 根据指定的批次大小分割SQL语句
- 生成带编号的文件名（如：`output_1.sql`, `output_2.sql`, `output_3.sql`...）
- 显示每个文件包含的SQL条数

**注意**：分片功能仅在指定了`-o`（输出文件）参数时有效。如果未指定输出文件，SQL将显示在控制台上。

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

### v0.4.0
- 添加批量SQL生成功能（Batch SQL Mode）
- 支持通过`--rows-per-sql`参数指定每条SQL语句使用的行数
- 支持三种批量模式：
  - IN子句模式：使用`{values}`占位符（如：WHERE id IN (v1, v2, v3...)）
  - VALUES批量插入模式：使用`{@row}`占位符（如：INSERT ... VALUES (a1,b1), (a2,b2)...）
  - 多行占位符模式：使用`{#1}`, `{#A}`等占位符
- 大幅提高批量操作的执行效率
- 添加完整的测试用例和详细文档

### v0.3.0
- 添加分片输出功能（Batch Output）
- 支持通过`--batch-size`参数指定每个文件包含的SQL条数
- 自动将大量SQL语句分割到多个文件（如：output_1.sql, output_2.sql...）
- 显示详细的分片信息（文件数量、每文件条数）
- 适用于处理大型Excel文件和数据迁移场景

### v0.2.0
- 添加多列支持功能
- 支持从一行中提取多个列值
- 支持两种占位符格式：`{1},{2}`（数字索引）和`{A},{B}`（列名）
- 向后兼容单列模式的`{value}`占位符
- 添加更多测试用例和文档

### v0.1.0
- 初始版本
- 支持Excel读取和SQL生成
- 命令行参数解析
- 支持输出到文件或控制台