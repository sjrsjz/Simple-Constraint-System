#[cfg(test)]
mod tests {
    use crate::constraint::Constraint;

    #[test]
    fn test1() {
        let tuple = Constraint::make_union(vec![
            Constraint::make_pair(Constraint::LiteralInt(0), Constraint::LiteralInt(2)),
            Constraint::make_pair(Constraint::LiteralInt(1), Constraint::LiteralFloat(3.14)),
            Constraint::make_pair(
                Constraint::LiteralInt(3),
                Constraint::LiteralString("hello".to_string()),
            ),
        ]);

        let index0 = Constraint::make_pair(Constraint::LiteralInt(0), Constraint::Top);

        let value0 = tuple.intersection(&index0);
        println!("Tuple at index 0: {:?}", value0);

        // 差集测试
        let value1 = tuple.difference(&index0);
        println!("Tuple difference: {:?}", value1);
        // 更多差集测试
        let index1 = Constraint::make_pair(Constraint::LiteralInt(1), Constraint::Top);
        let value2 = tuple.difference(&index1);
        println!("Tuple difference (remove index 1): {:?}", value2);

        let index2 = Constraint::make_pair(Constraint::LiteralInt(2), Constraint::Top);
        let value3 = tuple.difference(&index2);
        println!("Tuple difference (remove index 2): {:?}", value3);

        // 移除不存在的索引
        let index_not_exist = Constraint::make_pair(Constraint::LiteralInt(99), Constraint::Top);
        let value4 = tuple.difference(&index_not_exist);
        println!("Tuple difference (remove non-existent index): {:?}", value4);

        // 连续移除多个索引
        let tuple2 = value2.difference(&index2);
        println!("Tuple difference (remove index 1 then 2): {:?}", tuple2);

        // 复杂嵌套差集测试
        // 构造嵌套Tuple和Union
        let nested_tuple = Constraint::make_union(vec![
            Constraint::make_pair(
                Constraint::LiteralInt(0),
                Constraint::make_tuple(vec![
                    Constraint::LiteralInt(10),
                    Constraint::LiteralFloat(2.71),
                ]),
            ),
            Constraint::make_pair(
                Constraint::LiteralInt(1),
                Constraint::make_union(vec![
                    Constraint::LiteralString("a".to_string()),
                    Constraint::LiteralString("b".to_string()),
                ]),
            ),
            Constraint::make_pair(
                Constraint::LiteralInt(2),
                Constraint::make_tuple(vec![
                    Constraint::make_union(vec![
                        Constraint::LiteralInt(100),
                        Constraint::LiteralInt(200),
                    ]),
                    Constraint::LiteralFloat(3.14),
                ]),
            ),
        ]);

        // 差集：移除嵌套的某个元素
        let nested_index = Constraint::make_pair(
            Constraint::LiteralInt(2),
            Constraint::make_tuple(vec![
                Constraint::LiteralInt(100),
                Constraint::LiteralFloat(3.14),
            ]),
        );
        let nested_diff = nested_tuple.difference(&nested_index);
        println!("Nested tuple difference (remove nested index): {:?}", nested_diff);

        // 差集：移除嵌套Union中的某个分支
        let nested_index2 = Constraint::make_pair(
            Constraint::LiteralInt(1),
            Constraint::LiteralString("a".to_string()),
        );
        let nested_diff2 = nested_tuple.difference(&nested_index2);
        println!("Nested tuple difference (remove union branch): {:?}", nested_diff2);

        // 差集：移除嵌套Tuple的全部内容
        let nested_index3 = Constraint::make_pair(
            Constraint::LiteralInt(0),
            Constraint::make_tuple(vec![
                Constraint::LiteralInt(10),
                Constraint::LiteralFloat(2.71),
            ]),
        );
        let nested_diff3 = nested_tuple.difference(&nested_index3);
        println!("Nested tuple difference (remove full nested tuple): {:?}", nested_diff3);

        // 差集：移除不存在的嵌套内容
        let nested_index4 = Constraint::make_pair(
            Constraint::LiteralInt(2),
            Constraint::make_tuple(vec![
                Constraint::LiteralInt(999),
                Constraint::LiteralFloat(3.14),
            ]),
        );
        let nested_diff4 = nested_tuple.difference(&nested_index4);
        println!("Nested tuple difference (remove non-existent nested): {:?}", nested_diff4);
    }
}
