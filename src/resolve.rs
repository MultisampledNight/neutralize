use {
    super::{Color, LinkedScheme, Map, Metadata, Set, SlotName, Value},
    serde::Serialize,
    thiserror::Error,
};

#[derive(Clone, Debug, Error)]
pub enum ResolveError {
    #[error("Noticed a neverending loop of slot links, involved are {involved:?}")]
    InfiniteLoop { involved: Vec<SlotName> },
    #[error("{from:?} links to '{to}', which doesn't exist")]
    LinkToNonexistent { from: Vec<SlotName>, to: SlotName },
    #[error("Impossible to resolve '{subject}', which links to itself")]
    LinkToItself { subject: SlotName },
}

#[derive(Default)]
struct State {
    /// slots which are already done
    resolved: Map<SlotName, Color>,
    /// left slots which are depended on, right slots which depend on the left
    pending: Map<SlotName, Vec<SlotName>>,
    /// any loops/other errors which were detected while processing
    errors: Vec<ResolveError>,
}

impl State {
    /// Returns all resolved values, erroring if there's anything pending.
    fn resolved_or_errors(self) -> Result<Map<SlotName, Color>, Vec<ResolveError>> {
        if self.pending.is_empty() {
            Ok(self.resolved)
        } else {
            // ok, then let's analyze what happened
            // they've been detected as loops already and might issue false positives else
            let names_to_ignore: Set<_> = self
                .errors
                .clone()
                .into_iter()
                .flat_map(|err| match err {
                    ResolveError::InfiniteLoop { involved } => involved,
                    _ => {
                        unreachable!(
                            "no other error than InfiniteLoop could have been emitted in before"
                        )
                    }
                })
                .collect();

            Err(self
                .errors
                .into_iter()
                .chain(self.pending.into_iter().filter_map(|(target, dependents)| {
                    (!names_to_ignore.contains(&target)).then(|| {
                        // [0] is safe here, there's literally no case where an empty set depends
                        // on something
                        if dependents[0] == target {
                            ResolveError::LinkToItself { subject: target }
                        } else {
                            ResolveError::LinkToNonexistent {
                                from: dependents,
                                to: target,
                            }
                        }
                    })
                }))
                .collect())
        }
    }
}

impl TryFrom<LinkedScheme> for ResolvedScheme {
    type Error = Vec<ResolveError>;
    fn try_from(source: LinkedScheme) -> Result<Self, Vec<ResolveError>> {
        // this could be a fun benchmark, would it be better here to use a stack-based solution?
        // let resolved = Map::new();
        //
        // let resolve_stack = Vec::new();
        // while let Some((name, value)) = resolve_stack.pop().unwrap_or_else(|| source.slots.pop()) {
        //     match value {
        //         Value::Contains(color) => resolved.insert(key, ResolvingValue::Finished(color)),
        //         Value::LinkedTo(target) => if let Some(already_resolved) = resolved
        //     }
        // }

        Ok(Self {
            meta: source.meta,
            // idea is to run through all values once, and at each
            // - if it's a link and it's not been resolved yet, put it in an extra map with the
            //   target name as key and your name on a stack, also check whether others need to
            //   point on your target
            // - if it's a color, insert it into resolved and look if things in the queue want it
            // so links "accumulate" until they're resolved by a fitting color
            slots: source
                .slots
                .into_iter()
                .fold(State::default(), |mut state, (this_name, value)| {
                    match value {
                        Value::LinkedTo(target) => {
                            if let Some(color) = state.resolved.get(&target) {
                                insert_this_and_dependents(this_name, color.clone(), &mut state);
                            } else {
                                if let Err(involved) = detect_loop(&this_name, &target, &state) {
                                    state.errors.push(ResolveError::InfiniteLoop { involved });
                                } else {
                                    let mut pending_names =
                                        state.pending.remove(&this_name).unwrap_or_else(Vec::new);
                                    pending_names.push(this_name);
                                    state.pending.insert(target, pending_names);
                                }
                            }
                        }
                        Value::Contains(color) => {
                            insert_this_and_dependents(this_name, color, &mut state);
                        }
                    };

                    state
                })
                .resolved_or_errors()?,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ResolvedScheme {
    pub meta: Metadata,
    pub slots: Map<SlotName, Color>,
}

fn insert_this_and_dependents(this_name: SlotName, color: Color, state: &mut State) {
    for name in state
        .pending
        .remove(&this_name)
        .unwrap_or_else(Vec::new)
        .into_iter()
    {
        state.resolved.insert(name, color.clone());
    }
    state.resolved.insert(this_name, color);
}

/// Checks whether `start` is in a loop in `state`, errors with involved if that's the case.
fn detect_loop(start: &SlotName, target: &SlotName, state: &State) -> Result<(), Vec<SlotName>> {
    fn detect_loop_impl(
        target: SlotName,
        current: SlotName,
        mut path_depth: Vec<SlotName>,
        state: &State,
    ) -> Result<(), Vec<SlotName>> {
        path_depth.push(current.clone());

        for name in state
            .pending
            .get(&current)
            .cloned()
            .unwrap_or_else(Vec::new)
        {
            if target == name {
                // that's a loop
                path_depth.push(name);
                return Err(path_depth);
            } else {
                detect_loop_impl(target.clone(), name.clone(), path_depth.clone(), state)?;
            }
        }

        Ok(())
    }

    detect_loop_impl(target.clone(), start.clone(), Vec::new(), state)
}
