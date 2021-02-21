#[derive(Debug)]
pub(crate) enum Message {
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
