use std::fmt::{Display, Formatter};

#[derive(Debug, Default)]
pub enum Sorting {
    #[default]
    DESC,
    ASC,
}
impl Display for Sorting {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Sorting::DESC => "DESC",
            Sorting::ASC => "ASC",
        })
    }
}
#[derive(Debug, Default)]
pub struct QueryOptions {
    pub sorting: Sorting,
    pub limit: Option<u64>,
}
impl QueryOptions {
    pub fn new(limit: Option<u64>, sorting: Sorting) -> Self {
        QueryOptions { limit, sorting }
    }
}
