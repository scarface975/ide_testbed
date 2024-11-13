use std::rc::Rc;

use dominator::{clone, events, Dom, EventOptions};
use dominator_bulma::{block, column, columns, image};
use futures_signals::{map_ref, signal::{self, Mutable, Signal, SignalExt}};

pub mod explorer;
pub mod search;

const DEFAULT_PANEL_SIZE: u32 = 200;
const MENU_SIZE_PX: u32 = 50;
const RESIZER_PX: u32 = 4;

enum Panel {
    // Not sure if this Rc is necessary?
    Explorer(Rc<explorer::Explorer>),
    Search(search::Search)
}

impl Panel {
    fn tooltip(&self) -> &'static str {
        match self {
            Panel::Explorer(explorer) => explorer.tooltip(),
            Panel::Search(search) => search.tooltip(),
        }
    }
    
    fn icon(&self, active: impl Signal<Item = bool> + 'static) -> dominator::Dom {
        match self {
            Panel::Explorer(explorer) => explorer.icon(active),
            Panel::Search(search) => search.icon(active),
        }
    }

    fn render(&self, workspace_command_tx: &crate::WorkspaceCommandSender) -> dominator::Dom {
        match self {
            Panel::Explorer(explorer) => explorer::Explorer::render(explorer, workspace_command_tx),
            Panel::Search(search) => search.render(),
        }
    }
}

pub struct Sidebar {
    panels: Vec<Rc<Panel>>,
    active_panel: Mutable<Option<Rc<Panel>>>,
    panel_size: Mutable<u32>,
    resize_active: Mutable<bool>,
    resizer_hover: Mutable<bool>
}

impl Default for Sidebar {
    fn default() -> Self {
        // hack
        let explorer = Rc::new(Panel::Explorer(explorer::Explorer::default().into()));

        Self {
            panels: vec![
                explorer.clone(),
                Rc::new(Panel::Search(search::Search::default()))
            ],
            // hack
            active_panel: Mutable::new(Some(explorer)),
            panel_size: Mutable::new(DEFAULT_PANEL_SIZE),
            resize_active: Mutable::new(false),
            resizer_hover: Mutable::new(false),
        }
    }
}
   

impl Sidebar {
    pub fn width(this: &Rc<Sidebar>) -> impl Signal<Item = u32> + 'static {
        map_ref! {
            let panel_size = this.panel_size.signal(),
            let panel_active = this.active_panel.signal_ref(Option::is_some) =>
            MENU_SIZE_PX + match *panel_active {
                true => RESIZER_PX + *panel_size,
                false => 0
            }
        }
    }

    pub fn render(this: &Rc<Sidebar>, workspace_command_tx: &crate::WorkspaceCommandSender) -> Dom {
        columns!("is-gapless", "is-mobile", {
            // menu
            .child(column!("is-narrow", {
                .style("width", &format!("{MENU_SIZE_PX}px"))
                .child(Self::render_menu(this))
            }))
            
            // panel
            .child_signal(this.active_panel.signal_cloned().map(clone!(this, workspace_command_tx => move |panel| {
                panel.map(clone!(this, workspace_command_tx => move |panel| column!("is-narrow", {
                    .style_signal("width", this.panel_size.signal_ref(|size| {
                        format!("{size}px")
                    }))
                    .class_signal("is-hidden", this.panel_size.signal().eq(0))
                    .child(panel.render(&workspace_command_tx))
                })))
            })))
            
            // resizer
            .child_signal(this.active_panel.signal_cloned().map(clone!(this => move |panel| {
                panel.map(clone!(this => move |_| column!("is-narrow", {
                    .style("cursor", "ew-resize")
                    .style("min-height", "100vh")
                    .style("width", &format!("{RESIZER_PX}px"))
                    .class_signal("has-background-white-ter",
                        signal::not(signal::or(this.resize_active.signal(), this.resizer_hover.signal())))
                    .class_signal("has-background-info",
                        signal::or(this.resize_active.signal(), this.resizer_hover.signal()))
                    .event_with_options(&EventOptions::preventable(),
                        clone!(this => move |ev: events::PointerDown| {
                        this.resize_active.set_neq(true);
                        ev.prevent_default();
                    }))
                    .global_event(clone!(this => move |_: events::PointerUp| {
                        this.resize_active.set_neq(false);
                        if this.panel_size.get() == 0 {
                            this.active_panel.set(None);
                            this.panel_size.set(DEFAULT_PANEL_SIZE)
                        }
                    }))
                    .event(clone!(this => move |_: events::PointerOver| {
                        this.resizer_hover.set_neq(true);
                    }))
                    .event(clone!(this => move |_: events::PointerOut| {
                        this.resizer_hover.set_neq(false);
                    }))
                    .global_event(clone!(this => move |event: events::PointerMove| {
                        if this.resize_active.get() {
                            let max_panel_size_px = web_sys::window()
                                .unwrap()
                                .inner_width()
                                .unwrap()
                                .as_f64()
                                .map(|window_size| 0.8 * window_size)
                                .unwrap() as u32;
                            let panel_size_px = (event.x().max(0) as u32).saturating_sub(MENU_SIZE_PX)
                                .min(max_panel_size_px);
                            match panel_size_px {
                                0..=150 => {
                                    this.panel_size.set(0);
                                }
                                151..=200 => {}
                                _ => {
                                    this.panel_size.set(panel_size_px);
                                }
                            }
                        }
                    }))
                })))
            })))
        })
    }

    fn render_menu(this: &Rc<Sidebar>) -> Dom {
        let buttons = this.panels.iter()
            .map(clone!(this => move |panel| {
                let mouse_over = Mutable::new(false);
                let active = this.active_panel
                    .signal_ref(clone!(panel => move |active_panel| active_panel
                        .as_ref()
                        .map(|active_panel| Rc::ptr_eq(active_panel, &panel))
                        .unwrap_or(false)));
                let active = signal::or(active, mouse_over.signal());

                image!("px-2", "pt-4", {
                    .attr("title", panel.tooltip())
                    .child(panel.icon(active))
                    .style("cursor", "pointer")
                    .event(clone!(mouse_over => move |_: events::PointerOver| {
                        mouse_over.set_neq(true);
                    }))
                    .event(clone!(mouse_over => move |_: events::PointerOut| {
                        mouse_over.set_neq(false);
                    }))
                    .event_with_options(
                        &EventOptions::preventable(),
                        clone!(this, panel => move |ev: events::PointerDown| {
                            ev.prevent_default();
                            let mut active_panel = this.active_panel.lock_mut();
                            *active_panel = match &*active_panel {
                                Some(active_panel) if Rc::ptr_eq(active_panel, &panel) => None,
                                _ => Some(panel.clone())
                            }
                        })
                    )
                })
            }));

        block!({
            .class("has-background-grey-darker")
            .style("min-height", "100vh")
            .children(buttons)
        })
    }
}
