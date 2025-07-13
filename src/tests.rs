#[cfg(test)]
mod tests {
    use crate::constraint::{AtomicConstraint, Constraint, ConstraintNode};

    #[test]
    fn test1() {
        let mut list_def = Constraint::new("L_T".to_string());
        list_def.add_node(
            "L_T".to_string(),
            crate::constraint::ConstraintNode::Enum(vec![
                ConstraintNode::Pair(
                    ConstraintNode::T.into(),
                    ConstraintNode::Def("L_T".to_string()).into(),
                ),
                ConstraintNode::Leaf(AtomicConstraint::Nil).into(),
            ]),
        );

        let mut simple_list = Constraint::new("S_L".to_string());
        simple_list.add_node(
            "S_L".to_string(),
            ConstraintNode::Pair(
                ConstraintNode::Leaf(AtomicConstraint::LiteralInt(1)).into(),
                ConstraintNode::Pair(
                    ConstraintNode::Leaf(AtomicConstraint::LiteralInt(1)).into(),
                    ConstraintNode::T.into(),
                )
                .into(),
            ),
        );

        println!("List definition: {}", list_def);
        println!("Simple list: {}", simple_list);

        assert_eq!(list_def.super_of(&simple_list), false);
    }

    #[test]
    fn test2() {
        let mut list_def = Constraint::new("L_T".to_string());
        list_def.add_node(
            "L_T".to_string(),
            crate::constraint::ConstraintNode::Enum(vec![
                ConstraintNode::Pair(
                    ConstraintNode::T.into(),
                    ConstraintNode::Def("L_T".to_string()).into(),
                ),
                ConstraintNode::Leaf(AtomicConstraint::Nil).into(),
            ]),
        );

        let mut simple_list = Constraint::new("S_L".to_string());
        simple_list.add_node(
            "S_L".to_string(),
            ConstraintNode::Pair(
                ConstraintNode::Leaf(AtomicConstraint::LiteralInt(1)).into(),
                ConstraintNode::Pair(
                    ConstraintNode::Leaf(AtomicConstraint::LiteralInt(1)).into(),
                    ConstraintNode::Leaf(AtomicConstraint::Nil).into(),
                )
                .into(),
            ),
        );

        println!("List definition: {}", list_def);
        println!("Simple list: {}", simple_list);

        assert_eq!(list_def.super_of(&simple_list), true);
    }

    #[test]
    fn test3() {
        let mut list_def = Constraint::new("L_T".to_string());
        list_def.add_node(
            "L_T".to_string(),
            crate::constraint::ConstraintNode::Enum(vec![
                ConstraintNode::Pair(
                    ConstraintNode::T.into(),
                    ConstraintNode::Def("L_T".to_string()).into(),
                ),
                ConstraintNode::Leaf(AtomicConstraint::Nil).into(),
            ]),
        );

        let mut double_list_def = Constraint::new("L_T_2".to_string());
        double_list_def.add_node(
            "L_T_2".to_string(),
            crate::constraint::ConstraintNode::Enum(vec![
                ConstraintNode::Pair(
                    ConstraintNode::T.into(),
                    ConstraintNode::Pair(
                        ConstraintNode::T.into(),
                        ConstraintNode::Def("L_T_2".to_string()).into(),
                    )
                    .into(),
                ),
                ConstraintNode::Leaf(AtomicConstraint::Nil).into(),
            ]),
        );

        println!("List definition: {}", list_def);
        println!("Double list definition: {}", double_list_def);

        assert_eq!(list_def.super_of(&double_list_def), true);
        println!("----------");
        assert_eq!(double_list_def.super_of(&list_def), false);
    }
}
