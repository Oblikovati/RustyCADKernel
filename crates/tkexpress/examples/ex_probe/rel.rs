//! `ex_rel`: equations and systems of tools/oracle/tkexpress_probe.cxx.
use super::common::ProbeLine;
use super::text::process;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkexpress::expr::ExprNamedExpression;
use rustycad_tkexpress::expr::package::nb_of_free_variables_relation;
use rustycad_tkexpress::expr::{
    ExprEqual, ExprHandle, ExprNumericValue, ExprRUIterator, ExprRelationHandle,
    ExprRelationIterator, ExprSystemRelation,
};
use rustycad_tkexpress::expr_intrp::ExprIntrpGenExp;

/// The equations of the text pairs, parsed by one interpreter.
fn equations(fields: &[&str]) -> Result<Vec<ExprRelationHandle>, Failure> {
    let interpreter: Handle<ExprIntrpGenExp> = ExprIntrpGenExp::create();
    let mut relations: Vec<ExprRelationHandle> = Vec::new();
    for pair in fields.as_chunks::<2>().0 {
        process(&interpreter, pair[0]);
        let left: ExprHandle = interpreter.expression()?;
        process(&interpreter, pair[1]);
        let right: ExprHandle = interpreter.expression()?;
        relations.push(ExprEqual::new(&left, &right).cast());
    }
    Ok(relations)
}

/// The relation of `ex_rel`: the one equation, or the system of all of them.
fn relation_of(relations: &[ExprRelationHandle]) -> ExprRelationHandle {
    if relations.len() == 1 {
        return relations[0].clone();
    }
    let system: Handle<ExprSystemRelation> = ExprSystemRelation::new(&relations[0]);
    for relation in &relations[1..] {
        system.add(relation);
    }
    system.cast()
}

/// The iterator part of the relation battery.
fn iterations(line: &mut ProbeLine, rel: &ExprRelationHandle) {
    line.step(|line| {
        let mut relations: ExprRelationIterator = ExprRelationIterator::new(rel)?;
        while relations.more() {
            line.text(&relations.value()?.string()?);
            relations.next()?;
        }
        Ok(())
    });
    line.step(|line| {
        let mut unknowns: ExprRUIterator = ExprRUIterator::new(rel)?;
        while unknowns.more() {
            line.text(&unknowns.value()?.get_name());
            unknowns.next()?;
        }
        Ok(())
    });
}

/// The `Remove` quirks of a system, nested steps as natively.
fn removals(line: &mut ProbeLine, rel: &ExprRelationHandle, first: &ExprRelationHandle) {
    let system: Handle<ExprSystemRelation> = rel.cast();
    let Some(system) = system.get() else {
        return;
    };
    line.step(|_| system.remove(first));
    let one: ExprHandle = ExprNumericValue::expression(1.0);
    let other: ExprRelationHandle = ExprEqual::new(&one, &ExprNumericValue::expression(1.0)).cast();
    line.step(|_| system.remove(&other));
}

/// The mutating tail of the relation battery: replace the first unknown, simplify.
fn mutations(line: &mut ProbeLine, rel: &ExprRelationHandle) {
    line.step(|line| {
        let unknowns: ExprRUIterator = ExprRUIterator::new(rel)?;
        if unknowns.more() {
            let x = unknowns.value()?;
            line.boolean(rel.contains(&x.cast())?);
            rel.replace(&x, &ExprNumericValue::expression(2.0))?;
            line.text(&rel.string()?);
        }
        Ok(())
    });
    line.step(|line| {
        rel.simplify()?;
        line.text(&rel.string()?);
        line.boolean(rel.is_satisfied()?);
        Ok(())
    });
}

/// `ex_rel <left> <right> ...`.
pub fn rel_case(fields: &[&str]) {
    let mut line: ProbeLine = ProbeLine::default();
    let relations: Vec<ExprRelationHandle> = equations(fields).unwrap_or_default();
    if relations.is_empty() {
        line.token("!ex_rel_needs_valid_texts");
        line.end();
        return;
    }
    let rel: ExprRelationHandle = relation_of(&relations);
    line.step(|line| Ok(line.text(&rel.string()?)));
    line.step(|line| Ok(line.boolean(rel.is_satisfied()?)));
    line.step(|line| Ok(line.boolean(rel.is_linear()?)));
    line.step(|line| {
        line.int(rel.nb_of_sub_relations());
        line.int(rel.nb_of_single_relations()?);
        Ok(())
    });
    iterations(&mut line, &rel);
    line.step(|line| Ok(line.int(nb_of_free_variables_relation(&rel)?)));
    line.step(|line| Ok(line.text(&rel.simplified()?.string()?)));
    line.step(|line| Ok(line.text(&rel.copy()?.string()?)));
    line.step(|line| Ok(line.text(&rel.sub_relation(1)?.string()?)));
    removals(&mut line, &rel, &relations[0]);
    mutations(&mut line, &rel);
    line.end();
}
