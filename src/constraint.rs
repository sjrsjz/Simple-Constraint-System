//! 集合论与区间约束系统的核心定义与操作实现。
//! 本文件实现了基于集合论的约束表达、区间约束、约束的自动归约、超约束(super_of)、联合(union)、判等(equals)、直接交集(direct_intersection)等操作。
//! 注意：本系统不使用“类型”一词，而是统一称为“约束”与“超约束”。

use std::{collections::VecDeque, sync::Arc};

/// 浮点区间端点约束。
/// Inclusive/Exclusive 表示端点是否包含，NegativeInfinity/PositiveInfinity 表示无穷。
#[derive(Clone, Debug, PartialEq)]
pub enum FloatBound {
    Inclusive(f64),   // 闭区间端点
    Exclusive(f64),   // 开区间端点
    NegativeInfinity, // 负无穷
    PositiveInfinity, // 正无穷
}

/// 浮点区间约束，表示 [start, end] 区间，端点可为开/闭/无穷。
#[derive(Clone, Debug, PartialEq)]
pub struct FloatField(FloatBound, FloatBound);

impl FloatField {
    /// 构造新的浮点区间约束。
    pub fn new(start: FloatBound, end: FloatBound) -> Self {
        FloatField(start, end)
    }

    /// 获取区间起点。
    pub fn start(&self) -> &FloatBound {
        &self.0
    }

    /// 获取区间终点。
    pub fn end(&self) -> &FloatBound {
        &self.1
    }

    /// 计算两个浮点区间的交集。
    /// 返回的区间可能为空（is_empty）。
    pub fn intersection(&self, other: &Self) -> Self {
        let start_bound = match (self.start(), other.start()) {
            (FloatBound::NegativeInfinity, s) | (s, FloatBound::NegativeInfinity) => s.clone(),
            (FloatBound::PositiveInfinity, _) | (_, FloatBound::PositiveInfinity) => {
                FloatBound::PositiveInfinity
            }
            (FloatBound::Inclusive(a), FloatBound::Inclusive(b)) => {
                FloatBound::Inclusive(a.max(*b))
            }
            (FloatBound::Exclusive(a), FloatBound::Exclusive(b)) => {
                FloatBound::Exclusive(a.max(*b))
            }
            (FloatBound::Inclusive(a), FloatBound::Exclusive(b))
            | (FloatBound::Exclusive(b), FloatBound::Inclusive(a)) => {
                if a > b {
                    FloatBound::Inclusive(*a)
                } else {
                    FloatBound::Exclusive(*b)
                }
            }
        };

        let end_bound = match (self.end(), other.end()) {
            (FloatBound::PositiveInfinity, s) | (s, FloatBound::PositiveInfinity) => s.clone(),
            (FloatBound::NegativeInfinity, _) | (_, FloatBound::NegativeInfinity) => {
                FloatBound::NegativeInfinity
            }
            (FloatBound::Inclusive(a), FloatBound::Inclusive(b)) => {
                FloatBound::Inclusive(a.min(*b))
            }
            (FloatBound::Exclusive(a), FloatBound::Exclusive(b)) => {
                FloatBound::Exclusive(a.min(*b))
            }
            (FloatBound::Inclusive(a), FloatBound::Exclusive(b))
            | (FloatBound::Exclusive(b), FloatBound::Inclusive(a)) => {
                if a < b {
                    FloatBound::Inclusive(*a)
                } else {
                    FloatBound::Exclusive(*b)
                }
            }
        };

        FloatField::new(start_bound, end_bound)
    }

    /// 判断区间是否包含某个浮点值。
    pub fn contains(&self, value: f64) -> bool {
        let start_check = match self.start() {
            FloatBound::Inclusive(a) => value >= *a,
            FloatBound::Exclusive(a) => value > *a,
            FloatBound::NegativeInfinity => true,
            FloatBound::PositiveInfinity => false,
        };
        let end_check = match self.end() {
            FloatBound::Inclusive(b) => value <= *b,
            FloatBound::Exclusive(b) => value < *b,
            FloatBound::PositiveInfinity => true,
            FloatBound::NegativeInfinity => false,
        };
        start_check && end_check
    }

    /// 判断当前区间是否完全包含另一个区间。
    pub fn contains_field(&self, other: &Self) -> bool {
        let start_check = match (self.start(), other.start()) {
            (FloatBound::NegativeInfinity, _) => true,
            (_, FloatBound::NegativeInfinity) => false,
            (FloatBound::PositiveInfinity, _) => false,
            (_, FloatBound::PositiveInfinity) => false,
            (FloatBound::Inclusive(a), FloatBound::Inclusive(b)) => a <= b,
            (FloatBound::Inclusive(a), FloatBound::Exclusive(b)) => a <= b,
            (FloatBound::Exclusive(a), FloatBound::Inclusive(b)) => a < b,
            (FloatBound::Exclusive(a), FloatBound::Exclusive(b)) => a <= b,
        };

        let end_check = match (self.end(), other.end()) {
            (FloatBound::PositiveInfinity, _) => true,
            (_, FloatBound::PositiveInfinity) => false,
            (FloatBound::NegativeInfinity, _) => false,
            (_, FloatBound::NegativeInfinity) => false,
            (FloatBound::Inclusive(a), FloatBound::Inclusive(b)) => a >= b,
            (FloatBound::Inclusive(a), FloatBound::Exclusive(b)) => a > b,
            (FloatBound::Exclusive(a), FloatBound::Inclusive(b)) => a >= b,
            (FloatBound::Exclusive(a), FloatBound::Exclusive(b)) => a >= b,
        };

        start_check && end_check
    }

