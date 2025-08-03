//! Scene tab UI components for multi-scene navigation

use egui::{*, self};
use crate::scene::{SceneManager, SceneId, SceneSummary};

/// Scene tab interaction events
#[derive(Clone, Debug)]
pub enum SceneTabEvent {
    /// User clicked on a scene tab to switch
    SwitchToScene(SceneId),
    /// User requested to add a new scene
    AddScene,
    /// User right-clicked on a scene tab
    ContextMenu(SceneId, Pos2),
    /// User double-clicked to rename scene
    RenameScene(SceneId),
    /// User confirmed rename with new name
    ConfirmRename(SceneId, String),
    /// User cancelled rename
    CancelRename,
    /// User clicked close button on scene tab
    CloseScene(SceneId),
}

/// State for scene tab interactions
#[derive(Clone, Debug, Default)]
pub struct SceneTabState {
    /// Scene currently being renamed
    pub renaming_scene: Option<(SceneId, String)>,
    /// Context menu state
    pub context_menu: Option<SceneContextMenuState>,
    /// Drag and drop state for reordering
    pub dragging_scene: Option<SceneId>,
    /// Hover state for visual feedback
    pub hovered_scene: Option<SceneId>,
}

/// Context menu state for scene tabs
#[derive(Clone, Debug)]
pub struct SceneContextMenuState {
    pub scene_id: SceneId,
    pub position: Pos2,
}

/// Scene tabs widget for displaying and managing multiple scenes
pub struct SceneTabs<'a> {
    scene_manager: &'a SceneManager,
    state: &'a mut SceneTabState,
}

impl<'a> SceneTabs<'a> {
    /// Create new scene tabs widget
    pub fn new(scene_manager: &'a SceneManager, state: &'a mut SceneTabState) -> Self {
        Self {
            scene_manager,
            state,
        }
    }

    /// Show scene tabs and return any events
    pub fn show(mut self, ui: &mut Ui) -> Vec<SceneTabEvent> {
        let mut events = Vec::new();

        // Scene tabs container
        ui.horizontal(|ui| {
            ui.style_mut().spacing.item_spacing.x = 0.0;

            let scenes = self.scene_manager.get_scene_summaries();
            let active_scene_id = self.scene_manager.get_active_scene_id();

            // Draw each scene tab
            for scene in &scenes {
                let is_active = active_scene_id == Some(&scene.id);
                
                if let Some(tab_events) = self.draw_scene_tab(ui, scene, is_active) {
                    events.extend(tab_events);
                }
            }

            // Add scene button
            ui.separator();
            if ui.button("➕").on_hover_text("Add new scene").clicked() {
                events.push(SceneTabEvent::AddScene);
            }
        });

        // Show context menu if active
        if let Some(menu_state) = self.state.context_menu.clone() {
            if let Some(menu_events) = self.show_context_menu(ui, &menu_state) {
                events.extend(menu_events);
            }
        }

        events
    }

    /// Draw a single scene tab
    fn draw_scene_tab(&mut self, ui: &mut Ui, scene: &SceneSummary, is_active: bool) -> Option<Vec<SceneTabEvent>> {
        let mut events = Vec::new();

        // Check if this scene is being renamed
        let is_renaming = self.state.renaming_scene.as_ref()
            .map(|(id, _)| id == &scene.id)
            .unwrap_or(false);

        if is_renaming {
            // Show text edit for renaming
            if let Some((_, ref mut new_name)) = &mut self.state.renaming_scene {
                let text_edit = ui.text_edit_singleline(new_name);
                
                if text_edit.lost_focus() {
                    if ui.input(|i| i.key_pressed(Key::Enter)) {
                        // Confirm rename
                        events.push(SceneTabEvent::ConfirmRename(scene.id.clone(), new_name.clone()));
                        self.state.renaming_scene = None;
                    } else if ui.input(|i| i.key_pressed(Key::Escape)) {
                        // Cancel rename
                        events.push(SceneTabEvent::CancelRename);
                        self.state.renaming_scene = None;
                    }
                }

                // Auto-focus the text edit
                if !text_edit.has_focus() {
                    text_edit.request_focus();
                }
            }
        } else {
            // Normal tab display
            let tab_rect = self.draw_tab_background(ui, scene, is_active);
            
            // Tab content
            let response = ui.allocate_rect(tab_rect, Sense::click());
            
            // Tab text with modification indicator
            let tab_text = if scene.modified {
                format!("{}*", scene.name)
            } else {
                scene.name.clone()
            };

            // Draw tab text
            let text_color = if is_active {
                ui.style().visuals.strong_text_color()
            } else {
                ui.style().visuals.text_color()
            };

            ui.painter().text(
                tab_rect.center(),
                Align2::CENTER_CENTER,
                &tab_text,
                FontId::default(),
                text_color,
            );

            // Close button for non-active tabs (if more than one scene)
            if !is_active && self.scene_manager.scene_count() > 1 {
                let close_button_rect = Rect::from_center_size(
                    pos2(tab_rect.max.x - 10.0, tab_rect.center().y),
                    vec2(12.0, 12.0),
                );
                
                let close_response = ui.allocate_rect(close_button_rect, Sense::click());
                if close_response.hovered() {
                    ui.painter().rect_filled(
                        close_button_rect,
                        2.0,
                        ui.style().visuals.widgets.hovered.bg_fill,
                    );
                }
                
                ui.painter().text(
                    close_button_rect.center(),
                    Align2::CENTER_CENTER,
                    "×",
                    FontId::default(),
                    text_color,
                );

                if close_response.clicked() {
                    events.push(SceneTabEvent::CloseScene(scene.id.clone()));
                }
            }

            // Handle tab interactions
            if response.clicked() {
                if !is_active {
                    events.push(SceneTabEvent::SwitchToScene(scene.id.clone()));
                }
            } else if response.double_clicked() {
                events.push(SceneTabEvent::RenameScene(scene.id.clone()));
                self.state.renaming_scene = Some((scene.id.clone(), scene.name.clone()));
            } else if response.secondary_clicked() {
                if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                    events.push(SceneTabEvent::ContextMenu(scene.id.clone(), pos));
                    self.state.context_menu = Some(SceneContextMenuState {
                        scene_id: scene.id.clone(),
                        position: pos,
                    });
                }
            }

            // Hover state
            if response.hovered() {
                self.state.hovered_scene = Some(scene.id.clone());
            }

            // Tooltip with scene information
            if response.hovered() {
                response.on_hover_ui(|ui| {
                    ui.vertical(|ui| {
                        ui.label(format!("Scene: {}", scene.name));
                        ui.label(format!("Layers: {}", scene.layer_count));
                        ui.label(format!("Frames: {}", scene.frame_count));
                        if scene.modified {
                            ui.colored_label(Color32::YELLOW, "● Modified");
                        }
                    });
                });
            }
        }

