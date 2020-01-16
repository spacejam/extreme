# extreme

super boring async function runner, less than 40 lines of code.

docs:

```
/// Run a `Future`.
pub fn run<F, O>(f: F) -> O
where
    F: Future<Output = O>
```