    /// 尝试合并两个区间，若有交集或紧邻则返回合并后的区间，否则返回None。
    pub fn union(&self, other: &Self) -> Option<Self> {
        let intersection = self.intersection(other);
        if !intersection.is_empty() {
            let start_bound = match (self.start(), other.start()) {
                (FloatBound::NegativeInfinity, _) | (_, FloatBound::NegativeInfinity) => {
                    FloatBound::NegativeInfinity
                }
                (FloatBound::PositiveInfinity, s) | (s, FloatBound::PositiveInfinity) => s.clone(),
                (FloatBound::Inclusive(a), FloatBound::Inclusive(b)) => {
                    FloatBound::Inclusive(a.min(*b))
                }
                (FloatBound::Exclusive(a), FloatBound::Exclusive(b)) => {
                    FloatBound::Exclusive(a.min(*b))
                }
                (FloatBound::Inclusive(a), FloatBound::Exclusive(b))
                | (FloatBound::Exclusive(b), FloatBound::Inclusive(a)) => {
                    if a < b {
                        FloatBound::Inclusive(*a)
                    } else {
                        FloatBound::Exclusive(*b)
                    }
                }
            };
            let end_bound = match (self.end(), other.end()) {
                (FloatBound::PositiveInfinity, _) | (_, FloatBound::PositiveInfinity) => {
                    FloatBound::PositiveInfinity
                }
                (FloatBound::NegativeInfinity, s) | (s, FloatBound::NegativeInfinity) => s.clone(),
                (FloatBound::Inclusive(a), FloatBound::Inclusive(b)) => {
                    FloatBound::Inclusive(a.max(*b))
                }
                (FloatBound::Exclusive(a), FloatBound::Exclusive(b)) => {
                    FloatBound::Exclusive(a.max(*b))
                }
                (FloatBound::Inclusive(a), FloatBound::Exclusive(b))
                | (FloatBound::Exclusive(b), FloatBound::Inclusive(a)) => {
                    if a > b {
                        FloatBound::Inclusive(*a)
                    } else {
                        FloatBound::Exclusive(*b)
                    }
                }
            };
            return Some(FloatField(start_bound, end_bound));
        }

        // 检查是否紧挨着
        match (self.end(), other.start()) {
            (FloatBound::Inclusive(a), FloatBound::Inclusive(b))
                if (*a - *b).abs() < f64::EPSILON =>
            {
                return Some(FloatField(self.start().clone(), other.end().clone()));
            }
            (FloatBound::Exclusive(a), FloatBound::Inclusive(b))
                if (*a - *b).abs() < f64::EPSILON =>
            {
                return Some(FloatField(self.start().clone(), other.end().clone()));
            }
            (FloatBound::Inclusive(a), FloatBound::Exclusive(b))
                if (*a - *b).abs() < f64::EPSILON =>
            {
                return Some(FloatField(self.start().clone(), other.end().clone()));
            }
            _ => {}
        }
        match (other.end(), self.start()) {
            (FloatBound::Inclusive(a), FloatBound::Inclusive(b))
                if (*a - *b).abs() < f64::EPSILON =>
            {
                return Some(FloatField(other.start().clone(), self.end().clone()));
            }
            (FloatBound::Exclusive(a), FloatBound::Inclusive(b))
                if (*a - *b).abs() < f64::EPSILON =>
            {
                return Some(FloatField(other.start().clone(), self.end().clone()));
            }
            (FloatBound::Inclusive(a), FloatBound::Exclusive(b))
                if (*a - *b).abs() < f64::EPSILON =>
            {
                return Some(FloatField(other.start().clone(), self.end().clone()));
            }
            _ => {}
        }

        None
    }

    /// 判断区间是否为空。
    pub fn is_empty(&self) -> bool {
        match (self.start(), self.end()) {
            (FloatBound::PositiveInfinity, _) | (_, FloatBound::NegativeInfinity) => true,
            (FloatBound::Inclusive(a), FloatBound::Exclusive(b)) if a >= b => true,
            (FloatBound::Exclusive(a), FloatBound::Inclusive(b)) if a >= b => true,
            (FloatBound::Inclusive(a), FloatBound::Inclusive(b)) if a > b => true,
            (FloatBound::Exclusive(a), FloatBound::Exclusive(b)) if a >= b => true,
            _ => false,
        }
    }
}

