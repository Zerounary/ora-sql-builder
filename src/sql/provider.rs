use super::StatementType;

pub trait SQLProvider {
    fn get_statement_type(&self) -> StatementType;
    fn get_select(&mut self) -> Vec<String>;
    fn get_tables(&mut self) -> Vec<String>;
    fn get_join(&self) -> Vec<String>;
    fn get_inner_join(&self) -> Vec<String>;
    fn get_outer_join(&self) -> Vec<String>;
    fn get_left_outer_join(&mut self) -> Vec<String>;
    fn get_right_outer_join(&self) -> Vec<String>;
    fn get_where(&mut self) -> Vec<String>;
    fn get_having(&self) -> Vec<String>;
    fn get_group_by(&mut self) -> Vec<String>;
    fn get_order_by(&mut self) -> Vec<String>;
    fn get_columns(&self) -> Vec<String>;
    fn get_values(&self) -> Vec<String>;
    fn get_sets(&self) -> Vec<String>;
}
