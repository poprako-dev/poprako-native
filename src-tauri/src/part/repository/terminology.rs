use poprako_orchestra::Oper;

use crate::data::terminology::portable::PortableTermbase;
use crate::data::terminology::{
    TermMetadata, TermbaseInfo, TermbaseMetadata, TerminologyContext, TerminologyQuery,
};
use crate::model::terminology::{Term, Termbase, TermbaseScope};

#[derive(Oper)]
#[oper(output = TermbaseScope)]
pub struct ResolveTerminologyScope {
    pub access: TerminologyContext,
}

#[derive(Oper)]
#[oper(output = Term)]
pub struct ReadTerm {
    pub access: TerminologyContext,
    pub id: String,
}

#[derive(Oper)]
#[oper(output = Vec<TermbaseInfo>)]
pub struct ListTermbaseInfos {
    pub access: TerminologyContext,
    pub query: TerminologyQuery,
}

#[derive(Oper)]
#[oper(output = TermbaseInfo)]
pub struct ReadTermbase {
    pub access: TerminologyContext,
    pub id: String,
}

#[derive(Oper)]
#[oper(output = PortableTermbase)]
pub struct ReadTermbaseExport {
    pub access: TerminologyContext,
    pub id: String,
}

#[derive(Oper)]
#[oper(output = Vec<Term>)]
pub struct ListTermInfos {
    pub access: TerminologyContext,
    pub termbase_id: String,
    pub query: TerminologyQuery,
}

#[derive(Oper)]
#[oper(output = Termbase)]
pub struct CreateTermbase {
    pub access: TerminologyContext,
    pub scope: TermbaseScope,
    pub id: String,
    pub metadata: TermbaseMetadata,
}

#[derive(Oper)]
#[oper(output = Termbase)]
pub struct UpdateTermbase {
    pub access: TerminologyContext,
    pub baseline: Termbase,
    pub metadata: TermbaseMetadata,
}

#[derive(Oper)]
#[oper(output = ())]
pub struct DeleteTermbase {
    pub access: TerminologyContext,
    pub baseline: Termbase,
}

#[derive(Oper)]
#[oper(output = Term)]
pub struct CreateTerm {
    pub access: TerminologyContext,
    pub termbase_id: String,
    pub id: String,
    pub metadata: TermMetadata,
}

#[derive(Oper)]
#[oper(output = Termbase)]
pub struct ImportTermbase {
    pub base: CreateTermbase,
    pub terms: Vec<CreateTerm>,
}

#[derive(Oper)]
#[oper(output = Term)]
pub struct UpdateTerm {
    pub access: TerminologyContext,
    pub baseline: Term,
    pub metadata: TermMetadata,
}

#[derive(Oper)]
#[oper(output = ())]
pub struct DeleteTerm {
    pub access: TerminologyContext,
    pub baseline: Term,
}
