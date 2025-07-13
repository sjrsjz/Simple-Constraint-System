use std::{
    collections::{HashMap, HashSet},
    fmt,
    hash::Hash,
    sync::Arc,
};

#[derive(Debug, Clone)]
pub enum AtomicConstraint {
    Nil,
    LiteralInt(i32),
}

impl PartialEq for AtomicConstraint {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AtomicConstraint::LiteralInt(a), AtomicConstraint::LiteralInt(b)) => a == b,
            (AtomicConstraint::Nil, AtomicConstraint::Nil) => true,
            _ => false,
        }
    }
}

impl Hash for AtomicConstraint {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            AtomicConstraint::LiteralInt(value) => value.hash(state),
            AtomicConstraint::Nil => "Nil".hash(state),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConstraintNode {
    T,                                              // 顶点约束，表示任意值
    F,                                              // 假约束，表示不可能的值
    Leaf(AtomicConstraint),                         // 原子约束，例如整数字面量
    Enum(Vec<ConstraintNode>),                      // 枚举约束，对应集合并集
    Pair(Box<ConstraintNode>, Box<ConstraintNode>), // 组合约束，对应笛卡尔积
    Def(String),                                    // 定义约束，用于表示递归定义
}

impl PartialEq for ConstraintNode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ConstraintNode::T, ConstraintNode::T) => true,
            (ConstraintNode::F, ConstraintNode::F) => true,
            (ConstraintNode::Leaf(a), ConstraintNode::Leaf(b)) => a == b,
            (ConstraintNode::Enum(a), ConstraintNode::Enum(b)) => a == b,
            (ConstraintNode::Pair(a1, b1), ConstraintNode::Pair(a2, b2)) => a1 == a2 && b1 == b2,
            (ConstraintNode::Def(name1), ConstraintNode::Def(name2)) => name1 == name2,
            _ => false,
        }
    }
}

impl Eq for ConstraintNode {}

impl Hash for ConstraintNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            ConstraintNode::T => "T".hash(state),
            ConstraintNode::F => "F".hash(state),
            ConstraintNode::Leaf(lit) => lit.hash(state),
            ConstraintNode::Enum(nodes) => {
                "Enum".hash(state);
                nodes.hash(state);
            }
            ConstraintNode::Pair(a, b) => {
                "Pair".hash(state);
                a.hash(state);
                b.hash(state);
            }
            ConstraintNode::Def(name) => name.hash(state),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Constraint {
    graph: Arc<HashMap<String, ConstraintNode>>,
    entry: String, // 入口节点
}

impl Constraint {
    pub fn new(entry: String) -> Self {
        Constraint {
            graph: HashMap::new().into(),
            entry,
        }
    }

    pub fn add_node(&mut self, name: String, node: ConstraintNode) {
        match Arc::get_mut(&mut self.graph) {
            Some(graph) => {
                graph.insert(name, node);
            }
            None => {
                panic!("Cannot modify a shared graph");
            }
        }
    }

    pub fn get_node(&self, name: &str) -> Option<&ConstraintNode> {
        self.graph.get(name)
    }

    pub fn entry(&self) -> &String {
        &self.entry
    }
}

impl Constraint {
    pub fn super_of(&self, other: &Self) -> bool {
        let mut assumption = HashSet::new();
        match self.get_node(&self.entry) {
            Some(node_a) => match other.get_node(&other.entry) {
                Some(node_b) => {
                    Constraint::check_subsumption(self, other, node_a, node_b, &mut assumption)
                }
                None => false,
            },
            None => false,
        }
    }

    pub fn refine(&self, v: &Self) -> Self {
        if self.super_of(v) {
            return v.clone();
        }
        let mut graph = HashMap::new();
        graph.insert("F".to_string(), ConstraintNode::F);
        Constraint {
            graph: Arc::new(graph),
            entry: "F".to_string(),
        }
    }