/// 整数区间端点约束。
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Bound {
    Inclusive(i64),   // 闭区间端点
    Exclusive(i64),   // 开区间端点
    NegativeInfinity, // 负无穷
    PositiveInfinity, // 正无穷
}

/// 整数区间约束，表示 [start, end] 区间，端点可为开/闭/无穷。
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field(Bound, Bound);

impl Field {
    /// 构造新的整数区间约束。
    pub fn new(start: Bound, end: Bound) -> Self {
        Field(start, end)
    }

    /// 获取区间起点。
    pub fn start(&self) -> &Bound {
        &self.0
    }

    /// 获取区间终点。
    pub fn end(&self) -> &Bound {
        &self.1
    }

    /// 计算两个整数区间的交集。
    /// 返回的区间可能为空（is_empty）。
    pub fn intersection(&self, other: &Self) -> Self {
        let start_bound = match (self.start(), other.start()) {
            (Bound::NegativeInfinity, s) | (s, Bound::NegativeInfinity) => s.clone(),
            (Bound::PositiveInfinity, _) | (_, Bound::PositiveInfinity) => Bound::PositiveInfinity,
            (Bound::Inclusive(a), Bound::Inclusive(b)) => Bound::Inclusive((*a).max(*b)),
            (Bound::Exclusive(a), Bound::Exclusive(b)) => Bound::Exclusive((*a).max(*b)),
            (Bound::Inclusive(a), Bound::Exclusive(b))
            | (Bound::Exclusive(b), Bound::Inclusive(a)) => {
                if a > b {
                    Bound::Inclusive(*a)
                } else {
                    Bound::Exclusive(*b)
                }
            }
        };

        let end_bound = match (self.end(), other.end()) {
            (Bound::PositiveInfinity, s) | (s, Bound::PositiveInfinity) => s.clone(),
            (Bound::NegativeInfinity, _) | (_, Bound::NegativeInfinity) => Bound::NegativeInfinity,
            (Bound::Inclusive(a), Bound::Inclusive(b)) => Bound::Inclusive((*a).min(*b)),
            (Bound::Exclusive(a), Bound::Exclusive(b)) => Bound::Exclusive((*a).min(*b)),
            (Bound::Inclusive(a), Bound::Exclusive(b))
            | (Bound::Exclusive(b), Bound::Inclusive(a)) => {
                if a < b {
                    Bound::Inclusive(*a)
                } else {
                    Bound::Exclusive(*b)
                }
            }
        };

        Field::new(start_bound, end_bound)
    }

    /// 判断区间是否包含某个整数值。
    pub fn contains(&self, value: i64) -> bool {
        let start_check = match self.start() {
            Bound::Inclusive(a) => value >= *a,
            Bound::Exclusive(a) => value > *a,
            Bound::NegativeInfinity => true,
            Bound::PositiveInfinity => false,
        };
        let end_check = match self.end() {
            Bound::Inclusive(b) => value <= *b,
            Bound::Exclusive(b) => value < *b,
            Bound::PositiveInfinity => true,
            Bound::NegativeInfinity => false,
        };
        start_check && end_check
    }

    /// 判断当前区间是否完全包含另一个区间。
    pub fn contains_field(&self, other: &Self) -> bool {
        let start_check = match (self.start(), other.start()) {
            (Bound::NegativeInfinity, _) => true,
            (_, Bound::NegativeInfinity) => false,
            (Bound::PositiveInfinity, _) => false,
            (_, Bound::PositiveInfinity) => false,
            (Bound::Inclusive(a), Bound::Inclusive(b)) => a <= b,
            (Bound::Inclusive(a), Bound::Exclusive(b)) => a <= b,
            (Bound::Exclusive(a), Bound::Inclusive(b)) => a < b,
            (Bound::Exclusive(a), Bound::Exclusive(b)) => a <= b,
        };

        let end_check = match (self.end(), other.end()) {
            (Bound::PositiveInfinity, _) => true,
            (_, Bound::PositiveInfinity) => false,
            (Bound::NegativeInfinity, _) => false,
            (_, Bound::NegativeInfinity) => false,
            (Bound::Inclusive(a), Bound::Inclusive(b)) => a >= b,
            (Bound::Inclusive(a), Bound::Exclusive(b)) => a > b,
            (Bound::Exclusive(a), Bound::Inclusive(b)) => a >= b,
            (Bound::Exclusive(a), Bound::Exclusive(b)) => a >= b,
        };

        start_check && end_check
    }

