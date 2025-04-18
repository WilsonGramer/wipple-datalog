use crate::{Erased, TypeKey, Val};
use std::{fmt::Debug, marker::PhantomData};

pub struct Var<T> {
    pub(crate) index: usize,
    _data: PhantomData<T>,
}

impl<T> Debug for Var<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Var").field(&self.index).finish()
    }
}

impl<T> Clone for Var<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Var<T> {}

impl<T> PartialEq for Var<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for Var<T> {}

impl<T> Var<T> {
    pub const fn new(index: usize) -> Self {
        Var {
            index,
            _data: PhantomData,
        }
    }

    const fn erase(&self) -> Var<Erased> {
        Var {
            index: self.index,
            _data: PhantomData,
        }
    }
}

pub trait BuildQuery: 'static {
    type Left;
    type Right;

    const NAME: &'static str;
}

impl BuildQuery for Erased {
    type Left = Erased;
    type Right = Erased;

    const NAME: &'static str = "<erased>";
}

#[derive(Debug, Clone)]
pub struct QueryBuilder {
    pub left: Var<Erased>,
    pub right: Result<Val<Erased>, Var<Erased>>,
    pub dependencies: &'static [QueryBuilder],
}

#[derive(Debug, Clone)]
pub struct Query<F: BuildQuery> {
    pub(crate) fact_key: TypeKey,
    pub(crate) left: Option<Val<Erased>>,
    pub(crate) right: Option<Val<Erased>>,
    _data: PhantomData<F>,
}

impl<F: BuildQuery> Query<F> {
    pub fn new(left: Option<Val<F::Left>>, right: Option<Val<F::Right>>) -> Self {
        Query::raw(
            TypeKey::of::<F>(),
            left.as_ref().map(Val::erase),
            right.as_ref().map(Val::erase),
        )
    }

    pub(crate) fn raw(
        fact_key: TypeKey,
        left: Option<Val<Erased>>,
        right: Option<Val<Erased>>,
    ) -> Self {
        Query {
            fact_key,
            left,
            right,
            _data: PhantomData,
        }
    }

    pub fn all() -> Self {
        Query::new(None, None)
    }
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub vars: usize,
    pub first: PlanFirstStep,
    pub steps: &'static [PlanStep],
    pub last: PlanLastStep<Val<Erased>>,
}

type PlanFirstStep = Step<PhantomData<Erased>>;
type PlanStep = Step<Var<Erased>>;
type PlanLastStep<Output> = Step<Result<Output, Var<Erased>>>;

#[derive(Debug, Clone)]
pub struct Step<Right> {
    pub(crate) fact_key: TypeKey,
    pub(crate) fact_name: &'static str,
    pub(crate) left: Var<Erased>,
    pub(crate) right: Right,
}

impl PlanFirstStep {
    pub const fn first<F: BuildQuery>(left: Var<F::Left>) -> Self {
        Step {
            fact_key: TypeKey::of::<F>(),
            fact_name: F::NAME,
            left: left.erase(),
            right: PhantomData,
        }
    }
}

impl PlanStep {
    pub const fn plan<F: BuildQuery>(left: Var<F::Left>, right: Var<F::Right>) -> Self {
        Step {
            fact_key: TypeKey::of::<F>(),
            fact_name: F::NAME,
            left: left.erase(),
            right: right.erase(),
        }
    }
}

impl<Output> PlanLastStep<Output> {
    pub const fn last_var<F: BuildQuery>(left: Var<F::Left>, right: Var<F::Right>) -> Self {
        Step {
            fact_key: TypeKey::of::<F>(),
            fact_name: F::NAME,
            left: left.erase(),
            right: Err(right.erase()),
        }
    }

    pub const fn last_val<F: BuildQuery<Right = Output>>(
        left: Var<F::Left>,
        right: F::Right,
    ) -> Self {
        Step {
            fact_key: TypeKey::of::<F>(),
            fact_name: F::NAME,
            left: left.erase(),
            right: Ok(right),
        }
    }
}

pub trait BuildRule {
    const LABEL: &str;
    const PLAN: Plan;
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub label: &'static str,
    pub plan: Plan,
}

impl Rule {
    pub const fn new<T: BuildRule>() -> Self {
        Rule {
            label: T::LABEL,
            plan: T::PLAN,
        }
    }
}

#[derive(Default)]
pub struct Rules(Vec<Rule>);

impl Rules {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add<T: BuildRule>(mut self) -> Self {
        self.0.push(Rule::new::<T>());
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = &Rule> {
        self.0.iter()
    }
}