    fn check_subsumption(
        constraint_a: &Self,
        constraint_b: &Self,
        node_a: &ConstraintNode,
        node_b: &ConstraintNode,
        assumption: &mut HashSet<(ConstraintNode, ConstraintNode)>,
    ) -> bool {
        println!("Checking {:?} >= {:?}", node_a, node_b);
        let result = Constraint::check_subsumption_inner(
            constraint_a,
            constraint_b,
            node_a,
            node_b,
            assumption,
        );
        println!(
            "Result of subsumption check: {:?} >= {:?} is {:?}",
            node_a, node_b, result
        );
        result
    }

    /// a >= b
    fn check_subsumption_inner(
        constraint_a: &Self,
        constraint_b: &Self,
        node_a: &ConstraintNode,
        node_b: &ConstraintNode,
        assumption: &mut HashSet<(ConstraintNode, ConstraintNode)>,
    ) -> bool {
        if assumption.contains(&(node_a.clone(), node_b.clone())) {
            return true;
        }

        match (node_a, node_b) {
            (ConstraintNode::T, _) => true,
            (_, ConstraintNode::F) => true,
            (ConstraintNode::F, _) => false,
            (_, ConstraintNode::T) => false,
            (ConstraintNode::Leaf(a_lit), ConstraintNode::Leaf(b_lit)) => a_lit == b_lit,

            // 这一行是用来避免歧义的
            (ConstraintNode::Enum(a_nodes), ConstraintNode::Enum(b_nodes)) => {
                for b_node in b_nodes {
                    let mut subsumed = false;
                    for a_node in a_nodes {
                        if Constraint::check_subsumption(
                            constraint_a,
                            constraint_b,
                            a_node,
                            b_node,
                            assumption,
                        ) {
                            subsumed = true;
                            break;
                        }
                    }
                    if !subsumed {
                        return false;
                    }
                }
                true
            }
            (a, ConstraintNode::Enum(b_nodes)) => {
                for b_node in b_nodes {
                    if !Constraint::check_subsumption(
                        constraint_a,
                        constraint_b,
                        a,
                        b_node,
                        assumption,
                    ) {
                        return false;
                    }
                }
                true
            }
            (ConstraintNode::Enum(a_nodes), b) => {
                for a_node in a_nodes {
                    if Constraint::check_subsumption(
                        constraint_a,
                        constraint_b,
                        a_node,
                        b,
                        assumption,
                    ) {
                        return true;
                    }
                }
                false
            }

            (ConstraintNode::Pair(a_left, a_right), ConstraintNode::Pair(b_left, b_right)) => {
                if Constraint::check_subsumption(
                    constraint_a,
                    constraint_b,
                    a_left,
                    b_left,
                    assumption,
                ) && Constraint::check_subsumption(
                    constraint_a,
                    constraint_b,
                    a_right,
                    b_right,
                    assumption,
                ) {
                    return true;
                }
                false
            }

            // 这一行是用来避免歧义的
            (ConstraintNode::Def(a), ConstraintNode::Def(b)) => {
                // 这种情况下，显然假设集中不包含，那么我们在假设集中加入假设
                println!("Assuming {} >= {}", a, b);
                assumption.insert((node_a.clone(), node_b.clone()));
                // 然后解包
                let a = constraint_a
                    .get_node(a)
                    .expect("Failed to find definition in constraint_a");
                let b = constraint_b
                    .get_node(b)
                    .expect("Failed to find definition in constraint_b");
                let result =
                    Constraint::check_subsumption(constraint_a, constraint_b, a, b, assumption);
                assumption.remove(&(node_a.clone(), node_b.clone()));
                result
            }
            (ConstraintNode::Def(a), b) => {
                // 这种情况下，显然假设集中不包含，那么我们在假设集中加入假设
                println!("Assuming {} >= {:?}", a, b);
                assumption.insert((node_a.clone(), b.clone()));
                // 然后解包
                let a = constraint_a
                    .get_node(a)
                    .expect("Failed to find definition in constraint_a");
                let result =
                    Constraint::check_subsumption(constraint_a, constraint_b, a, b, assumption);
                assumption.remove(&(node_a.clone(), b.clone()));
                result
            }
            (a, ConstraintNode::Def(b)) => {
                // 这种情况下，显然假设集中不包含，那么我们在假设集中加入假设
                println!("Assuming {:?} >= {}", a, b);
                assumption.insert((a.clone(), node_b.clone()));
                // 然后解包
                let b = constraint_b
                    .get_node(b)
                    .expect("Failed to find definition in constraint_b");
                let result =
                    Constraint::check_subsumption(constraint_a, constraint_b, a, b, assumption);
                assumption.remove(&(a.clone(), node_b.clone()));
                result
            }
            _ => false,
        }
    }
}