    /// 尝试合并两个区间，若有交集或紧邻则返回合并后的区间，否则返回None。
    pub fn union(&self, other: &Self) -> Option<Self> {
        let intersection = self.intersection(other);
        if !intersection.is_empty() {
            let start_bound = match (self.start(), other.start()) {
                (Bound::NegativeInfinity, _) | (_, Bound::NegativeInfinity) => {
                    Bound::NegativeInfinity
                }
                (Bound::PositiveInfinity, s) | (s, Bound::PositiveInfinity) => s.clone(),
                (Bound::Inclusive(a), Bound::Inclusive(b)) => Bound::Inclusive((*a).min(*b)),
                (Bound::Exclusive(a), Bound::Exclusive(b)) => Bound::Exclusive((*a).min(*b)),
                (Bound::Inclusive(a), Bound::Exclusive(b))
                | (Bound::Exclusive(b), Bound::Inclusive(a)) => {
                    if a < b {
                        Bound::Inclusive(*a)
                    } else {
                        Bound::Exclusive(*b)
                    }
                }
            };
            let end_bound = match (self.end(), other.end()) {
                (Bound::PositiveInfinity, _) | (_, Bound::PositiveInfinity) => {
                    Bound::PositiveInfinity
                }
                (Bound::NegativeInfinity, s) | (s, Bound::NegativeInfinity) => s.clone(),
                (Bound::Inclusive(a), Bound::Inclusive(b)) => Bound::Inclusive((*a).max(*b)),
                (Bound::Exclusive(a), Bound::Exclusive(b)) => Bound::Exclusive((*a).max(*b)),
                (Bound::Inclusive(a), Bound::Exclusive(b))
                | (Bound::Exclusive(b), Bound::Inclusive(a)) => {
                    if a > b {
                        Bound::Inclusive(*a)
                    } else {
                        Bound::Exclusive(*b)
                    }
                }
            };
            return Some(Field(start_bound, end_bound));
        }

        // 检查是否紧挨着
        match (self.end(), other.start()) {
            (Bound::Inclusive(a), Bound::Inclusive(b)) if *a + 1 == *b => {
                return Some(Field(self.start().clone(), other.end().clone()));
            }
            (Bound::Exclusive(a), Bound::Inclusive(b)) if *a == *b => {
                return Some(Field(self.start().clone(), other.end().clone()));
            }
            (Bound::Inclusive(a), Bound::Exclusive(b)) if *a == *b => {
                return Some(Field(self.start().clone(), other.end().clone()));
            }
            _ => {}
        }
        match (other.end(), self.start()) {
            (Bound::Inclusive(a), Bound::Inclusive(b)) if *a + 1 == *b => {
                return Some(Field(other.start().clone(), self.end().clone()));
            }
            (Bound::Exclusive(a), Bound::Inclusive(b)) if *a == *b => {
                return Some(Field(other.start().clone(), self.end().clone()));
            }
            (Bound::Inclusive(a), Bound::Exclusive(b)) if *a == *b => {
                return Some(Field(other.start().clone(), self.end().clone()));
            }
            _ => {}
        }

        None
    }

    /// 判断区间是否为空。
    pub fn is_empty(&self) -> bool {
        match (self.start(), self.end()) {
            (Bound::PositiveInfinity, _) | (_, Bound::NegativeInfinity) => true,
            (Bound::Inclusive(a), Bound::Exclusive(b)) if a >= b => true,
            (Bound::Exclusive(a), Bound::Inclusive(b)) if a >= b => true,
            (Bound::Inclusive(a), Bound::Inclusive(b)) if a > b => true,
            (Bound::Exclusive(a), Bound::Exclusive(b)) if a >= b => true, // (a, a) is empty
            _ => false,
        }
    }
}

/// 约束表达式的核心枚举。
/// 约束可为字面量、区间、基本约束、Tuple、联合等。
/// 这里我们暂时不实现关系约束Relation<P, F>（其中F为关系的映射定义R）
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Constraint {
    Top,    // 逻辑意义上的空约束
    Bottom, // 强制任何逻辑上存在的约束，任何非Bottom约束都是Bottom的超约束

    // 原子约束的定义，他们不具备可约性
    // 原子约束是这样的一种约束：他们与其他的原子约束互不兼容，其自身的交集运算必须使用下述公理：
    // AtomicA intersection AtomicB = AtomicA.refine(AtomicB) union AtomicB.refine(AtomicA)
    LiteralInt(i64),             // 整数字面量约束
    LiteralFloat(f64),           // 浮点字面量约束
    LiteralString(String),       // 字符串字面量约束
    Tuple(Arc<Vec<Constraint>>), // Tuple约束（元素列表）。Tuple不是笛卡尔积！相反，Tuple指的是笛卡尔积结果集合中的那些元素，即有序列表。

    // 注意，在我们的系统中，下述区间约束是连续Literal的并集的简化表示，比如Bound(0, 10) 表示 0 union 1 union ... union 10
    // 我们的系统本质上可以脱离这些区间约束而存在，但为了方便表达和优化，我们仍然保留了这些区间约束。
    Bound(Bound, Bound),                // 整数区间约束
    FloatBound(FloatBound, FloatBound), // 浮点区间约束
    Int,                                // 整数超约束
    Float,                              // 浮点超约束
    String,                             // 字符串超约束
    // 还需要一个 Difference 约束，用于表示差集约束。
    // 注意，我们的 Difference 约束仍然对应一个抽象的Union，只不过我们无法在内存中存储这个 Difference 约束的具体内容。Difference(A, B) 对应 A intersection not B。
    Difference(Arc<Constraint>, Arc<Constraint>), // 差集约束，表示 A intersection not B

    // 我们的系统必须包含一个联合约束类型，用于表示多个并操作不可约约束的并集。
    Union(Arc<Vec<Constraint>>), //
}

