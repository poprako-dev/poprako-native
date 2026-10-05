use poprako_orchestra::Oper;

use crate::data::recovery::{MutationState, RecoveryScope};

#[derive(Oper)]
#[oper(output = MutationState)]
pub struct ReadMutationState {
    pub scope: RecoveryScope,
}