struct PrettyFormatter<'a> {
    constraint: &'a Constraint,
    indent_level: usize,
    visited_defs: HashSet<String>, // 用于防止递归定义的无限循环
}

impl<'a> PrettyFormatter<'a> {
    fn new(constraint: &'a Constraint) -> Self {
        PrettyFormatter {
            constraint,
            indent_level: 0,
            visited_defs: HashSet::new(),
        }
    }

    fn format_node(&mut self, f: &mut fmt::Formatter<'_>, node: &ConstraintNode) -> fmt::Result {
        match node {
            ConstraintNode::T => write!(f, "T"),
            ConstraintNode::F => write!(f, "F"),
            ConstraintNode::Leaf(atomic) => match atomic {
                AtomicConstraint::Nil => write!(f, "Nil"),
                AtomicConstraint::LiteralInt(i) => write!(f, "{}", i),
            },
            ConstraintNode::Pair(left, right) => {
                write!(f, "(")?;
                self.format_node(f, left)?;
                write!(f, ", ")?;
                self.format_node(f, right)?;
                write!(f, ")")
            }
            ConstraintNode::Enum(variants) => {
                if variants.is_empty() {
                    return write!(f, "Never"); // 类似于 Rust 的 `!` 类型
                }

                // 如果 Enum 很简单，可以放在一行
                let is_simple = variants
                    .iter()
                    .all(|v| matches!(v, ConstraintNode::Leaf(_)));

                if is_simple && variants.len() <= 3 {
                    self.format_node(f, &variants[0])?;
                    for variant in variants.iter().skip(1) {
                        write!(f, " | ")?;
                        self.format_node(f, variant)?;
                    }
                    Ok(())
                } else {
                    // 复杂 Enum，换行并缩进
                    self.indent_level += 1;
                    for (i, variant) in variants.iter().enumerate() {
                        if i > 0 {
                            write!(f, " |")?;
                        }
                        writeln!(f)?;
                        write!(f, "{}", "  ".repeat(self.indent_level))?;
                        self.format_node(f, variant)?;
                    }
                    self.indent_level -= 1;
                    writeln!(f)?;
                    write!(f, "{}", "  ".repeat(self.indent_level))
                }
            }
            ConstraintNode::Def(name) => {
                // 如果我们已经访问过这个定义，说明遇到了递归，只打印名字
                if self.visited_defs.contains(name) {
                    write!(f, "{}", name)
                } else {
                    // 否则，打印名字并展开它的定义
                    self.visited_defs.insert(name.clone());
                    write!(f, "{}", name)?;
                    if let Some(defined_node) = self.constraint.get_node(name) {
                        write!(f, " := ")?;
                        self.format_node(f, defined_node)?;
                    }
                    // 注意：这里我们不从 visited_defs 中移除，
                    // 因为在一个格式化任务中，一个定义展开一次就够了。
                    Ok(())
                }
            }
        }
    }
}

// 为 Constraint 实现 Display trait
impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut formatter = PrettyFormatter::new(self);
        // 从入口点开始格式化
        if let Some(entry_node) = self.get_node(self.entry()) {
            formatter.format_node(f, entry_node)
        } else {
            write!(
                f,
                "Error: Entry '{}' not found in constraint graph.",
                self.entry()
            )
        }
    }
}

impl PartialEq for Constraint {
    fn eq(&self, other: &Self) -> bool {
        self.super_of(other) && other.super_of(self)
    }
}
