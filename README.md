# 基于集合论的约束系统 (Set-Theoretic Constraint System)

这是一个在 Rust 中实现的、基于集合论和区间理论的约束系统。它旨在提供一个强大且灵活的工具，用于定义、操作和推导各种数据约束。

与传统的类型系统不同，本系统不使用“类型”的概念，而是统一采用“约束”（Constraint）和“超约束”（Super-constraint）来描述数据满足的条件。系统的核心目标是实现所有约束操作的**自动归约**，确保任何约束组合都能被化简为其最简、最精确的范式。

## 核心设计理念

- **集合论基础**：所有操作（联合、交集、子集判断）都严格遵循集合论公理。例如，`A.super_of(B)` 等价于 `A` 约束是 `B` 约束的子集（约束更少对应的值范围越大）。
- **完备性与范式归约**：系统被设计为“完备的”，任何约束之间的运算（如交集）都会立即被求解并归约为一个已知的约束形式。因此，系统中不存在 `Intersection` 这样的中间约束，所有结果都是其最简范式。
- **精确的约束表达**：支持从具体的字面量（如 `5`）到抽象的超约束（如 `Int`），再到复杂的区间和联合约束，实现了多层次的精确表达。

## 功能特性

- **基础约束**:
  - `Top`: 万能超约束，包含任何值。
  - `Bottom`: 空约束，不包含任何值。
- **原子约束**:
  - `LiteralInt(i64)`, `LiteralFloat(f64)`, `LiteralBool(bool)`, `LiteralString(String)`: 表示精确的常量值。
  - `Tuple`: 表示一个元组约束，可用于构建更复杂的结构。
- **区间约束**:
  - `Bound`: 表示整数区间 `[start, end]`。
  - `FloatBound`: 表示浮点数区间 `[start, end]`。
  - 支持开区间、闭区间以及正负无穷。
  - `Int`, `Float`, `Bool`, `String`: 基本超约束（抽象的无穷Union）。
- **复合约束**:
  - `Union`: 表示多个约束的并集，支持自动归约、去重和吸收。
- **核心操作**:
  - `super_of(&self, other: &Self)`: 判断 `self` 是否为 `other` 的超约束（即 `other` 是否为 `self` 的子集）。
  - `union(&self, other: &Self)`: 计算两个约束的并集，并自动归约。
  - `intersection(&self, other: &Self)`: 计算两个约束的交集，并自动归约。
  - `equals(&self, other: &Self)`: 判断两个约束在逻辑上是否等价。
  - `reduce(&self)`: 将一个约束归约为其最简范式。

## 使用示例

以下示例展示了本约束系统的强大功能（代码来自 `main.rs`）。

### 1. 区间与字面量的联合

```rust
// 一个在区间内的点会被区间吸收
let c1 = Constraint::Bound(Bound::Inclusive(1), Bound::Inclusive(10));
let c2 = Constraint::LiteralInt(5);
let c3 = c1.union(&c2);
// 输出: Bound(Inclusive(1), Inclusive(10))

// 一个在区间外的点会构成联合约束
let c4 = Constraint::Bound(Bound::Inclusive(1), Bound::Inclusive(10));
let c5 = Constraint::LiteralInt(15);
let c6 = c4.union(&c5);
// 输出: Union([Bound(Inclusive(1), Inclusive(10)), LiteralInt(15)])
```

### 2. 约束的自动归约

```rust
// 两个相邻的区间可以合并
let c14 = Constraint::Bound(Bound::Inclusive(1), Bound::Inclusive(5));
let c15 = Constraint::Bound(Bound::Inclusive(6), Bound::Inclusive(10));
let c16 = c14.union(&c15);
// 输出: Bound(Inclusive(1), Inclusive(10))

// 浮点数、整数、字面量和 Bottom 约束的复杂联合
let mixed_union = Constraint::make_union(vec![
    Constraint::LiteralInt(1),
    Constraint::Int, // 吸收 LiteralInt(1)
    Constraint::LiteralFloat(2.5),
    Constraint::Float,  // 吸收 LiteralFloat(2.5) 和 Int
    Constraint::Bottom, // 被任何约束吸收
]);
// 归约后输出: Float
```

### 3. 超约束判断

```rust
// Float 是 LiteralInt 的超约束（类型提升）
let float_type = Constraint::Float;
let int_val = Constraint::LiteralInt(42);
println!(
    "Float super_of LiteralInt(42) = {}",
    float_type.super_of(&int_val)
);
// 输出: true

// 嵌套 Pair 的超约束判断
let nested_pair = Constraint::make_pair(
    Constraint::make_pair(Constraint::Int, Constraint::Float),
    Constraint::String,
);
let concrete_nested = Constraint::make_pair(
    Constraint::make_pair(Constraint::LiteralInt(10), Constraint::LiteralFloat(3.14)),
    Constraint::LiteralString("world".to_string()),
);
println!(
    "Nested pair super_of concrete = {}",
    nested_pair.super_of(&concrete_nested)
);
// 输出: true
```