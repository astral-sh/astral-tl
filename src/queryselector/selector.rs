use crate::{InnerNodeHandle, Node, NodeHandle, Parser};

/// A single query selector node
#[derive(Debug, Clone)]
pub enum Selector<'a> {
    /// Tag selector: foo
    Tag(&'a [u8]),
    /// ID selector: #foo
    Id(&'a [u8]),
    /// Class selector: .foo
    Class(&'a [u8]),
    /// All selector: *
    All,
    /// And combinator: .foo.bar
    And(Box<Selector<'a>>, Box<Selector<'a>>),
    /// Or combinator: .foo, .bar
    Or(Box<Selector<'a>>, Box<Selector<'a>>),
    /// Descendant combinator: .foo .bar
    Descendant(Box<Selector<'a>>, Box<Selector<'a>>),
    /// Parent combinator: .foo > .bar
    Parent(Box<Selector<'a>>, Box<Selector<'a>>),
    /// Attribute: \[foo\]
    Attribute(&'a [u8]),
    /// Attribute with value: [foo=bar]
    AttributeValue(&'a [u8], &'a [u8]),
    /// Attribute with whitespace-separated list of values that contains a value: [foo~=bar]
    AttributeValueWhitespacedContains(&'a [u8], &'a [u8]),
    /// Attribute with value that starts with: [foo^=bar]
    AttributeValueStartsWith(&'a [u8], &'a [u8]),
    /// Attribute with value that ends with: [foo$=bar]
    AttributeValueEndsWith(&'a [u8], &'a [u8]),
    /// Attribute with value that contains: [foo*=bar]
    AttributeValueSubstring(&'a [u8], &'a [u8]),
}

impl<'a> Selector<'a> {
    /// Checks if the given node matches this selector (no ancestry context required)
    pub fn matches<'b>(&self, node: &Node<'b>) -> bool {
        match self {
            Self::Tag(tag) => node.as_tag().is_some_and(|t| t._name.as_bytes().eq(*tag)),
            Self::Id(id) => node
                .as_tag()
                .is_some_and(|t| t._attributes.id == Some((*id).into())),
            Self::Class(class) => node
                .as_tag()
                .is_some_and(|t| t._attributes.is_class_member(*class)),
            Self::And(a, b) => a.matches(node) && b.matches(node),
            Self::Or(a, b) => a.matches(node) || b.matches(node),
            Self::All => true,
            Self::Attribute(attribute) => node
                .as_tag()
                .is_some_and(|t| t._attributes.get(*attribute).is_some()),
            Self::AttributeValue(attribute, value) => {
                check_attribute(node, attribute, value, |attr, value| attr == value)
            }
            Self::AttributeValueEndsWith(attribute, value) => {
                check_attribute(node, attribute, value, |attr, value| attr.ends_with(value))
            }
            Self::AttributeValueStartsWith(attribute, value) => {
                check_attribute(node, attribute, value, |attr, value| {
                    attr.starts_with(value)
                })
            }
            Self::AttributeValueSubstring(attribute, value) => {
                check_attribute(node, attribute, value, |attr, value| attr.contains(value))
            }
            Self::AttributeValueWhitespacedContains(attribute, value) => {
                check_attribute(node, attribute, value, |attr, value| {
                    attr.split_whitespace().any(|x| x == value)
                })
            }
            Self::Descendant(..) | Self::Parent(..) => {
                unreachable!("combinator selectors require matches_with_ancestors")
            }
        }
    }

    /// Returns true if this selector contains a `>` or descendant-space combinator
    pub fn needs_ancestry(&self) -> bool {
        match self {
            Self::Descendant(..) | Self::Parent(..) => true,
            Self::And(a, b) | Self::Or(a, b) => a.needs_ancestry() || b.needs_ancestry(),
            _ => false,
        }
    }

    /// Checks if the given node matches this selector given the current ancestor stack.
    ///
    /// `ancestors` is a slice of `(subtree_end_inclusive, handle)` pairs ordered
    /// from outermost to innermost ancestor, maintained by the iterator.
    pub fn matches_with_ancestors<'b>(
        &self,
        node: &Node<'b>,
        ancestors: &[(InnerNodeHandle, NodeHandle)],
        parser: &Parser<'b>,
    ) -> bool {
        match self {
            Self::Tag(..)
            | Self::Id(..)
            | Self::Class(..)
            | Self::All
            | Self::Attribute(..)
            | Self::AttributeValue(..)
            | Self::AttributeValueSubstring(..)
            | Self::AttributeValueStartsWith(..)
            | Self::AttributeValueEndsWith(..)
            | Self::AttributeValueWhitespacedContains(..) => self.matches(node),

            Self::And(a, b) => {
                a.matches_with_ancestors(node, ancestors, parser)
                    && b.matches_with_ancestors(node, ancestors, parser)
            }
            Self::Or(a, b) => {
                a.matches_with_ancestors(node, ancestors, parser)
                    || b.matches_with_ancestors(node, ancestors, parser)
            }
            Self::Parent(ancestor_sel, node_sel) => {
                if !node_sel.matches_with_ancestors(node, ancestors, parser) {
                    return false;
                }
                match ancestors.last() {
                    None => false,
                    Some(&(_, parent_handle)) => {
                        let Some(parent_node) = parent_handle.get(parser) else {
                            return false;
                        };
                        let parent_ancestors = &ancestors[..ancestors.len() - 1];
                        ancestor_sel.matches_with_ancestors(parent_node, parent_ancestors, parser)
                    }
                }
            }
            Self::Descendant(ancestor_sel, node_sel) => {
                if !node_sel.matches_with_ancestors(node, ancestors, parser) {
                    return false;
                }
                for i in (0..ancestors.len()).rev() {
                    let (_, anc_handle) = ancestors[i];
                    if let Some(anc_node) = anc_handle.get(parser) {
                        if ancestor_sel.matches_with_ancestors(anc_node, &ancestors[..i], parser) {
                            return true;
                        }
                    }
                }
                false
            }
        }
    }
}

fn check_attribute<F>(node: &Node, attribute: &[u8], value: &[u8], callback: F) -> bool
where
    F: Fn(&str, &str) -> bool,
{
    node.as_tag().is_some_and(|t| {
        t._attributes
            .get(attribute)
            .flatten()
            .is_some_and(|attr| callback(&attr.as_utf8_str(), &String::from_utf8_lossy(value)))
    })
}
