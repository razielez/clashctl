use std::{collections::HashMap, time::Instant};

use smart_default::SmartDefault;

use crate::{
    interactive::ProxySort,
    model::{Traffic, Version},
    ui::{
        components::{MovableListItem, MovableListState, ProxyTree},
        Action, Event, InputEvent, UpdateEvent,
    },
    Result,
};

#[derive(Debug, Clone, SmartDefault)]
pub struct TuiStates<'a> {
    pub should_quit: bool,
    #[default(_code = "Instant::now()")]
    pub start_time: Instant,
    pub version: Option<Version>,
    pub traffics: Vec<Traffic>,
    pub max_traffic: Traffic,
    pub events: Vec<Event>,
    pub all_events_recv: usize,
    pub page_index: u8,
    pub show_debug: bool,
    pub proxy_tree: ProxyTree<'a>,
    pub proxy_sort: ProxySort,
    pub debug_state: MovableListState<'a>,
    pub log_state: MovableListState<'a>,
    pub con_state: MovableListState<'a>,
    pub rule_state: MovableListState<'a>,
    pub rule_freq: HashMap<String, usize>,
    // (upload_size, download_size)
    pub con_size: (u64, u64),
}

// TODO fix: drop_events not working
impl<'a> TuiStates<'a> {
    pub const TITLES: &'static [&'static str] = &[
        "Status", "Proxies", "Rules", "Conns", "Logs", "Configs", "Debug",
    ];

    pub fn handle(&mut self, event: Event) -> Result<Option<Action>> {
        self.all_events_recv += 1;
        if self.events.len() >= 300 {
            drop(self.drop_events(100))
        }
        self.events.push(event.to_owned());
        self.debug_state
            .push(MovableListItem::Spans((&event).into()));

        match event {
            Event::Quit => {
                self.should_quit = true;
                Ok(None)
            }
            Event::Input(event) => self.handle_input(event),
            Event::Update(update) => self.handle_update(update),
            _ => Ok(None),
        }
    }

    #[inline]
    pub fn page_len(&mut self) -> usize {
        if self.show_debug {
            Self::TITLES.len()
        } else {
            Self::TITLES.len() - 1
        }
    }

    #[inline]
    pub fn title(&self) -> &str {
        Self::TITLES[self.page_index as usize]
    }

    pub fn active_list_state(&mut self) -> Option<&mut MovableListState<'a>> {
        match self.title() {
            "Logs" => Some(&mut self.log_state),
            "Debug" => Some(&mut self.debug_state),
            "Rules" => Some(&mut self.rule_state),
            "Conns" => Some(&mut self.con_state),
            _ => None,
        }
    }

    fn handle_update(&mut self, update: UpdateEvent) -> Result<Option<Action>> {
        match update {
            UpdateEvent::Connection(connection) => {
                self.con_size = (connection.upload_total, connection.download_total);
                self.con_state.merge(connection.into());
            }
            UpdateEvent::Version(version) => self.version = Some(version),
            UpdateEvent::Traffic(traffic) => {
                let Traffic { up, down } = traffic;
                self.max_traffic.up = self.max_traffic.up.max(up);
                self.max_traffic.down = self.max_traffic.down.max(down);
                self.traffics.push(traffic)
            }
            UpdateEvent::Proxies(proxies) => {
                let mut new_tree = Into::<ProxyTree>::into(proxies);
                new_tree.sort_with_frequency(&self.rule_freq);
                self.proxy_tree.replace_with(new_tree);
            }
            UpdateEvent::Log(log) => {
                self.log_state.push(MovableListItem::Spans(log.into()));
            }
            UpdateEvent::Rules(rules) => {
                self.rule_freq = rules.owned_frequency();
                self.rule_state.merge(rules.into());
            }
            UpdateEvent::ProxyTestLatencyDone => {
                self.proxy_tree.end_testing();
            }
        }
        Ok(None)
    }

    fn handle_input(&mut self, event: InputEvent) -> Result<Option<Action>> {
        match event {
            InputEvent::TabGoto(index) => {
                if index >= 1 && index <= self.page_len() as u8 {
                    self.page_index = index - 1
                }
            }
            InputEvent::ToggleDebug => {
                self.show_debug = !self.show_debug;
                // On the debug page
                if self.page_index == Self::TITLES.len() as u8 - 1 {
                    self.page_index -= 1;
                } else if self.show_debug {
                    self.page_index = self.debug_page_index()
                }
            }
            InputEvent::ToggleHold => match self.active_list_state() {
                Some(state) => state.toggle(),
                None => {
                    if self.title() == "Proxies" {
                        self.proxy_tree.toggle();
                    }
                }
            },
            InputEvent::List(event) => match self.active_list_state() {
                Some(state) => state.handle(event),
                None => {
                    if self.title() == "Proxies" {
                        return Ok(self.proxy_tree.handle(event));
                    }
                }
            },
            InputEvent::TestLatency => {
                if self.title() == "Proxies" && !self.proxy_tree.is_testing() {
                    self.proxy_tree.start_testing();
                    let group = self.proxy_tree.current_group();
                    let proxies = group
                        .members()
                        .iter()
                        .filter(|x| x.proxy_type().is_normal())
                        .map(|x| x.name().into())
                        .collect();
                    return Ok(Some(Action::TestLatency { proxies }));
                }
            }
            InputEvent::Esc => match self.active_list_state() {
                Some(state) => state.end(),
                None => {
                    if self.title() == "Proxies" {
                        self.proxy_tree.end();
                    }
                }
            },
            InputEvent::Sort => {}
            InputEvent::Other(_) => {} // InterfaceEvent::Other(event) => self.handle_list(event),
        }
        Ok(None)
    }

    pub const fn debug_page_index(&self) -> u8 {
        Self::TITLES.len() as u8 - 1
    }

    fn drop_events(&mut self, num: usize) -> impl Iterator<Item = Event> + '_ {
        self.events.drain(..num)
    }
}