/// 约束的核心操作实现。
#[allow(dead_code)]
impl Constraint {
    /// 判断 self 是否为 other 的超约束。
    /// 即 self 是否包含 other 所有可能的取值。
    pub fn super_of(&self, other: &Self) -> bool {
        match (self, other) {
            (Constraint::Top, _) => true,
            (_, Constraint::Bottom) => true,
            (Constraint::Bottom, _) => false,
            (_, Constraint::Top) => false,

            (Constraint::Int, Constraint::LiteralInt(_)) => true,
            (Constraint::Float, Constraint::LiteralFloat(_)) => true,
            (Constraint::FloatBound(s1, e1), Constraint::FloatBound(s2, e2)) => {
                FloatField::new(s1.clone(), e1.clone())
                    .contains_field(&FloatField::new(s2.clone(), e2.clone()))
            }
            (Constraint::FloatBound(s, e), Constraint::LiteralFloat(v)) => {
                FloatField::new(s.clone(), e.clone()).contains(*v)
            }
            (Constraint::FloatBound(s, e), Constraint::LiteralInt(v)) => {
                FloatField::new(s.clone(), e.clone()).contains(*v as f64)
            }
            (Constraint::LiteralInt(a), Constraint::LiteralInt(b)) => a == b,
            (Constraint::LiteralFloat(a), Constraint::LiteralFloat(b)) => a == b,
            (Constraint::Int, Constraint::Int) => true,
            (Constraint::Float, Constraint::Float) => true,
            (Constraint::Int, Constraint::Bound(_, _)) => true,
            (Constraint::Float, Constraint::FloatBound(_, _)) => true,
            (Constraint::Float, Constraint::LiteralInt(_)) => true,
            (Constraint::String, Constraint::LiteralString(_)) => true,
            (Constraint::LiteralString(a), Constraint::LiteralString(b)) => a == b,
            (Constraint::String, Constraint::String) => true,
            (Constraint::Bound(s1, e1), Constraint::Bound(s2, e2)) => {
                Field::new(s1.clone(), e1.clone())
                    .contains_field(&Field::new(s2.clone(), e2.clone()))
            }
            (Constraint::Bound(s, e), Constraint::LiteralInt(v)) => {
                Field::new(s.clone(), e.clone()).contains(*v)
            }
            (Constraint::Tuple(t1), Constraint::Tuple(t2)) => {
                if t1.len() != t2.len() {
                    return false;
                }
                for (elem1, elem2) in t1.iter().zip(t2.iter()) {
                    if !elem1.super_of(elem2) {
                        return false;
                    }
                }
                true
            }
            (Constraint::Union(a), b) => {
                let b_elements = b.make_union_flat();
                b_elements
                    .iter()
                    .all(|b_elem| a.iter().any(|a_elem| a_elem.super_of(b_elem)))
            }
            (a, Constraint::Union(b)) => b.iter().all(|c| a.super_of(c)),

            // // 匹配 (A - B) ⊇ (C - D)
            // // 它是可以被后面的匹配所覆盖的，但为了性能考虑，我们将其放在这里。
            // (Constraint::Difference(a, b), Constraint::Difference(c, d)) => {
            //     // 条件1: (A ∪ D) ⊇ C
            //     let a_union_d = a.as_ref().union(d.as_ref());
            //     if !a_union_d.super_of(c.as_ref()) {
            //         return false;
            //     }

            //     // 条件2: D ⊇ (B ∩ C)
            //     let b_intersect_c = b.as_ref().intersection(c.as_ref());
            //     d.as_ref().super_of(&b_intersect_c)
            // }

            // 匹配 A ⊇ (B - C) => (A ∪ C) ⊇ B
            (a, Constraint::Difference(b, c)) => {
                // 计算 A ∪ C
                let a_union_c = a.union(c.as_ref());
                // 判断 (A ∪ C) ⊇ B
                a_union_c.super_of(b.as_ref())
            }

            // 匹配 (A - B) ⊇ C => A ⊇ (B ∪ C) & (B ∩ C) = Bottom
            (Constraint::Difference(a, b), c) => {
                // 检查条件1: B 和 C 是否不相交
                let b_intersect_c = b.as_ref().intersection(c);
                if !matches!(b_intersect_c, Constraint::Bottom) {
                    return false; // 如果相交，则不成立
                }

                // 检查条件2: A 是否是 C 的超集
                a.as_ref().super_of(c)
            }

            _ => false,
        }
    }

