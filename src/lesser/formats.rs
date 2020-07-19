#[derive(Debug)]
pub(crate) enum Message {
    Empty,
    ScrollDownPage,
    ScrollDown,
    ScrollUpPage,
    ScrollUp,
    ScrollLeft,
    ScrollRight,
    ScrollToBeginning,
    ScrollToEnd,
    Exit,
    Reload,
}
