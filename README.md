# extreme
super boring async function runner

docs:

```
/// Run a `Future`.
pub fn run<F, O>(f: F) -> O
where
    F: Future<Output = O> + Unpin
```