    /// 判等：判断两个约束是否等价（互为超约束）。
    pub fn equals(&self, other: &Self) -> bool {
        self.super_of(other) && other.super_of(self)
    }

    /// 约束细化：若 self 是 other 的超约束，则返回 other，否则返回 Bottom。
    pub fn refine(&self, other: &Self) -> Self {
        if self.super_of(other) {
            return other.clone();
        }
        Self::Bottom
    }

    /// 联合约束归约：自动去重、合并、吸收，归约为最简联合。
    fn reduce_union(elements: &Vec<Self>) -> Self {
        if elements.is_empty() {
            return Constraint::Bottom;
        }
        if elements.len() == 1 {
            return elements[0].clone();
        }
        let mut unique_elements = Vec::new();
        let mut queue: VecDeque<_> = elements.clone().into();

        while let Some(current) = queue.pop_front() {
            if current.equals(&Constraint::Bottom) {
                continue;
            }

            let mut is_absorbed = false;
            unique_elements.retain(|existing_element: &Constraint| {
                if existing_element.super_of(&current) {
                    is_absorbed = true;
                    return true;
                }
                !current.super_of(existing_element)
            });

            if is_absorbed {
                continue;
            }

            let mut can_union = false;
            for i in (0..unique_elements.len()).rev() {
                if let Some(union) = current.direct_union(&unique_elements[i]) {
                    can_union = true;
                    unique_elements.remove(i);
                    queue.push_back(union);
                    break;
                }
            }
            if !can_union {
                unique_elements.push(current);
            }
        }
        if unique_elements.is_empty() {
            return Constraint::Bottom;
        }
        if unique_elements.len() == 1 {
            return unique_elements.pop().unwrap();
        }
        Constraint::Union(Arc::new(
            unique_elements.iter().map(|e| e.reduce()).collect(),
        ))
    }

    fn direct_difference(&self, other: &Constraint) -> Option<Self> {
        if let Constraint::Top = other {
            return Some(Constraint::Bottom);
        }
        if let Constraint::Bottom = self {
            return Some(Constraint::Bottom);
        }
        if let Constraint::Bottom = other {
            return Some(self.clone());
        }

        // 此处还应该有其他逻辑来处理抽象约束

        // 对于原子值，我们认为当A > B时无简单表示，当B >= A时返回Bottom，当A与B没有任何superof关系时直接返回A

        // B >= A
        if other.super_of(self) {
            return Some(Constraint::Bottom);
        }
        // A > B
        if self.super_of(other) {
            return None;
        }
        // A 与 B 没有任何 super_of 关系，直接返回 A
        return Some(self.clone());
    }

    fn reduce_difference(&self, other: &Constraint) -> Option<Self> {
        // 直接返回 A - B 的差集约束
        // (A - B) - C => A - (B ∪ C)
        // (A U B) - C => (A - C) U (B - C)
        // A - (B - C) => A intersection not (B intersection not C) => A intersection (not B union C) => (A - B) U (A intersection C)
        // A - (B U C) => 依次检查A可否被第N项直接归约，不能则跳过。能则归约成A'然后进行下一个元素的判定
        match (self, other) {
            (Constraint::Difference(a, b), c) => {
                return a.reduce_difference(&b.union(c));
            }
            (Constraint::Union(elements), c) => {
                let v = elements.iter().map(|e| e.difference(c)).collect::<Vec<_>>();
                return Some(Self::reduce_union(&v));
            }
            (a, Constraint::Union(elements)) => {
                let mut new_a = a.clone();
                let mut new_union = elements.as_ref().clone();
                let mut modified = true;
                while modified {
                    modified = false;
                    new_union.retain(|e| {
                        match new_a.reduce_difference(e) {
                            Some(diff) => {
                                new_a = diff;
                                modified = true; // 需要继续处理
                                false // 直接差集后不再需要这个元素
                            }
                            None => true, // 保留这个元素
                        }
                    });
                }
                if new_union.is_empty() {
                    // 所有元素都被成功差集掉了
                    Some(new_a)
                } else {
                    Some(Constraint::Difference(
                        Arc::new(new_a),
                        Arc::new(Constraint::reduce_union(&new_union)),
                    ))
                }
            }
            (a, Constraint::Difference(b, c)) => {
                return Some(Self::reduce_union(&vec![
                    a.difference(b),
                    a.intersection(c),
                ]));
            }
            (a, b) => return a.direct_difference(b),
        }
    }

    pub fn difference(&self, other: &Self) -> Self {
        match self.reduce_difference(other) {
            Some(diff) => diff,
            None => Constraint::Difference(Arc::new(self.clone()), Arc::new(other.clone())),
        }
    }

