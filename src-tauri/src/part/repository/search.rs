use poprako_orchestra::Oper;

use crate::data::search::{ReplaceUnits, ReplacementResult, SearchHit, SearchUnits};

#[derive(Oper)]
#[oper(output = Vec<SearchHit>)]
pub struct Search {
    pub input: SearchUnits,
}

#[derive(Oper)]
#[oper(output = ReplacementResult)]
pub struct Replace {
    pub input: ReplaceUnits,
}
