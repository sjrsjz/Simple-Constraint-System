pub mod constraint;

use std::sync::Arc;

use constraint::*;
fn main() {
    let c1 = Constraint::Bound(Bound::Inclusive(1), Bound::Inclusive(10));
    let c2 = Constraint::LiteralInt(5);
    let c3 = c1.union(&c2);
    println!("c1 union c2 = {:?}", c3);

    let c4 = Constraint::Bound(Bound::Inclusive(1), Bound::Inclusive(10));
    let c5 = Constraint::LiteralInt(15);
    let c6 = c4.union(&c5);
    println!("c4 union c5 = {:?}", c6);

    let c7 = Constraint::Union(Arc::new(vec![
        Constraint::LiteralInt(1),
        Constraint::LiteralInt(3),
    ]));
    let c8 = Constraint::Union(Arc::new(vec![
        Constraint::LiteralInt(1),
        Constraint::LiteralInt(2),
        Constraint::LiteralInt(3),
    ]));
    println!("c8 super_of c7 = {}", c8.super_of(&c7));

    let c9 = Constraint::Union(Arc::new(vec![
        Constraint::LiteralInt(1),
        Constraint::LiteralInt(4),
    ]));
    println!("c8 super_of c9 = {}", c8.super_of(&c9));

    let c10 = Constraint::Bound(Bound::Inclusive(0), Bound::Inclusive(10));
    let c11 = Constraint::Union(Arc::new(vec![
        Constraint::LiteralInt(1),
        Constraint::LiteralInt(5),
    ]));
    println!("c10 super_of c11 = {}", c10.super_of(&c11));

    let c12 = Constraint::Union(Arc::new(vec![
        Constraint::Bound(Bound::Inclusive(0), Bound::Inclusive(5)),
        Constraint::Bound(Bound::Inclusive(5), Bound::Inclusive(10)),
    ]));
    let c13 = Constraint::Bound(Bound::Inclusive(0), Bound::Inclusive(10));
    println!("c12.union(&c13) = {:?}", c12.union(&c13));

    // 区间与区间并集
    let c14 = Constraint::Bound(Bound::Inclusive(1), Bound::Inclusive(5));
    let c15 = Constraint::Bound(Bound::Inclusive(6), Bound::Inclusive(10));
    let c16 = c14.union(&c15);
    println!("c14 union c15 = {:?}", c16);

    // 区间与单点，且单点在区间外
    let c17 = Constraint::Bound(Bound::Inclusive(1), Bound::Inclusive(5));
    let c18 = Constraint::LiteralInt(8);
    let c19 = c17.union(&c18);
    println!("c17 union c18 = {:?}", c19);

    // 区间与联合，联合中有区间外的点
    let c20 = Constraint::Union(Arc::new(vec![
        Constraint::LiteralInt(2),
        Constraint::LiteralInt(8),
    ]));
    let c21 = Constraint::Bound(Bound::Inclusive(1), Bound::Inclusive(5));
    let c22 = c20.union(&c21);
    println!("c20 union c21 = {:?}", c22);

    // 联合与联合，部分可合并
    let c23 = Constraint::Union(Arc::new(vec![
        Constraint::LiteralInt(1),
        Constraint::LiteralInt(2),
    ]));
    let c24 = Constraint::Union(Arc::new(vec![
        Constraint::LiteralInt(2),
        Constraint::LiteralInt(3),
    ]));
    let c25 = c23.union(&c24);
    println!("c23 union c24 = {:?}", c25);

    // 嵌套联合归约
    let c26 = Constraint::Union(Arc::new(vec![
        Constraint::Union(Arc::new(vec![
            Constraint::LiteralInt(1),
            Constraint::LiteralInt(2),
        ])),
        Constraint::LiteralInt(2),
        Constraint::LiteralInt(3),
    ]));
    println!("c26 reduce = {:?}", c26.reduce());

    // Top/Bottom 测试
    let c27 = Constraint::Top;
    let c28 = Constraint::Bottom;
    println!("Top union Bound = {:?}", c27.union(&c14));
    println!("Bottom union Bound = {:?}", c28.union(&c14));
    println!("Top super_of Bound = {}", c27.super_of(&c14));
    println!("Bound super_of Bottom = {}", c14.super_of(&c28));

    // Pair 类型测试
    let pair1 = Constraint::make_pair(
        Constraint::Bound(Bound::Inclusive(1), Bound::Inclusive(5)),
        Constraint::LiteralInt(10),
    );
    let pair2 = Constraint::make_pair(Constraint::LiteralInt(3), Constraint::LiteralInt(10));

    println!("pair1 union pair2 = {:?}", pair1.union(&pair2));
    println!("pair1 super_of pair2 = {}", pair1.super_of(&pair2));

    // 浮点区间测试
    let f1 = Constraint::FloatBound(FloatBound::Inclusive(0.0), FloatBound::Exclusive(1.0));
    let f2 = Constraint::LiteralFloat(0.5);
    println!(
        "FloatBound [0.0, 1.0) super_of LiteralFloat(0.5) = {}",
        f1.super_of(&f2)
    );

    let f3 = Constraint::FloatBound(FloatBound::Inclusive(0.5), FloatBound::Inclusive(2.0));
    let f4 = f1.union(&f3);
    println!("FloatBound [0.0, 1.0) union [0.5, 2.0] = {:?}", f4);

    // 浮点区间与整数混合测试
    let f5 = Constraint::FloatBound(FloatBound::Inclusive(-10.0), FloatBound::Inclusive(10.0));
    let i1 = Constraint::LiteralInt(5);
    println!(
        "FloatBound [-10.0, 10.0] super_of LiteralInt(5) = {}",
        f5.super_of(&i1)
    );

    let f6 = f5.union(&i1);
    println!("FloatBound [-10.0, 10.0] union LiteralInt(5) = {:?}", f6);

    // Float 类型与整数混合
    let float_type = Constraint::Float;
    let int_val = Constraint::LiteralInt(42);
    println!(
        "Float super_of LiteralInt(42) = {}",
        float_type.super_of(&int_val)
    );
    println!(
        "Float union LiteralInt(42) = {:?}",
        float_type.union(&int_val)
    );

    // 复杂联合类型测试
    let complex_union = Constraint::make_union(vec![
        Constraint::LiteralInt(1),
        Constraint::LiteralFloat(1.5),
        Constraint::FloatBound(FloatBound::Inclusive(2.0), FloatBound::Inclusive(3.0)),
        Constraint::LiteralBool(true),
        Constraint::LiteralString("hello".to_string()),
    ]);
    println!("Complex union = {:?}", complex_union);

    // 布尔类型测试
    let bool_union = Constraint::make_union(vec![
        Constraint::LiteralBool(true),
        Constraint::LiteralBool(false),
    ]);
    let bool_type = Constraint::Bool;
    println!(
        "Bool super_of Union[true, false] = {}",
        bool_type.super_of(&bool_union)
    );
    println!(
        "Bool union Union[true, false] = {:?}",
        bool_type.union(&bool_union)
    );

    // 嵌套 Pair 测试
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

    // 混合类型的复杂 Union 归约
    let mixed_union = Constraint::make_union(vec![
        Constraint::LiteralInt(1),
        Constraint::Int, // 应该吸收 LiteralInt(1)
        Constraint::LiteralFloat(2.5),
        Constraint::Float,  // 应该吸收 LiteralFloat(2.5) 和 Int
        Constraint::Bottom, // 应该被任何类型吸收
    ]);
    println!("Mixed union reduction = {:?}", mixed_union);

    // 区间边界测试
    let edge_case1 = Constraint::FloatBound(FloatBound::Exclusive(0.0), FloatBound::Inclusive(0.0));
    println!("Empty float bound is_empty = {}", {
        match &edge_case1 {
            Constraint::FloatBound(s, e) => FloatField::new(s.clone(), e.clone()).is_empty(),
            _ => false,
        }
    });

    // 精度边界测试
    let close_bounds = Constraint::make_union(vec![
        Constraint::FloatBound(FloatBound::Inclusive(0.0), FloatBound::Exclusive(1.0)),
        Constraint::FloatBound(FloatBound::Inclusive(1.0), FloatBound::Inclusive(2.0)),
    ]);
    println!("Close float bounds union = {:?}", close_bounds);

    // Top 和 Bottom 的复杂交互
    let top_union = Constraint::make_union(vec![
        Constraint::Top,
        Constraint::Int,
        Constraint::Float,
        Constraint::String,
    ]);
    println!("Union with Top = {:?}", top_union);

    let everything_union = Constraint::make_union(vec![
        Constraint::LiteralInt(1),
        Constraint::LiteralFloat(2.0),
        Constraint::LiteralBool(true),
        Constraint::LiteralString("test".to_string()),
        Constraint::Bound(Bound::Inclusive(10), Bound::Inclusive(20)),
        Constraint::FloatBound(FloatBound::Inclusive(30.0), FloatBound::Inclusive(40.0)),
        Constraint::make_pair(Constraint::Int, Constraint::String),
    ]);
    println!("Everything union = {:?}", everything_union);
    println!(
        "Top super_of everything = {}",
        Constraint::Top.super_of(&everything_union)
    );
}
