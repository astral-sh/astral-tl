use std::marker::PhantomData;

use crate::{InnerNodeHandle, NodeHandle, Parser};

use super::{iterable::QueryIterable, Selector};

/// A query selector iterator that yields matching HTML nodes
pub struct QuerySelectorIterator<'a, 'b, Q: QueryIterable<'a>> {
    selector: Selector<'b>,
    collection: &'b Q,
    parser: &'b Parser<'a>,
    index: usize,
    len: usize,
    needs_ancestry: bool,
    ancestor_stack: Vec<(InnerNodeHandle, NodeHandle)>,
    _a: PhantomData<&'a ()>,
}

impl<'a, 'b, Q: QueryIterable<'a>> Clone for QuerySelectorIterator<'a, 'b, Q> {
    fn clone(&self) -> Self {
        Self {
            selector: self.selector.clone(),
            collection: self.collection,
            parser: self.parser,
            index: self.index,
            len: self.len,
            needs_ancestry: self.needs_ancestry,
            ancestor_stack: self.ancestor_stack.clone(),
            _a: PhantomData,
        }
    }
}

impl<'a, 'b, Q: QueryIterable<'a>> QuerySelectorIterator<'a, 'b, Q> {
    /// Creates a new query selector iterator
    pub fn new(selector: Selector<'b>, parser: &'b Parser<'a>, collection: &'b Q) -> Self {
        let needs_ancestry = selector.needs_ancestry();
        let ancestor_stack = if needs_ancestry {
            Vec::with_capacity(8)
        } else {
            Vec::new()
        };
        Self {
            selector,
            collection,
            index: 0,
            len: collection.len(parser),
            parser,
            needs_ancestry,
            ancestor_stack,
            _a: PhantomData,
        }
    }
}

impl<'a, 'b, Q: QueryIterable<'a>> Iterator for QuerySelectorIterator<'a, 'b, Q> {
    type Item = NodeHandle;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.len {
            let Some((node, id)) = self.collection.get(self.parser, self.index) else {
                self.index += 1;
                continue;
            };
            let current_idx = id.get_inner();
            self.index += 1;

            if self.needs_ancestry {
                // Pop ancestors whose subtrees ended before the current node
                while self
                    .ancestor_stack
                    .last()
                    .is_some_and(|&(end, _)| end < current_idx)
                {
                    self.ancestor_stack.pop();
                }

                let m = self
                    .selector
                    .matches_with_ancestors(node, &self.ancestor_stack, self.parser);

                // Push this tag onto the ancestor stack for its descendants
                if let Some(tag) = node.as_tag() {
                    if let Some((_, end)) = tag.children().boundaries(self.parser) {
                        self.ancestor_stack.push((end, id));
                    }
                }

                if m {
                    return Some(id);
                }
            } else if self.selector.matches(node) {
                return Some(id);
            }
        }

        None
    }
}
