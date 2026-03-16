# STEP 7：方法一（编译反馈闭环）设计与改造清单

本 Step 的目标：在现有 ArkTS → Rust 的规则生成器基础上，引入“编译反馈闭环”，自动把 rustc 的报错反馈给 LLM 进行修复，直到通过编译或达到重试上限。

---

## 1. 总体思路

流水线：

```text
ArkTS 源码
  -> arkts2rust 生成 Rust 源码（baseline）
  -> rustc / cargo check 编译
  -> 收集诊断信息（error + span）
  -> LLM 生成修复补丁
  -> 应用补丁并重试
```

核心原则：
- 生成器保持不变，保证结构正确
- AI 只做“修补与规范化”，避免全量重写
- 通过编译器反馈形成闭环，保证可执行性

---

## 2. 需要改造的模块

### 2.1 新增驱动层（Coordinator）

新增一个脚本作为外层驱动，职责包括：
- 调用现有 CLI 生成 Rust 代码
- 运行 rustc / cargo check
- 解析编译错误（标准错误输出）
- 构建 LLM 提示词并请求修复
- 应用补丁，继续检查

建议路径：
- `tools/driver.py`（Python 实现，便于快速迭代）

### 2.2 可选：增强 CLI

给 `src/main.rs` 增加一个可选参数，例如：
- `--emit-only`：只生成 Rust 不编译
- `--stdout`：输出到 stdout 便于管道处理

这是为了驱动层在不同实验中复用，但不是强制项。

---

## 3. Prompt 设计（修复任务）

输入信息：
- ArkTS 原始源码（只读）
- 规则生成的 Rust 源码
- rustc 的错误信息（含行号、错误码）

输出目标：
- 只返回修改后的 Rust 源码
- 保持原有结构，不要重排函数顺序
- 尽量小改动修复编译错误

建议 Prompt 结构：

```text
你是 Rust 编译修复器。
输入包含：
1) ArkTS 源码
2) 规则生成的 Rust 源码
3) rustc 编译错误信息

输出只包含修复后的 Rust 源码，不要解释。
要求：
- 最小修改原则
- 不新增无关功能
- 保持函数与变量命名
```

---

## 4. 重试策略

参数建议：
- 最大重试次数：3~5
- 每次修复后重新运行 rustc
- 如果错误类型不变且次数达到上限，返回失败

输出报告：
- 修复成功 / 失败
- 修复轮数
- 最终错误摘要（失败时）

---

## 5. 改动清单（文件级）

新增：
- `tools/driver.py`：闭环驱动脚本

可选修改：
- `src/main.rs`：补充 stdout 或 emit-only 支持

不改动：
- `src/ast.rs`
- `src/parser/*`
- `src/codegen.rs`

---

## 6. 评测方案（论文用）

数据集：
- 20~50 个 ArkTS 程序（算法题或教材示例）

指标：
- Baseline 通过率（仅规则生成）
- Feedback Loop 通过率（规则 + 修复）
- 平均修复轮次
- 平均编译错误数下降

消融实验：
- 无 AI
- AI 单次修复（不闭环）
- AI 多次修复（闭环）

---

## 7. 本 Step 的最小可运行目标

1) `tools/driver.py` 能从输入 `.ets` 文件跑通：
   - 生成 Rust
   - 编译错误被捕获
   - 调用 LLM 后生成修复版 Rust
   - 通过 rustc 或给出明确失败信息

2) 命令行示例：

```bash
python tools/driver.py --input examples/demo.ets --out output.rs --max-retries 3
```

---

## 8. 后续可扩展点

- 将错误类别做结构化（borrow/lifetime/move）
- 使用 diff/patch 方式避免 LLM 输出全量代码
- 引入语义等价性测试（输入输出一致）
