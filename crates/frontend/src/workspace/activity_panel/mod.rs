use std::{pin::Pin, rc::Rc};

use dominator::{clone, events, html, svg, Dom, EventOptions};
use dominator_bulma::{block, column, columns, icon, icon_text};
use futures::StreamExt;
use futures_signals::{signal::{self, Mutable, Signal, SignalExt}, signal_vec::{MutableVec, SignalVecExt}};
use crate::contextmenu::ContextMenuState;

pub mod editor;
pub mod welcome;

const TAB_HEIGHT: u32 = 48;

#[derive(Clone)]
enum Activity {
    Editor(Rc<editor::Editor>),
    Welcome(Rc<welcome::Welcome>),
}

impl Activity {
    pub fn render(
        this: &Rc<Activity>,
        width: impl Signal<Item = u32> + 'static,
        height: impl Signal<Item = u32> + 'static
    ) -> Pin<Box<dyn Signal<Item = Option<dominator::Dom>>>> {
        match this.as_ref() {
            Activity::Editor(editor) => Box::pin(editor::Editor::render(editor, width, height)),
            Activity::Welcome(welcome) => Box::pin(welcome::Welcome::render(welcome, width, height)),
        }
    }

    pub fn label(&self) -> Dom {
        match self {
            Activity::Editor(editor) => editor.label(),
            Activity::Welcome(welcome) => welcome.label(),
        }
    }

    pub fn icon(&self) -> Dom {
        match self {
            Activity::Editor(editor) => editor.icon(),
            Activity::Welcome(welcome) => welcome.icon(),
        }
    }

    fn render_tab(
        this: &Rc<Activity>,
        panel: &Rc<ActivityPanel>
    ) -> Dom {
        let close_icon = svg!("svg", {
            .attr("height", "1em")
            .attr("viewBox", "0 0 24 24")
            .child(svg!("path", {
                .attr("d", CLOSE_ICON_PATH)
            }))
        });

        let mouse_over = Mutable::new(false);
        let mouse_over_close = Mutable::new(false);
        let is_active = panel.active_activity.signal_ref(clone!(this => move |active_activity| {
            active_activity.as_ref().is_some_and(|active_activity| Rc::ptr_eq(active_activity, &this))
        }));

        block!("py-3", "px-3", {
            .style("cursor", "pointer")
            .event(clone!(mouse_over => move |_: events::PointerOver| {
                mouse_over.set_neq(true);
            }))
            .event(clone!(mouse_over => move |_: events::PointerOut| {
                mouse_over.set_neq(false);
            }))
            .event(clone!(panel, this => move |_: events::PointerDown| {                
                panel.active_activity.set(Some(this.clone()))
            }))
            .class_signal("has-background-white", signal::or(is_active, mouse_over.signal()))
            .child(icon_text!({
                .child(icon!({
                    .child(this.icon())
                }))
            }))
            .child(this.label())
            .apply_if(matches!(**this, Activity::Editor(_)), |dom| {
                dom.child(icon!({
                    .event(clone!(mouse_over_close => move |_: events::PointerOver| {
                        mouse_over_close.set_neq(true);
                    }))
                    .event(clone!(mouse_over_close => move |_: events::PointerOut| {
                        mouse_over_close.set_neq(false);
                    }))
                    .event_with_options(&EventOptions::preventable(), clone!(panel, this => move |ev: events::PointerDown| {
                        ev.stop_propagation();
                        panel.close_tab(this.clone());
                    }))
                    .class_signal("has-background-white-ter", mouse_over_close.signal())
                    .class_signal("is-invisible", signal::not(mouse_over.signal()))
                    .child(close_icon)
                }))
            })
        })
    }
}

pub struct ActivityPanel {
    activities: MutableVec<Rc<Activity>>,
    active_activity: Mutable<Option<Rc<Activity>>>,
    context_menu_state: Rc<ContextMenuState>
}

impl Default for ActivityPanel {
    fn default() -> Self {
        let welcome = Rc::new(Activity::Welcome(Rc::new(welcome::Welcome::new())));
        
        Self {
            activities: vec![welcome.clone()].into(),
            active_activity: Some(welcome).into(),
            context_menu_state: ContextMenuState::new()
        }
    }
}

const CLOSE_ICON_PATH: &str = "M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z";

impl ActivityPanel {
    // Closes a specific tab
    fn close_tab(&self, activity: Rc<Activity>) {
        self.activities.lock_mut().retain(|a| !Rc::ptr_eq(a, &activity));
        if let Some(active) = self.active_activity.lock_ref().as_ref() {
            if Rc::ptr_eq(active, &activity) {
                // If the closed tab was active, set another tab as active or None if no tabs remain
                *self.active_activity.lock_mut() = self.activities.lock_ref().first().cloned();
            }
        }
        self.context_menu_state.show_menu.set(false); // Ensure context menu is hidden after close
    }

