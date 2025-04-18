use crate::{BuildQuery, Erased, Rule, TypeKey, Val};
use colored::Colorize;
use std::fmt::Debug;

#[derive(Clone)]
pub struct Fact<F: BuildQuery> {
    pub(crate) fact_key: TypeKey,
    pub(crate) fact_name: &'static str,
    pub left: Val<F::Left>,
    pub right: Val<F::Right>,
    pub trace: Option<Trace>,
}

impl Fact<Erased> {
    pub fn new<F: BuildQuery>(
        left: &Val<F::Left>,
        right: &Val<F::Right>,
        description: Option<impl AsRef<str>>,
    ) -> Self {
        Fact {
            fact_key: TypeKey::of::<F>(),
            fact_name: F::NAME,
            left: left.erase(),
            right: right.erase(),
            trace: description.map(|description| Trace::Custom(description.as_ref().to_string())),
        }
    }

    pub(crate) fn cast<F: BuildQuery>(&self) -> Fact<F> {
        Fact {
            fact_key: self.fact_key,
            fact_name: self.fact_name,
            left: self.left.cast(),
            right: self.right.cast(),
            trace: self.trace.clone(),
        }
    }
}

impl<F: BuildQuery> Debug for Fact<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({:?}, {:?})", self.fact_name, self.left, self.right)
    }
}

impl<F: BuildQuery> Fact<F> {
    pub fn to_trace_string(&self) -> String {
        let mut buf = Vec::new();
        self.write_trace(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    pub fn write_trace(&self, f: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.write_trace_inner(0, f)
    }

    fn write_trace_inner(&self, level: usize, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        let indent = "  ".repeat(level);

        let mut fact = format!("{self:?}");
        if level == 0 {
            fact = fact.bold().to_string();
        }

        write!(w, "{indent}{}", fact)?;
        if let Some(trace) = &self.trace {
            match trace {
                Trace::Custom(description) => {
                    write!(w, "{}", format!(" - {}", description).dimmed())?
                }
                Trace::Rule { rule, dependencies } => {
                    write!(w, "{}", format!(" - {}", rule.label).dimmed())?;

                    for dep in dependencies {
                        writeln!(w)?;
                        dep.write_trace_inner(level + 1, w)?;
                    }
                }
            }
        }

        if level == 0 {
            writeln!(w)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Trace {
    Custom(String),
    Rule {
        rule: Rule,
        dependencies: Vec<Fact<Erased>>,
    },
}
