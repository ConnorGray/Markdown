use pulldown_cmark::{Event, Tag};

//======================================
// Representation
//======================================

#[derive(Debug)]
pub(crate) enum UnflattenedEvent<'a> {
    /// This [`Event`] can never by [`Event::Start`] or [`Event::End`]. Those events
    /// are represented by
    Event(Event<'a>),
    Nested {
        tag: Tag<'a>,
        events: Vec<UnflattenedEvent<'a>>,
    },
}

//======================================
// Implementation
//======================================

pub(crate) fn parse_markdown_to_unflattened_events<'i>(
    event_stream: impl Iterator<Item = Event<'i>>,
) -> Vec<UnflattenedEvent<'i>> {
    let mut unflattener = Unflattener {
        root: vec![],
        nested: vec![],
    };

    for event in event_stream {
        unflattener.handle_event(event);
    }

    unflattener.finish()
}

struct Unflattener<'a> {
    root: Vec<UnflattenedEvent<'a>>,
    nested: Vec<(Tag<'a>, Vec<UnflattenedEvent<'a>>)>,
}

impl<'a> Unflattener<'a> {
    fn handle_event(&mut self, event: Event<'a>) {
        match event {
            Event::Start(tag) => {
                self.nested.push((tag, vec![]));
            },
            Event::End(tag) => {
                let (tag2, inner) =
                    self.nested.pop().expect("expected nested events");

                debug_assert_eq!(tag, tag2.to_end());

                self.seq().push(UnflattenedEvent::Nested {
                    tag: tag2,
                    events: inner,
                });
            },
            event => self.seq().push(UnflattenedEvent::Event(event)),
        }
    }

    fn seq(&mut self) -> &mut Vec<UnflattenedEvent<'a>> {
        if let Some((_, seq)) = self.nested.last_mut() {
            seq
        } else {
            &mut self.root
        }
    }

    fn finish(self) -> Vec<UnflattenedEvent<'a>> {
        let Unflattener { root, nested } = self;

        assert!(nested.is_empty());

        root
    }
}