    // Closes all open tabs
    fn close_all_tabs(&self) {
        self.activities.lock_mut().clear();
        self.active_activity.set(None);
        self.context_menu_state.show_menu.set(false); // Ensure context menu is hidden after close all
    }

    // Splits the selected tab by creating a new instance of the same activity type
    fn split_tab(&self, activity: Rc<Activity>) {
        let new_activity = match &*activity {
            Activity::Editor(editor) => Activity::Editor(editor.clone()),
            Activity::Welcome(welcome) => Activity::Welcome(Rc::new(welcome::Welcome::new())),
        };
        self.activities.lock_mut().push_cloned(Rc::new(new_activity));
    }

    pub fn render(
        this: &Rc<ActivityPanel>,
        workspace_command_rx: crate::WorkspaceCommandReceiver,
        width: impl Signal<Item = u32> + 'static,
        height: impl Signal<Item = u32> + 'static
    ) -> dominator::Dom {
        let activity_count = this.activities.signal_vec_cloned().len().broadcast();
        let width = width.broadcast();
        let height = height.broadcast();
        let context_menu_state = this.context_menu_state.clone();
        
        columns!("is-gapless", "is-mobile", "is-multiline", {
            .future(workspace_command_rx.for_each(clone!(this => move |command| clone!(this => async move {
                match command {
                    crate::WorkspaceCommand::OpenFile(file) => {
                        let mut activities = this.activities.lock_mut();
                        let editor = activities.iter()
                            .find(|activity| match &***activity {
                                Activity::Editor(editor) => Rc::ptr_eq(&editor.file, &file),
                                _ => false,
                            })
                            .cloned()
                            .unwrap_or_else(move || {
                                let editor = Rc::new(Activity::Editor(Rc::new(editor::Editor::new(file))));
                                activities.push_cloned(editor.clone());
                                editor
                            });
                        this.active_activity.set(Some(editor));
                    }
                }
            }))))

            .child_signal(activity_count.signal().map(clone!(height => move |count| {
                if count == 0 {
                    Some(Self::render_background(height.signal()))
                } else {
                    None
                }
            })))

            .child(column!("is-full", {
                .class("has-background-white-ter")
                .child(columns!("is-gapless", "is-mobile", {
                    .children_signal_vec(this.activities.signal_vec_cloned().map(clone!(this => move |activity| {
                        column!("is-narrow", {
                            .child(Activity::render_tab(&activity, &this))
                            .event_with_options(&EventOptions::preventable(), clone!(context_menu_state => move |event: events::ContextMenu| {
                                event.prevent_default();  
                                context_menu_state.show_menu.set(true); 
                                context_menu_state.menu_position.set((event.x(), event.y())); 
                            }))
                            .child_signal(context_menu_state.show_menu.signal_ref(clone!(context_menu_state, this => move |&show| {
                                if show {
                                    Some(html!("div", {
                                        .class("context-menu")
                                        .style("position", "absolute")
                                        .style("background-color", "lightgray")
                                        .style("border", "1px solid black")
                                        .style("padding", "10px")
                                        .style("z-index", "1000")
                                        .style_signal("left", context_menu_state.menu_position.signal_ref(|(x, _y)| format!("{}px", x)))
                                        .style_signal("top", context_menu_state.menu_position.signal_ref(|(_x, y)| format!("{}px", y)))
                                        .children(&mut [
                                            html!("div", {
                                                .text("Close Tab")
                                                .style("cursor", "pointer")
                                                .event(clone!(context_menu_state, this, activity => move |_event: events::MouseDown| {
                                                    this.close_tab(activity.clone());
                                                    context_menu_state.show_menu.set_neq(false);
                                                }))
                                            }),
                                            html!("div", {
                                                .text("Close All Tabs")
                                                .style("cursor", "pointer")
                                                .event(clone!(context_menu_state, this => move |_event: events::MouseDown| {
                                                    this.close_all_tabs();
                                                    context_menu_state.show_menu.set_neq(false);
                                                }))
                                            }),
                                            html!("div", {
                                                .text("Split Tab")
                                                .style("cursor", "pointer")
                                                .event(clone!(context_menu_state, this, activity => move |_event: events::MouseDown| {
                                                    this.split_tab(activity.clone());
                                                    context_menu_state.show_menu.set_neq(false);
                                                }))
                                            }),
                                        ])
                                    }))
                                } else {
                                    None
                                }
                            })))
                        })
                    })))
                }))
            }))
        })
    }

    fn render_background(height: impl Signal<Item = u32> + 'static) -> Dom {
        block!("px-3", {
            .class("is-centered")
            .text("No tabs open yet")
        })
    }
}


