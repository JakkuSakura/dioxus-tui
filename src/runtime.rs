#![doc = include_str!("../README.md")]

mod config;
pub mod focus;
pub mod hooks;
pub mod layout;
pub mod prevent_default;
pub mod query;
mod render;
mod style;
mod style_attributes;
mod widget;
pub mod widgets;

use std::{
    pin::Pin,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    cursor::{MoveTo, RestorePosition, SavePosition, Show},
    event::{DisableMouseCapture, EnableMouseCapture, Event as TermEvent, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dioxus_native_core::{prelude::*, tree::TreeRef};
use dioxus_native_core::{node_ref::NodeMaskBuilder, real_dom::RealDom, FxDashSet, NodeId, SendAnyMap};
use focus::FocusState;
use futures::{channel::mpsc::UnboundedSender, pin_mut, Future, StreamExt};
use futures_channel::mpsc::unbounded;
use layout::TaffyLayout;
use prevent_default::PreventDefault;
use ratatui::{backend::CrosstermBackend, Terminal};
pub use taffy::{geometry::Point, prelude::*};
use tokio::select;
use widgets::{register_widgets, RinkWidgetResponder, RinkWidgetTraitObject};

pub use config::{Config, RenderingMode};
pub use hooks::*;
pub use query::Query;

pub fn screen_to_layout_space(screen: u16) -> f32 {
    screen as f32 * 10.0
}

pub fn unit_to_layout_space(screen: f32) -> f32 {
    screen * 10.0
}

pub fn layout_to_screen_space(layout: f32) -> f32 {
    layout / 10.0
}

#[derive(Clone)]
pub struct TuiContext {
    tx: UnboundedSender<InputEvent>,
}

impl TuiContext {
    pub fn new(tx: UnboundedSender<InputEvent>) -> Self {
        Self { tx }
    }

    pub fn quit(&self) {
        let _ = self.tx.unbounded_send(InputEvent::Close);
    }

    pub fn inject_event(&self, event: crossterm::event::Event) {
        let _ = self.tx.unbounded_send(InputEvent::UserInput(event));
    }
}

pub fn render<R: Driver>(
    cfg: Config,
    create_renderer: impl FnOnce(
        &Arc<RwLock<RealDom>>,
        &Arc<Mutex<Taffy>>,
        UnboundedSender<InputEvent>,
    ) -> R,
) -> Result<()> {
    let mut rdom = RealDom::new([
        TaffyLayout::to_type_erased(),
        focus::Focus::to_type_erased(),
        style_attributes::StyleModifier::to_type_erased(),
        PreventDefault::to_type_erased(),
    ]);

    let (event_tx, mut event_reciever) = unbounded();

    let (raw_event_tx, mut raw_event_reciever) = unbounded();
    let event_tx_clone = raw_event_tx.clone();
    if !cfg.headless {
        std::thread::spawn(move || {
            let tick_rate = Duration::from_millis(10);
            loop {
                if crossterm::event::poll(tick_rate).unwrap() {
                    let evt = crossterm::event::read().unwrap();
                    if raw_event_tx.unbounded_send(InputEvent::UserInput(evt)).is_err() {
                        break;
                    }
                }
            }
        });
    }

    register_widgets(&mut rdom, event_tx);

    let (mut handler, mut register_event) = hooks::RinkInputHandler::create(&mut rdom);

    let rdom = Arc::new(RwLock::new(rdom));

    let taffy = Arc::new(Mutex::new(Taffy::new()));

    let mut renderer = create_renderer(&rdom, &taffy, event_tx_clone);

    let query_engine = Query::new(rdom.clone(), taffy.clone());
    {
        let mut rdom = rdom.write().unwrap();
        rdom.raw_world_mut().add_unique(query_engine);
    }

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            {
                renderer.update(&rdom);
                let mut any_map = SendAnyMap::new();
                any_map.insert(taffy.clone());
                let mut rdom = rdom.write().unwrap();
                let _ = rdom.update_state(any_map);
            }

            let mut terminal = (!cfg.headless).then(|| {
                enable_raw_mode().unwrap();
                let mut stdout = std::io::stdout();
                execute!(stdout, EnterAlternateScreen, EnableMouseCapture, MoveTo(0, 1000)).unwrap();
                let backend = CrosstermBackend::new(std::io::stdout());
                Terminal::new(backend).unwrap()
            });
            if let Some(terminal) = &mut terminal {
                terminal.clear().unwrap();
            }

            let mut to_rerender = FxDashSet::default();
            to_rerender.insert(rdom.read().unwrap().root_id());
            let mut updated = true;

            loop {
                if !to_rerender.is_empty() || updated {
                    updated = false;
                    fn resize(dims: ratatui::layout::Rect, taffy: &mut Taffy, rdom: &RealDom) {
                        let width = screen_to_layout_space(dims.width);
                        let height = screen_to_layout_space(dims.height);
                        let root_node = rdom
                            .get(rdom.root_id())
                            .unwrap()
                            .get::<TaffyLayout>()
                            .unwrap()
                            .node
                            .unwrap();

                        let mut style = taffy.style(root_node).unwrap().clone();
                        let new_size = Size {
                            width: Dimension::Points(width),
                            height: Dimension::Points(height),
                        };
                        if style.size != new_size {
                            style.size = new_size;
                            taffy.set_style(root_node, style).unwrap();
                        }

                        let size = Size {
                            width: AvailableSpace::Definite(width),
                            height: AvailableSpace::Definite(height),
                        };
                        taffy.compute_layout(root_node, size).unwrap();
                    }
                    if let Some(terminal) = &mut terminal {
                        execute!(terminal.backend_mut(), SavePosition).unwrap();
                        terminal
                            .draw(|frame| {
                                let rdom = rdom.write().unwrap();
                                let mut taffy = taffy.lock().expect("taffy lock poisoned");
                                resize(frame.size(), &mut taffy, &rdom);
                                let root = rdom.get(rdom.root_id()).unwrap();
                                render::render_vnode(frame, &taffy, root, cfg, Point::ZERO);
                            })
                            .unwrap();
                        execute!(terminal.backend_mut(), RestorePosition, Show).unwrap();
                    } else {
                        let rdom = rdom.read().unwrap();
                        resize(
                            ratatui::layout::Rect { x: 0, y: 0, width: 1000, height: 1000 },
                            &mut taffy.lock().expect("taffy lock poisoned"),
                            &rdom,
                        );
                    }
                }

                let mut event_recieved = None;
                {
                    let wait = renderer.poll_async();

                    pin_mut!(wait);

                    select! {
                        _ = wait => {},
                        evt = raw_event_reciever.next() => {
                            match evt.as_ref().unwrap() {
                                InputEvent::UserInput(event) => match event {
                                    TermEvent::Key(key) => {
                                        if matches!(key.code, KeyCode::Char('C' | 'c'))
                                            && key.modifiers.contains(KeyModifiers::CONTROL)
                                            && cfg.ctrl_c_quit
                                        {
                                            break;
                                        }
                                    }
                                    TermEvent::Resize(_, _) => updated = true,
                                    _ => {}
                                },
                                InputEvent::Close => break,
                            };

                            if let InputEvent::UserInput(evt) = evt.unwrap() {
                                register_event(evt);
                            }
                        },
                        Some(evt) = event_reciever.next() => {
                            event_recieved = Some(evt);
                        }
                    }
                }

                {
                    if let Some(evt) = event_recieved {
                        renderer.handle_event(
                            &rdom,
                            evt.id,
                            evt.name,
                            Rc::new(evt.data),
                            evt.bubbles,
                        );
                    }
                    {
                        let evts = handler.get_events(
                            &taffy.lock().expect("taffy lock poisoned"),
                            &mut rdom.write().unwrap(),
                        );
                        updated |= handler.clean_focus();

                        for e in evts {
                            bubble_event_to_widgets(&mut rdom.write().unwrap(), &e);
                            renderer.handle_event(&rdom, e.id, e.name, Rc::new(e.data), e.bubbles);
                        }
                    }
                    renderer.update(&rdom);
                    let mut rdom = rdom.write().unwrap();
                    let mut any_map = SendAnyMap::new();
                    any_map.insert(taffy.clone());
                    let (new_to_rerender, dirty) = rdom.update_state(any_map);
                    to_rerender = new_to_rerender;
                    let text_mask = NodeMaskBuilder::new().with_text().build();
                    for (id, mask) in dirty {
                        if mask.overlaps(&text_mask) {
                            to_rerender.insert(id);
                        }
                    }
                }
            }

            if let Some(terminal) = &mut terminal {
                disable_raw_mode().unwrap();
                execute!(
                    terminal.backend_mut(),
                    LeaveAlternateScreen,
                    DisableMouseCapture
                )
                .unwrap();
                terminal.show_cursor().unwrap();
            }

            Ok(())
        })
}

