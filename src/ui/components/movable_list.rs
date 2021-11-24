use std::ops::Range;

use tui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{List, ListItem},
};
use tui::{text::Spans, widgets::Widget};
use unicode_width::UnicodeWidthStr;

use crate::{
    components::{string_window, Footer, FooterItem, FooterWidget},
    ui::{
        components::{get_block, get_focused_block, get_text_style, spans_window},
        Coord,
    },
};

/// TODO Change [`GenericWidget`] into `MovableListItem` or same thing
///
/// E.g.
///   
/// ```rust
/// pub enum MovableListItem<'a> {
///     Spans(Spans<'a>),
///     Raw(Cow<'a, str>)
/// }
///
/// impl MovableListItem {
///     pub fn new() { todo!() }
///
///     pub fn width(&self) {
///         match self {
///             MovableListItem::Spans(x) => x.width(),
///             MovableListItem::Raw(x) => x.width()
///         }
///     }
///
///     pub fn scope(&self, range: Range) -> Self {
///         match self {
///             MovableListItem::Spans(x) => Self::Spans(spans_window(x, range)),
///             MovableListItem::Raw(x) => Self::raw(x[range].into())
///         }
///     }
///
///     pub fn render() { todo!() } // Maybe render here
/// }
/// ```
#[derive(Clone, Debug)]
pub struct MovableList<'a> {
    title: String,
    state: &'a MovableListState<'a>,
}

impl<'a> MovableList<'a> {
    pub fn new<TITLE: Into<String>>(title: TITLE, state: &'a MovableListState<'a>) -> Self {
        Self {
            state,
            title: title.into(),
        }
    }

    fn render_footer(&self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer, pos: Coord) {
        let mut footer = Footer::default();

        footer.push_right(FooterItem::span(Span::styled(
            format!(" Ln {}, Col {} ", pos.y, pos.x),
            Style::default()
                .fg(if pos.hold { Color::Green } else { Color::Blue })
                .add_modifier(Modifier::REVERSED),
        )));

        if pos.hold {
            footer.push_left(FooterItem::span(Span::styled(
                " HOLD ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::REVERSED),
            )));
            footer.push_left(FooterItem::span(Span::styled(
                " [^] ▲ ▼ ◀ ▶ Move ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::REVERSED),
            )));
        } else {
            footer.push_left(FooterItem::span(Span::styled(
                " FREE ",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::REVERSED),
            )));
            footer.push_left(FooterItem::span(Span::styled(
                " SPACE to hold ",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::REVERSED),
            )));
        }

        let widget = FooterWidget::new(&footer);
        widget.render(area, buf);
    }
}

#[derive(Debug, Default, Clone)]
pub struct MovableListState<'a> {
    pub offset: Coord,
    pub items: Vec<MovableListItem<'a>>,
}

impl<'a> MovableListState<'a> {
    pub fn current_pos(&self) -> Coord {
        let x = self.offset.x;
        let y = self.len().saturating_sub(self.offset.y);
        Coord {
            x,
            y,
            hold: self.offset.hold,
        }
    }
}

impl<'a> MovableListState<'a> {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn _is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[derive(Debug, Clone)]
pub enum MovableListItem<'a> {
    Spans(Spans<'a>),
    Raw(String),
}

impl<'a> MovableListItem<'a> {
    pub fn width(&self) -> usize {
        match self {
            Self::Spans(x) => x.width(),
            Self::Raw(x) => x.width(),
        }
    }

    pub fn range(&mut self, range: &Range<usize>) -> &mut Self {
        match self {
            MovableListItem::Spans(ref mut x) => *x = spans_window(x, range),
            MovableListItem::Raw(ref mut x) => *x = string_window(x, range),
        };
        self
    }
}

impl<'a, T: Into<String>> From<T> for MovableListItem<'a> {
    fn from(string: T) -> Self {
        Self::Raw(string.into())
    }
}

impl<'a> From<MovableListItem<'a>> for Spans<'a> {
    fn from(val: MovableListItem<'a>) -> Self {
        match val {
            MovableListItem::Spans(spans) => spans,
            MovableListItem::Raw(raw) => raw.into(),
        }
    }
}

impl<'a> Widget for MovableList<'a> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let height = (area.height as usize).saturating_sub(2);
        let num = self.state.items.len();
        let offset = self.state.offset;

        // Calculate which portion of the list will be displayed
        let y_offset = if offset.y + 1 > num {
            num.saturating_sub(1)
        } else {
            offset.y
        };

        // Get that portion of items
        let items = self
            .state
            .items
            .iter()
            .rev()
            .skip(y_offset)
            .take(height as usize);

        let x_offset = offset.x;

        let x_range = x_offset..(x_offset + area.width as usize);

        let items = items.cloned().map(move |mut x| {
            let x_width = x.width();
            let content = x.range(&x_range);
            if x_width != 0 && content.width() == 0 {
                *content = MovableListItem::Raw("◀".to_owned());
            }
            ListItem::new(Spans::from(content.to_owned()))
        });

        List::new(items.collect::<Vec<_>>())
            .block(if offset.hold {
                get_focused_block(&self.title)
            } else {
                get_block(&self.title)
            })
            .style(get_text_style())
            .render(area, buf);

        self.render_footer(area, buf, self.state.current_pos());
    }
}

// #[test]
// fn test_movable_list() {
//     let items = &["Test1", "测试1", "[ABCD] 🇺🇲 测试 符号 106"].into_iter().map(|x| x.);
//     assert_eq!()
// }
