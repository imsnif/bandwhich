/// An action that can be dispatched to a focused pane.
///
/// New per-widget behaviours should be added as variants here and handled in
/// [`crate::display::Ui::handle_widget_action`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetAction {
    TogglePause,
}