    /// 展开联合约束为扁平Vec。
    fn make_union_flat(&self) -> Vec<Self> {
        match self {
            Constraint::Union(elements) => {
                elements.iter().flat_map(|e| e.make_union_flat()).collect()
            }
            _ => vec![self.clone()],
        }
    }

    /// 归约自身为最简约束。
    pub fn reduce(&self) -> Self {
        match self {
            Constraint::Union(elements) => Constraint::reduce_union(elements),
            Constraint::Difference(a, b) => a.difference(b),
            _ => self.clone(),
        }
    }

    /// 直接联合：只处理两个原子约束的直接合并，不做归约和分解。
    fn direct_union(&self, other: &Self) -> Option<Self> {
        if self.super_of(other) {
            return Some(self.clone());
        }
        if other.super_of(self) {
            return Some(other.clone());
        }
        if let Some(result) = Self::direct_union_internal(self, other) {
            return Some(result);
        }
        if let Some(result) = Self::direct_union_internal(other, self) {
            return Some(result);
        }
        None
    }

    /// 直接联合的内部实现。
    fn direct_union_internal(c1: &Self, c2: &Self) -> Option<Self> {
        match (c1, c2) {
            (Constraint::FloatBound(s1, e1), Constraint::FloatBound(s2, e2)) => {
                let f1 = FloatField::new(s1.clone(), e1.clone());
                let f2 = FloatField::new(s2.clone(), e2.clone());
                f1.union(&f2).map(|union_field| {
                    Constraint::FloatBound(union_field.start().clone(), union_field.end().clone())
                })
            }
            (Constraint::Bound(s1, e1), Constraint::Bound(s2, e2)) => {
                let f1 = Field::new(s1.clone(), e1.clone());
                let f2 = Field::new(s2.clone(), e2.clone());
                f1.union(&f2).map(|union_field| {
                    Constraint::Bound(union_field.start().clone(), union_field.end().clone())
                })
            }
            _ => None,
        }
    }

    /// 联合操作：自动归约 self 与 other 的联合。
    pub fn union(&self, other: &Self) -> Self {
        if self.super_of(other) {
            return self.clone();
        }
        if other.super_of(self) {
            return other.clone();
        }
        let mut elements = self.make_union_flat();
        elements.extend(other.make_union_flat());
        Constraint::reduce_union(&elements)
    }

