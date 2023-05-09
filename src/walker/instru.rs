// instrument the AST to give useful insight to detection modules

use ethers_solc::artifacts::{
    lowfidelity::TypedAst,
    visitor::{VisitError, Visitable, Visitor},
    EmitStatement,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum AstNode {
    EmitStatement(EmitStatement),
}

#[derive(Default, Clone)]
pub struct InstruModule {
    instru: BTreeMap<usize, AstNode>,
}

impl Visitor<()> for InstruModule {
    fn shared_data(&mut self) -> &() {
        &()
    }

    fn visit_emit_statement(
        &mut self,
        emit_statement: &mut EmitStatement,
    ) -> eyre::Result<(), VisitError> {
        self.instru.insert(
            emit_statement.id,
            AstNode::EmitStatement(emit_statement.clone()),
        );

        emit_statement.visit(self)
    }
}

#[derive(Debug, Clone)]
pub struct InstrumentedAst {
    pub instru: BTreeMap<usize, AstNode>,
    pub ast: TypedAst,
}

pub fn to_instru_ast(mut typed_ast: TypedAst) -> eyre::Result<InstrumentedAst> {
    let mut module = InstruModule::default();

    typed_ast.source_unit.visit(&mut module)?;

    Ok(InstrumentedAst {
        instru: module.instru,
        ast: typed_ast,
    })
}