        if events.is_empty() {
            None
        } else {
            Some(events)
        }
    }

    /// Draw tab background with appropriate styling
    fn draw_tab_background(&self, ui: &mut Ui, scene: &SceneSummary, is_active: bool) -> Rect {
        let desired_size = vec2(120.0, 28.0);
        let (rect, _) = ui.allocate_exact_size(desired_size, Sense::hover());

        // Tab background
        let bg_color = if is_active {
            ui.style().visuals.selection.bg_fill
        } else if self.state.hovered_scene.as_ref() == Some(&scene.id) {
            ui.style().visuals.widgets.hovered.bg_fill
        } else {
            ui.style().visuals.widgets.inactive.bg_fill
        };

        // Draw tab shape (rounded top corners)
        let mut tab_rect = rect;
        tab_rect.max.y -= 2.0; // Leave space for bottom border

        ui.painter().rect_filled(
            tab_rect,
            4.0,
            bg_color,
        );

        // Active tab gets a bottom border
        if is_active {
            ui.painter().line_segment(
                [
                    pos2(tab_rect.min.x, tab_rect.max.y),
                    pos2(tab_rect.max.x, tab_rect.max.y),
                ],
                Stroke::new(2.0, ui.style().visuals.selection.stroke.color),
            );
        }

        // Modified indicator
        if scene.modified {
            let indicator_pos = pos2(tab_rect.max.x - 18.0, tab_rect.min.y + 6.0);
            ui.painter().circle_filled(indicator_pos, 3.0, Color32::YELLOW);
        }

        rect
    }

    /// Show context menu for scene tab (simplified approach)
    fn show_context_menu(&mut self, _ui: &mut Ui, _menu_state: &SceneContextMenuState) -> Option<Vec<SceneTabEvent>> {
        // For now, just clear the context menu state and return empty events
        // The actual context menu will be handled directly in the tab response
        self.state.context_menu = None;
        None
    }
}

/// Scene navigation widget with keyboard shortcuts
pub struct SceneNavigation<'a> {
    scene_manager: &'a SceneManager,
}

impl<'a> SceneNavigation<'a> {
    pub fn new(scene_manager: &'a SceneManager) -> Self {
        Self { scene_manager }
    }

    /// Handle keyboard shortcuts for scene navigation
    pub fn handle_shortcuts(&self, ctx: &Context) -> Vec<SceneTabEvent> {
        let mut events = Vec::new();

        ctx.input(|i| {
            // Ctrl+PageUp/PageDown for scene navigation
            if i.modifiers.ctrl {
                if i.key_pressed(Key::PageUp) {
                    // Previous scene
                    if let Some(current_id) = self.scene_manager.get_active_scene_id() {
                        let scenes = self.scene_manager.get_scene_summaries();
                        if let Some(current_idx) = scenes.iter().position(|s| &s.id == current_id) {
                            if current_idx > 0 {
                                let prev_scene = &scenes[current_idx - 1];
                                events.push(SceneTabEvent::SwitchToScene(prev_scene.id.clone()));
                            }
                        }
                    }
                } else if i.key_pressed(Key::PageDown) {
                    // Next scene
                    if let Some(current_id) = self.scene_manager.get_active_scene_id() {
                        let scenes = self.scene_manager.get_scene_summaries();
                        if let Some(current_idx) = scenes.iter().position(|s| &s.id == current_id) {
                            if current_idx < scenes.len() - 1 {
                                let next_scene = &scenes[current_idx + 1];
                                events.push(SceneTabEvent::SwitchToScene(next_scene.id.clone()));
                            }
                        }
                    }
                }
            }

            // Ctrl+T for new scene
            if i.modifiers.ctrl && i.key_pressed(Key::T) {
                events.push(SceneTabEvent::AddScene);
            }
        });

        events
    }
}