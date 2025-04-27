use crate::{BuildQuery, Erased, Fact, Query, Rule, Rules, Trace, Var};
use std::{
    any::TypeId,
    collections::BTreeMap,
    fmt::Debug,
    io::{Write, stdout},
    marker::PhantomData,
    sync::{
        Arc, LazyLock,
        atomic::{self, AtomicU32},
    },
};

pub struct Val<T> {
    description: Arc<str>,
    counter: u32,
    _data: PhantomData<T>,
}

pub fn val<T>(description: &str) -> Val<T> {
    Val::new(description)
}

impl<T> Val<T> {
    pub fn new(description: &str) -> Self {
        static NEXT: LazyLock<AtomicU32> = LazyLock::new(|| AtomicU32::new(0));

        Val {
            description: Arc::from(description),
            counter: NEXT.fetch_add(1, atomic::Ordering::Relaxed),
            _data: PhantomData,
        }
    }

    pub(crate) fn erase(&self) -> Val<Erased> {
        Val {
            description: self.description.clone(),
            counter: self.counter,
            _data: PhantomData,
        }
    }
}

impl Val<Erased> {
    pub(crate) fn cast<T>(&self) -> Val<T> {
        Val {
            description: self.description.clone(),
            counter: self.counter,
            _data: PhantomData,
        }
    }
}

impl<T> Debug for Val<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl<T> Clone for Val<T> {
    fn clone(&self) -> Self {
        Self {
            description: self.description.clone(),
            counter: self.counter,
            _data: PhantomData,
        }
    }
}

impl<T> PartialEq for Val<T> {
    fn eq(&self, other: &Self) -> bool {
        self.counter == other.counter
    }
}

impl<T> Eq for Val<T> {}

#[derive(Debug, Clone, Default)]
pub struct Context {
    facts: BTreeMap<TypeId, Vec<Fact<Erased>>>,
}

impl Context {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn all(&self) -> impl Iterator<Item = &Fact<Erased>> {
        self.facts.values().flatten()
    }

    pub fn get<F: BuildQuery<Left: Sized, Right: Sized>>(
        &self,
        query: Query<F>,
    ) -> impl Iterator<Item = Fact<F>> {
        self.facts
            .get(&query.fact_key.type_id())
            .into_iter()
            .flatten()
            .map(Fact::cast)
            .filter(move |fact| filter_by(fact, &query))
    }

    pub fn add(&mut self, fact: Fact<Erased>) -> bool {
        self.extend([fact])
    }

    pub fn extend(&mut self, facts: impl IntoIterator<Item = Fact<Erased>>) -> bool {
        let mut did_add = false;
        for fact in facts {
            let query = Query::<Erased>::raw(
                fact.fact_key,
                Some(fact.left.clone()),
                Some(fact.right.clone()),
            );

            if self.get(query).next().is_none() {
                self.facts
                    .entry(fact.fact_key.type_id())
                    .or_default()
                    .push(fact);

                did_add = true;
            }
        }

        did_add
    }

    pub fn run(&mut self, rules: Rules) {
        loop {
            let mut new_facts = Vec::new();
            for rule in rules.iter() {
                let first = &rule.plan.first;

                let facts = self
                    .get(Query::<Erased>::raw(first.fact_key, None, None))
                    .collect::<Vec<_>>();

                for fact in &facts {
                    let mut vars = vec![None; rule.plan.vars];
                    vars[first.left.index] = Some(fact.left.clone());

                    self.run_rule(rule, 0, &mut vars, Vec::new(), &mut new_facts);
                }
            }

            let did_add = self.extend(new_facts);

            if !did_add {
                break;
            }
        }
    }

    fn run_rule(
        &mut self,
        rule: &Rule,
        index: usize,
        vars: &mut [Option<Val<Erased>>],
        dependencies: Vec<Fact<Erased>>,
        new_facts: &mut Vec<Fact<Erased>>,
    ) {
        if let Some(step) = rule.plan.steps.get(index) {
            let left = vars[step.left.index].as_ref().cloned();
            let right = vars[step.right.index].as_ref().cloned();

            let facts = self
                .get(Query::<Erased>::raw(step.fact_key, left, right))
                .collect::<Vec<_>>();

            for fact in facts {
                let mut vars = vars.to_vec();
                vars[step.right.index] = Some(fact.right.clone());

                let mut dependencies = dependencies.to_vec();
                dependencies.push(fact);

                self.run_rule(rule, index + 1, &mut vars, dependencies, new_facts);
            }
        } else {
            // We ran all the steps; produce the fact

            let last = &rule.plan.last;

            let get_var = |var: &Var<_>| vars[var.index].as_ref().expect("unknown var").clone();
            let left = get_var(&last.left);
            let right = get_var(&last.right);

            new_facts.push(Fact {
                fact_key: last.fact_key,
                fact_name: last.fact_name,
                left,
                right,
                trace: Some(Trace::Rule {
                    rule: rule.clone(),
                    dependencies,
                }),
            });
        }
    }

    pub fn print(&self) {
        let mut stdout = stdout().lock();

        for fact in self.all() {
            fact.write_trace(&mut stdout).unwrap();
            writeln!(stdout).unwrap();
        }
    }
}

fn filter_by<F: BuildQuery>(fact: &Fact<F>, query: &Query<F>) -> bool {
    query
        .left
        .as_ref()
        .is_none_or(|left| fact.left.erase() == *left)
        && query
            .right
            .as_ref()
            .is_none_or(|right| fact.right.erase() == *right)
}
