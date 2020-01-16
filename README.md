# extreme

super boring async function runner, less than 50 lines of code.

docs:

```
/// Run a `Future`.
pub fn run<F, O>(f: F) -> O
where
    F: Future<Output = O>
```
