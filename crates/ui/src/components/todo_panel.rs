use gpui::{
    actions, div, list, prelude::*, Action, AppContext, DismissEvent, Element,
    EventEmitter, FocusHandle, InteractiveElement, ListState, MouseDownEvent,
    ParentElement, StyledText, View, ViewContext, WeakView, WindowContext,
    ListHeaderStyle, ListItem,
};
use project::{TodoEntry, Project};
use workspace::{ViewId, Workspace};

pub struct TodoPanel {
    workspace: WeakView<Workspace>,
    todos: Vec<TodoEntry>,
    focus_handle: FocusHandle,
    selected_index: Option<usize>,
} 

impl TodoPanel {
    pub fn new(workspace: &View<Workspace>, cx: &mut ViewContext<Self>) -> Self {
        Self {
            workspace: workspace.weak_handle(), 
            todos: Vec::new(),
            focus_handle: cx.focus_handle(),
            selected_index: None,
        }
    }

    pub fn set_todos(&mut self, todos: Vec<TodoEntry>, cx: &mut ViewContext<Self>) {
        self.todos = todos;
        cx.notify();
    }
}

impl EventEmitter<()> for TodoPanel {}

impl FocusableView for TodoPanel {
    fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TodoPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl Element {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                list::build(ListState::new(
                    &self.todos,
                    cx,
                    |todo, cx| ListItem::new(format!("todo-{}", todo.line))
                        .inset_0()
                        .child(
                            h_flex()
                                .gap_2()
                                .child(StyledText::new(format!("{:?}", todo.kind)))
                                .child(StyledText::new(&todo.message))
                                .child(StyledText::new(format!("{}:{}", 
                                    todo.file_path.path.display(),
                                    todo.line
                                )))
                        )
                ))
            )
    }
}
