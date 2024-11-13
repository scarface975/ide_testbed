use std::rc::Rc;

use dominator::{clone, events, Dom, EventOptions};
use dominator_bulma::{column, columns};
use futures_signals::{map_ref, signal::{self, Mutable, Signal, SignalExt}};

pub mod console;
pub mod activity_panel;

const DEFAULT_CONSOLE_HEIGHT: u32 = 200;
const RESIZER_HEIGHT: u32 = 4;

pub struct Workspace {
    activity_panel: Rc<activity_panel::ActivityPanel>,
    console: Rc<console::Console>,
    console_height: Mutable<u32>,
    resize_active: Mutable<bool>,
    resizer_hover: Mutable<bool>
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            activity_panel: Default::default(),
            console: Default::default(),
            console_height: Mutable::new(DEFAULT_CONSOLE_HEIGHT),
            resize_active: Mutable::new(false),
            resizer_hover: Mutable::new(false),
        }
    }
}
   
// part of the problem is that I need to respond to the user moving the mouse, but also the size of the window
impl Workspace {
    pub fn render(
        this: &Rc<Workspace>,
        workspace_command_rx: crate::WorkspaceCommandReceiver,
        width: impl Signal<Item = u32> + 'static,
        height: impl Signal<Item = u32> + 'static
    ) -> Dom {
        use activity_panel::ActivityPanel;

        let console_height = this.console_height.signal();
        let activity_panel_height = 
            map_ref!(height, console_height => height.saturating_sub(console_height + RESIZER_HEIGHT));

        columns!("is-gapless", "is-mobile", "is-multiline", {
            // activity area
            .child(column!("is-full", {
                // .style_signal("height", activity_panel_height
                //     .map(|height| format!("{height}px")))
                .child(ActivityPanel::render(&this.activity_panel, workspace_command_rx, width, activity_panel_height))
            }))

            // resizer
            .child(column!("is-full", {
                .style("cursor", "ns-resize")
                .style("height", &format!("{RESIZER_HEIGHT}px"))
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
                    if this.console_height.get() == 0 {
                        // close console and reset default size, this could be a boolean
                        // e.g., console visible OR we could use something more similar
                        // to the sidebar/menu logic
                        //this.active_panel.set(None);
                        this.console_height.set(DEFAULT_CONSOLE_HEIGHT)
                    }
                }))
                .event(clone!(this => move |_: events::PointerEnter| {
                    this.resizer_hover.set_neq(true);
                }))
                .event(clone!(this => move |_: events::PointerLeave| {
                    this.resizer_hover.set_neq(false);
                }))
                .global_event(clone!(this => move |event: events::PointerMove| {
                    if this.resize_active.get() {
                        let available_height = web_sys::window()
                            .unwrap()
                            .inner_height()
                            .unwrap()
                            .as_f64()
                            .map(|window_size| window_size.max(0.0))
                            .unwrap() as u32;
                        let console_height = available_height
                            .saturating_sub(event.y().max(0) as u32 + RESIZER_HEIGHT);
                        match console_height {
                            0..=75 => {
                                this.console_height.set(0);
                            }
                            76..=150 => {}
                            _ => {
                                this.console_height.set(console_height);
                            }
                        }
                    }
                }))
            }))
            
            // terminal
            .child(column!("is-full", {
                .style_signal("height", this.console_height.signal()
                    .map(|height| format!("{height}px")))
                .child(this.console.render())
            }))
        })
    }
}