    /// 直接交集：只处理两个非联合约束的交集，不做归约和分解。
    /// 实际上，我们定义不可约原子值的交集操作为 A ∩ B := A.refine(B) ∪ B.refine(A)
    fn direct_intersection(&self, other: &Self) -> Self {
        // 专门为可部分重叠的复合类型设计的“快速通道”
        // 他们存在的意义是为了进行区间这种Union简化表示的交运算
        let specialized_intersection = |c1: &Self, c2: &Self| -> Option<Self> {
            match (c1, c2) {
                // 1. Top/Bottom handling
                (Constraint::Top, other) => Some(other.clone()),
                (Constraint::Bottom, _) => Some(Constraint::Bottom),

                // 2. Same base type intersection
                (Constraint::Int, Constraint::Int) => Some(Constraint::Int),
                (Constraint::Float, Constraint::Float) => Some(Constraint::Float),
                (Constraint::String, Constraint::String) => Some(Constraint::String),

                // 3. Type with Literal intersection
                (Constraint::Int, Constraint::LiteralInt(v)) => Some(Constraint::LiteralInt(*v)),
                (Constraint::Float, Constraint::LiteralFloat(v)) => {
                    Some(Constraint::LiteralFloat(*v))
                }
                (Constraint::String, Constraint::LiteralString(s)) => {
                    Some(Constraint::LiteralString(s.clone()))
                }

                // 4. Type with Bound intersection
                (Constraint::Int, Constraint::Bound(s, e)) => {
                    Some(Constraint::Bound(s.clone(), e.clone()))
                }
                (Constraint::Float, Constraint::FloatBound(s, e)) => {
                    Some(Constraint::FloatBound(s.clone(), e.clone()))
                }

                // 5. Cross-type intersections (Int and Float)
                (Constraint::Float, Constraint::Int) => Some(Constraint::Int),
                (Constraint::Float, Constraint::LiteralInt(v)) => {
                    Some(Constraint::LiteralFloat(*v as f64))
                }
                (Constraint::Float, Constraint::Bound(s, e)) => {
                    let start_float = match s {
                        Bound::Inclusive(i) => FloatBound::Inclusive(*i as f64),
                        Bound::Exclusive(i) => FloatBound::Exclusive(*i as f64),
                        Bound::NegativeInfinity => FloatBound::NegativeInfinity,
                        Bound::PositiveInfinity => FloatBound::PositiveInfinity,
                    };
                    let end_float = match e {
                        Bound::Inclusive(i) => FloatBound::Inclusive(*i as f64),
                        Bound::Exclusive(i) => FloatBound::Exclusive(*i as f64),
                        Bound::NegativeInfinity => FloatBound::NegativeInfinity,
                        Bound::PositiveInfinity => FloatBound::PositiveInfinity,
                    };
                    Some(Constraint::FloatBound(start_float, end_float))
                }
                (Constraint::Int, Constraint::LiteralFloat(v)) => {
                    if v.fract() == 0.0 {
                        Some(Constraint::LiteralInt(*v as i64))
                    } else {
                        Some(Constraint::Bottom) // Use Bottom for empty set
                    }
                }
                (Constraint::Int, Constraint::FloatBound(s, e)) => {
                    let start_int = match s {
                        FloatBound::Inclusive(v) => Bound::Inclusive(v.ceil() as i64),
                        FloatBound::Exclusive(v) => Bound::Inclusive(v.floor() as i64 + 1),
                        FloatBound::NegativeInfinity => Bound::NegativeInfinity,
                        FloatBound::PositiveInfinity => return Some(Constraint::Bottom),
                    };
                    let end_int = match e {
                        FloatBound::Inclusive(v) => Bound::Inclusive(v.floor() as i64),
                        FloatBound::Exclusive(v) => Bound::Inclusive(v.ceil() as i64 - 1),
                        FloatBound::PositiveInfinity => Bound::PositiveInfinity,
                        FloatBound::NegativeInfinity => return Some(Constraint::Bottom),
                    };
                    let result_field = Field::new(start_int, end_int);
                    if result_field.is_empty() {
                        Some(Constraint::Bottom)
                    } else {
                        Some(Constraint::Bound(result_field.0, result_field.1))
                    }
                }

                // 6. Bound with Bound intersection
                (Constraint::Bound(s1, e1), Constraint::Bound(s2, e2)) => {
                    let intersection = Field::new(s1.clone(), e1.clone())
                        .intersection(&Field::new(s2.clone(), e2.clone()));
                    if intersection.is_empty() {
                        Some(Constraint::Bottom)
                    } else {
                        Some(Constraint::Bound(intersection.0, intersection.1))
                    }
                }
                (Constraint::FloatBound(s1, e1), Constraint::FloatBound(s2, e2)) => {
                    let intersection = FloatField::new(s1.clone(), e1.clone())
                        .intersection(&FloatField::new(s2.clone(), e2.clone()));
                    if intersection.is_empty() {
                        Some(Constraint::Bottom)
                    } else {
                        Some(Constraint::FloatBound(intersection.0, intersection.1))
                    }
                }

                // 7. Bound with Literal intersection
                (Constraint::Bound(s, e), Constraint::LiteralInt(v)) => {
                    if Field::new(s.clone(), e.clone()).contains(*v) {
                        Some(Constraint::LiteralInt(*v))
                    } else {
                        Some(Constraint::Bottom)
                    }
                }
                (Constraint::FloatBound(s, e), Constraint::LiteralFloat(v)) => {
                    if FloatField::new(s.clone(), e.clone()).contains(*v) {
                        Some(Constraint::LiteralFloat(*v))
                    } else {
                        Some(Constraint::Bottom)
                    }
                }
                (Constraint::FloatBound(s, e), Constraint::LiteralInt(v)) => {
                    if FloatField::new(s.clone(), e.clone()).contains(*v as f64) {
                        Some(Constraint::LiteralInt(*v))
                    } else {
                        Some(Constraint::Bottom)
                    }
                }

                // For all other cases, we have no specialized logic.
                _ => None,
            }
        };

        // Try the specialized logic in both directions (for commutativity)
        if let Some(result) = specialized_intersection(self, other) {
            return result;
        }
        if let Some(result) = specialized_intersection(other, self) {
            return result;
        }

        return self.refine(&other).union(&other.refine(self));
    }

    /// 交集操作：自动归约 self 与 other 的交集。
    pub fn intersection(&self, other: &Self) -> Self {
        if self.super_of(other) {
            return other.clone();
        }
        if other.super_of(self) {
            return self.clone();
        }
        let a = self.make_union_flat();
        let b = other.make_union_flat();
        let mut intersection_elements = Vec::new();
        for a_elem in a {
            for b_elem in &b {
                intersection_elements.push(a_elem.direct_intersection(b_elem));
            }
        }
        Constraint::make_union(intersection_elements)
    }

    // --- Constructor Helpers ---

    /// 构造联合约束并自动归约。
    pub fn make_union(constraints: Vec<Self>) -> Self {
        Constraint::reduce_union(&constraints)
    }

    pub fn make_difference(a: Self, b: Self) -> Self {
        a.difference(&b)
    }

    /// 构造Pair约束。
    pub fn make_pair(key: Self, value: Self) -> Self {
        Constraint::Tuple(Arc::new(vec![key, value]))
    }

    pub fn make_tuple(elements: Vec<Self>) -> Self {
        Constraint::Tuple(Arc::new(elements))
    }
}
