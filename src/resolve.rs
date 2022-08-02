use {
    super::{Color, LinkedScheme, Map, Metadata, MultipleSlotNames, Set, SlotName, Value},
    indexmap::map::Entry,
    serde::Serialize,
    std::iter,
    thiserror::Error,
};

#[derive(Clone, Debug, Error)]
pub enum Error {
    #[error("Noticed a neverending loop of slot links, involved are {involved}")]
    InfiniteLoop { involved: MultipleSlotNames },
    #[error("{from} link to {to}, which doesn't exist")]
    LinkToNonexistent {
        from: MultipleSlotNames,
        to: SlotName,
    },
    #[error("Impossible to resolve {subject}, which links to itself")]
    LinkToItself { subject: SlotName },
}

enum PossiblyResolvedColor {
    BlockedOn(SlotName),
    Ready(Color),
}

#[derive(Default)]
struct State {
    /// slots which are already done
    resolved: Map<SlotName, PossiblyResolvedColor>,
    /// left slots which are depended on, right slots which depend on the left
    pending: Map<SlotName, Vec<SlotName>>,
    /// any loops/other errors which were detected while processing
    errors: Vec<Error>,
}

impl State {
    /// Returns all resolved values, erroring if there's anything pending.
    fn resolved_or_errors(self) -> Result<Map<SlotName, Color>, Vec<Error>> {
        if self.pending.is_empty() {
            Ok(self
                .resolved
                .into_iter()
                .map(|(name, value)| match value {
                    PossiblyResolvedColor::Ready(color) => (name, color),
                    _ => unreachable!(
                        "there can be no value blocked on another if there's nothing pending"
                    ),
                })
                .collect())
        } else {
            // ok, then let's analyze what happened
            // they've been detected as loops already and might issue false positives else
            let names_to_ignore: Set<_> = self
                .errors
                .clone()
                .into_iter()
                .flat_map(|err| match err {
                    Error::InfiniteLoop { involved } => involved.0,
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
                            Error::LinkToItself { subject: target }
                        } else {
                            Error::LinkToNonexistent {
                                from: dependents.into(),
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
    type Error = Vec<Error>;
    fn try_from(source: LinkedScheme) -> Result<Self, Vec<Error>> {
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
                        Value::Contains(color) => {
                            // noice, just resolved
                            insert_this_and_dependents(this_name, color, &mut state);
                        }
                        Value::LinkedTo(target) => {
                            // so this links to some other name...
                            match state.resolved.get(&target) {
                                Some(PossiblyResolvedColor::Ready(color)) => {
                                    // ...but that name has been resolved already
                                    insert_this_and_dependents(this_name, color.clone(), &mut state)
                                }
                                Some(PossiblyResolvedColor::BlockedOn(block_name)) => {
                                    // ...but that name is also just blocked, let's block on the
                                    // actual one and "migrate" all existing values (after checking
                                    // for loops)
                                    migrate_this_to_blocker(
                                        this_name.clone(),
                                        block_name.clone(),
                                        &mut state,
                                    );

                                    if let Err(involved) = detect_loop(&this_name, &target, &state)
                                    {
                                        state.errors.push(Error::InfiniteLoop {
                                            involved: involved.into(),
                                        })
                                    }
                                }
                                None => {
                                    // ...but that target name isn't even processed yet, in that
                                    // case the target name is the blocker
                                    migrate_this_to_blocker(this_name, target.clone(), &mut state);
                                }
                            }
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
        .chain(iter::once(this_name))
    {
        state
            .resolved
            .insert(name, PossiblyResolvedColor::Ready(color.clone()));
    }
}

fn migrate_this_to_blocker(this_name: SlotName, block_name: SlotName, state: &mut State) {
    let mut pending_on_this = state
        .pending
        .remove(&this_name)
        .unwrap_or_else(|| Vec::with_capacity(1));

    // make sure that this slot will also depend on the new blocker
    pending_on_this.push(this_name);

    for pending_name in pending_on_this.clone() {
        // fortunately pending things _cannot_ be "suddently" resolved for other reasons
        // that makes it impossible to change from resolved => blocked on, so it's fine to just
        // overwrite I guess
        state.resolved.insert(
            pending_name,
            PossiblyResolvedColor::BlockedOn(block_name.clone()),
        );
    }

    match state.pending.entry(block_name.clone()) {
        Entry::Occupied(mut entry) => {
            entry.get_mut().extend(pending_on_this);
        }
        Entry::Vacant(entry) => {
            entry.insert(pending_on_this);
        }
    }
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