#[derive(Debug)]
pub enum InputEvent {
    UserInput(TermEvent),
    Close,
}

pub trait Driver {
    fn update(&mut self, rdom: &Arc<RwLock<RealDom>>);
    fn handle_event(
        &mut self,
        rdom: &Arc<RwLock<RealDom>>,
        id: NodeId,
        event: &str,
        value: Rc<hooks::EventData>,
        bubbles: bool,
    );
    fn poll_async(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
}

fn bubble_event_to_widgets(rdom: &mut RealDom, event: &hooks::Event) {
    let id = event.id;
    let mut node = Some(id);

    while let Some(node_id) = node {
        let parent_id = {
            let tree = rdom.tree_ref();
            tree.parent_id_advanced(node_id, true)
        };

        {
            let mut node_mut = rdom.get_mut(node_id).unwrap();
            if let Some(mut widget) = node_mut
                .get_mut::<RinkWidgetTraitObject>()
                .map(|w| w.clone())
            {
                widget.handle_event(event, node_mut)
            }
        }

        if !event.bubbles {
            break;
        }
        node = parent_id;
    }
}

pub fn get_abs_layout(node: NodeRef, taffy: &Taffy) -> Layout {
    let mut node_layout = *taffy
        .layout(node.get::<TaffyLayout>().unwrap().node.unwrap())
        .unwrap();
    let mut current = node;

    let dom = node.real_dom();
    let tree = dom.tree_ref();

    while let Some(parent) = tree.parent_id_advanced(current.id(), true) {
        let parent = dom.get(parent).unwrap();
        current = parent;
        let parent_layout = taffy
            .layout(parent.get::<TaffyLayout>().unwrap().node.unwrap())
            .unwrap();
        node_layout.location.x += parent_layout.location.x;
        node_layout.location.y += parent_layout.location.y;
    }
    node_layout
}
