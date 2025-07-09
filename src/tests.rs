#[cfg(test)]
mod tests {
    use crate::constraint::Constraint;

    #[test]
    fn test1() {
        let tuple = Constraint::make_union(vec![
            Constraint::make_pair(Constraint::LiteralInt(0), Constraint::LiteralInt(2)),
            Constraint::make_pair(Constraint::LiteralInt(1), Constraint::LiteralFloat(3.14)),
            Constraint::make_pair(Constraint::LiteralInt(2), Constraint::LiteralBool(true)),
            Constraint::make_pair(
                Constraint::LiteralInt(3),
                Constraint::LiteralString("hello".to_string()),
            ),
        ]);

        let index0 = Constraint::make_pair(Constraint::LiteralInt(0), Constraint::Top);

        let value0 = tuple.intersection(&index0);
        println!("Tuple at index 0: {:?}", value0);
    }
}
